use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Month, NaiveDate, Utc};
use clap::Parser;
use colored::*;
use exif::{In, Tag};
use photosort::Summary;
use std::io::Write;
use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    time::Instant,
};
use walkdir::{DirEntry, WalkDir};

use num_traits::cast::FromPrimitive;

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    #[clap(short, long, parse(from_os_str))]
    source_path: PathBuf,

    #[clap(short, long, parse(from_os_str))]
    target_path: PathBuf,
}

static EXIF_COMPATIBLE_EXTENSIONS: [&str; 2] = ["jpg", "jpeg"];

fn main() {
    let args = Args::parse();
    if !args.source_path.exists() || !args.source_path.is_dir() {
        eprintln!("The source path is invalid. Please make sure it exists and is a directory.");
        std::process::exit(1);
    }
    if !args.target_path.exists() || !args.target_path.is_dir() {
        eprintln!("The target path is invalid. Please make sure it exists and is a directory.");
        std::process::exit(1);
    }

    let stats = copy_files(args.source_path, args.target_path);
    println!("{}", stats.display());
}

fn copy_files(source_path: PathBuf, target_path: PathBuf) -> Summary {
    let now = Instant::now();

    let mut summary = Summary::init();

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    for entry in WalkDir::new(source_path) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("{} while scanning - [{}]", "Error".red(), err);
                summary.mark_scan_error();
                continue;
            }
        };

        // walkdir also returns directory entries. Skip them.
        if entry.file_type().is_dir() {
            continue;
        }

        // get the file timestamp preferably from the exif data
        let file_date = match get_file_date(&entry) {
            Ok(file_date) => file_date,
            Err(err) => {
                eprintln!(
                    "{} while reading the file date for the file {} - [{}]",
                    "Error".red(),
                    entry.path().display(),
                    err
                );
                summary.mark_error(entry.into_path());
                continue;
            }
        };

        // convert the timestamp to a path at the target
        let target_path = get_target_path(&entry, file_date, &target_path);

        // if the file already exists at the target then skip it
        if target_path.exists() {
            let source_len = match entry.metadata() {
                Ok(metadata) => metadata.len(),
                Err(err) => {
                    eprintln!(
                        "{} while trying to read the size of the source file {} - [{}]",
                        "Error".red(),
                        entry.path().display(),
                        err
                    );
                    summary.mark_error(entry.into_path());
                    continue;
                }
            };
            let target_len = match target_path.metadata() {
                Ok(metadata) => metadata.len(),
                Err(err) => {
                    eprintln!(
                        "{} while trying to read the size of the target file {} - [{}]",
                        "Error".red(),
                        target_path.display(),
                        err
                    );
                    summary.mark_error(entry.into_path());
                    continue;
                }
            };
            if source_len == target_len {
                writeln!(
                    lock,
                    "{} {}. It's already present at {}",
                    "Skipping".cyan(),
                    entry.path().display(),
                    target_path.display()
                )
                .expect("Error writing to stdout");
                summary.mark_skipped();
            } else {
                eprintln!("A file with the same name but a different size exists at the target. This file would be skipped for copying- {}", entry.path().display());
                summary.mark_duplicate(entry.into_path());
            }
            continue;
        }

        // create the parent directory structure if it does not exist
        if let Some(parent_path) = target_path.parent() {
            match fs::create_dir_all(parent_path) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!(
                        "{} creating the parent directory {} at the target - [{}]",
                        "Error".red(),
                        parent_path.display(),
                        err
                    );
                    summary.mark_error(entry.into_path());
                    continue;
                }
            }
        }

        // copy the file
        match fs::copy(entry.path(), &target_path) {
            Ok(bytes) => {
                writeln!(
                    lock,
                    "{} {} to {}",
                    "Copied".green().bold(),
                    entry.path().display(),
                    target_path.display()
                )
                .expect("Error writing to stdout");
                summary.mark_copied(bytes);
            }
            Err(err) => {
                eprintln!(
                    "{} while copying {} to {} - [{}]",
                    "Error".red(),
                    entry.path().display(),
                    target_path.display(),
                    err
                );
                summary.mark_error(entry.into_path());
            }
        }
    }
    summary.set_duration(now.elapsed());

    summary
}

fn get_target_path(entry: &DirEntry, file_date: NaiveDate, target_root: &Path) -> PathBuf {
    let mut final_path = PathBuf::new();
    final_path.push(target_root);
    final_path.push(file_date.year().to_string());
    final_path.push(Month::from_u32(file_date.month()).unwrap().name());
    final_path.push(file_date.day().to_string());
    final_path.push(entry.file_name());

    final_path
}

fn get_file_date(entry: &DirEntry) -> Result<NaiveDate> {
    if exif_compatible_extension(entry) {
        match get_date_from_exif(entry) {
            Ok(date) => return Ok(date),
            Err(err) => {
                eprintln!(
                    "{} Could not read exif from the file {} - [{}]. Will default to file modified time.", "Warning.".yellow(),
                    entry.path().display(),
                    err.root_cause()
                );
            }
        };
    }
    get_date_from_file(entry)
}

fn get_date_from_file(entry: &DirEntry) -> Result<NaiveDate> {
    let datetime: DateTime<Utc> = entry
        .metadata()
        .context("Failed to read file metadata")?
        .modified()
        .context("Failed to read file modified time")?
        .into();
    Ok(NaiveDate::from_ymd(
        datetime.year(),
        datetime.month(),
        datetime.day(),
    ))
}

fn get_date_from_exif(entry: &DirEntry) -> Result<NaiveDate> {
    let file = File::open(entry.path()).context("Failed to open the file for reading exif")?;
    let mut bufreader = BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let datetime = exifreader
        .read_from_container(&mut bufreader)?
        .get_field(Tag::DateTimeOriginal, In::PRIMARY)
        .context("No datetime in the exif data")?
        .display_value()
        .to_string();
    let datetime = NaiveDate::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S")
        .context("Failed to parse the exif datetime")?;

    Ok(datetime)
}

fn exif_compatible_extension(entry: &DirEntry) -> bool {
    entry.path().extension().map_or(false, |extension| {
        EXIF_COMPATIBLE_EXTENSIONS
            .iter()
            .any(|&e| e == extension.to_ascii_lowercase())
    })
}
