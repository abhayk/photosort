use assert_cmd::prelude::*;
use assert_fs::{
    assert::PathAssert,
    fixture::{FileTouch, PathChild},
};
use filetime::FileTime;
use photosort::Summary;
use predicates::prelude::predicate;
use std::{env, fs, path::PathBuf, process::Command, time::Duration};

// the files in the data folder correspond to the following files
// from the exif-samples GitHub repo - https://github.com/ianare/exif-samples
//
// jpeg with valid exif - exif-samples/jpg/Canon_40D.jpg
// jpeg with no exif - exif-samples/jpg/invalid/image00971.jpg
// jpeg with no datetimeoriginal tag in exif - exif-samples/jpg/Canon_40D_photoshop_import.jpg

#[test]
fn cli_test() -> Result<(), Box<dyn std::error::Error>> {
    setup()?;

    let temp_dir = assert_fs::TempDir::new()?;

    let mut cmd = Command::cargo_bin("photosort")?;
    cmd.arg("--source-dir").arg("tests/data");
    cmd.arg("--target-dir").arg(temp_dir.path());

    let expected_summary = Summary {
        scan_error_count: 0,
        error_count: 0,
        skipped_count: 0,
        duplicate_count: 0,
        copy_count: 4,
        copied_bytes: 181870,
        duration: Duration::new(0, 0),
        duplicate_files: Vec::new(),
        errored_files: Vec::new(),
        exif_error_count: 0,
        exif_errored_files: Vec::new(),
    };

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(strip_timestamp_from_summary(
            expected_summary,
        )));

    let expected_paths = vec![
        // jpeg with valid exif
        r"2008/May/30/jpeg_with_valid_exif.jpg",
        // jpeg with no exif, target path based on the file modified time.
        r"2022/January/6/jpeg_with_no_exif.jpg",
        // jpeg with valid exif but no datetime, target path based on the file modified time.
        r"2022/January/6/jpeg_with_valid_exif_but_no_datetimeoriginal.jpg",
        // non image file, target path based on the file modified time.
        r"2022/January/6/non_image_file.txt",
    ];

    for path in &expected_paths {
        temp_dir.child(path).assert(predicate::path::exists());
    }

    let expected_summary_second_run = Summary {
        scan_error_count: 0,
        error_count: 0,
        skipped_count: 4,
        duplicate_count: 0,
        copy_count: 0,
        copied_bytes: 0,
        duration: Duration::new(0, 0),
        duplicate_files: Vec::new(),
        errored_files: Vec::new(),
        exif_error_count: 0,
        exif_errored_files: Vec::new(),
    };

    // run the same command again. all files should get skipped.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(strip_timestamp_from_summary(
            expected_summary_second_run,
        )));

    // make sure the existing files are still there.
    for path in &expected_paths {
        temp_dir.child(path).assert(predicate::path::exists());
    }

    // create a new source directory
    let temp_source = assert_fs::TempDir::new()?;
    // create a file with the same name as the existing data set.
    // the existing file has some data but this file is empty.
    let file = temp_source.child("non_image_file.txt");
    file.touch()?;
    set_default_modified_time(file.path().to_path_buf())?;

    let mut cmd = Command::cargo_bin("photosort")?;
    cmd.arg("--source-dir").arg(temp_source.path());
    cmd.arg("--target-dir").arg(temp_dir.path());

    let expected_summary_duplicate_file = Summary {
        scan_error_count: 0,
        error_count: 0,
        skipped_count: 0,
        duplicate_count: 1,
        copy_count: 0,
        copied_bytes: 0,
        duration: Duration::new(0, 0),
        duplicate_files: Vec::new(),
        errored_files: Vec::new(),
        exif_error_count: 0,
        exif_errored_files: Vec::new(),
    };

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(strip_timestamp_from_summary(
            expected_summary_duplicate_file,
        )));

    // make sure the existing files are still there.
    for path in &expected_paths {
        temp_dir.child(path).assert(predicate::path::exists());
    }

    Ok(())
}

fn setup() -> Result<(), Box<dyn std::error::Error>> {
    // disable colour for outputs. enabling colour screws up the stdout assertions.
    env::set_var("NO_COLOR", true.to_string());

    for entry in fs::read_dir("tests/data")? {
        set_default_modified_time(entry?.path())?;
    }
    Ok(())
}

fn set_default_modified_time(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 6-Jan-2022
    filetime::set_file_mtime(path, FileTime::from_unix_time(1641495779, 0))?;
    Ok(())
}

fn strip_timestamp_from_summary(summary: Summary) -> String {
    summary
        .display()
        .lines()
        .skip(2)
        .collect::<Vec<&str>>()
        .join("\n")
}
