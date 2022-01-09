use std::fmt::Write;
use std::path::PathBuf;
use std::time::Duration;

use colored::Colorize;

#[derive(Default)]
pub struct Summary {
    pub scan_error_count: u32,
    pub error_count: u32,
    pub skipped_count: u32,
    pub duplicate_count: u32,
    pub exif_error_count: u32,
    pub copy_count: u32,
    pub copied_bytes: u64,
    pub duration: Duration,
    pub errored_files: Vec<PathBuf>,
    pub duplicate_files: Vec<PathBuf>,
    pub exif_errored_files: Vec<PathBuf>,
}

impl Summary {
    pub fn init() -> Self {
        Default::default()
    }

    pub fn mark_scan_error(&mut self) {
        self.scan_error_count += 1;
    }

    pub fn mark_error(&mut self, path: PathBuf) {
        self.error_count += 1;
        self.errored_files.push(path);
    }

    pub fn mark_skipped(&mut self) {
        self.skipped_count += 1;
    }

    pub fn mark_duplicate(&mut self, path: PathBuf) {
        self.duplicate_count += 1;
        self.duplicate_files.push(path);
    }

    pub fn mark_exif_error(&mut self, path: PathBuf) {
        self.exif_error_count += 1;
        self.exif_errored_files.push(path);
    }

    pub fn mark_copied(&mut self, len: u64) {
        self.copy_count += 1;
        self.copied_bytes += len;
    }

    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    pub fn display(&self) -> String {
        let mut display: String = "\n".to_string();
        writeln!(
            display,
            "{} in {}",
            "Completed".green(),
            humantime::format_duration(self.duration)
        )
        .unwrap();
        writeln!(
            display,
            "{} {} files totalling {}",
            "Copied".green(),
            self.copy_count,
            bytesize::to_string(self.copied_bytes, true)
        )
        .unwrap();
        if self.skipped_count > 0 {
            writeln!(
                display,
                "{} copying {} files since they were already present at the target",
                "Skipped".cyan(),
                self.skipped_count
            )
            .unwrap();
        }
        if self.exif_error_count > 0 {
            writeln!(display,
                "{} reading the exif data for {} files. They were copied using the file modified time - ", 
                "Error".yellow(), 
                self.exif_error_count)
            .unwrap();
            for path in &self.exif_errored_files {
                writeln!(display, "{}", path.display()).unwrap();
            }
        }
        if self.duplicate_count > 0 {
            writeln!(
                display,
                "{} copying {} files since they were present at the target but was of a different size - ", "Skipped".red(),
                self.duplicate_count
            )
            .unwrap();
            for path in &self.duplicate_files {
                writeln!(display, "{}", path.display()).unwrap();
            }
        }
        if self.scan_error_count > 0 {
            writeln!(
                display,
                "{} to scan {} files.",
                "Failed".red(),
                self.scan_error_count
            )
            .unwrap();
        }
        if self.error_count > 0 {
            writeln!(
                display,
                "{} to copy {} files. The following files were not copied - ",
                "Failed".red(),
                self.error_count
            )
            .unwrap();
            for path in &self.errored_files {
                writeln!(display, "{}", path.display()).unwrap();
            }
        }
        display
    }
}
