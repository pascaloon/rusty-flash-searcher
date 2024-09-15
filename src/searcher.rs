use regex::Regex;
use rayon::prelude::*;
use memmap2::Mmap;
use std::fs::{self, File};
use std::path::Path;
use std::sync::Mutex;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub(crate) struct Searcher {
    regex: Regex,
    file_regex: Regex,
    output_mutex: Mutex<()>,
    colored: bool,
}

impl Searcher {
    pub(crate) fn new(regex: &str, file_filter: &str, colored: bool) -> Self {
        let regex_str = format!("(?i)({})", regex);
        let file_regex_str = format!("(?i){}", file_filter);
        let regex = Regex::new(&regex_str).unwrap();
        let file_regex = Regex::new(&file_regex_str).unwrap();
        Self {
            regex,
            file_regex,
            output_mutex: Mutex::new(()),
            colored,
        }
    }

    pub fn search(&self, search_dir: &str) {
        self.search_internal(search_dir);
    }

    fn search_internal(&self, search_dir: &str) {
        let read_dir = match fs::read_dir(search_dir) {
            Ok(rd) => rd,
            Err(e) => {
                let _lock = self.output_mutex.lock().unwrap();
                eprintln!("Error occurred for directory '{}': {}", search_dir, e);
                return;
            }
        };

        read_dir.par_bridge().for_each(|entry| {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_dir() {
                        self.search_internal(&path.to_string_lossy());
                    } else if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        if self.file_regex.is_match(file_name) {
                            self.process_file(&path);
                        }
                    }
                }
                Err(e) => {
                    let _lock = self.output_mutex.lock().unwrap();
                    eprintln!("Error reading entry in '{}': {}", search_dir, e);
                }
            }
        });
    }

    fn process_file(&self, file_path: &Path) {
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(e) => {
                let _lock = self.output_mutex.lock().unwrap();
                eprintln!("Error occurred for file '{}': {}", file_path.display(), e);
                return;
            }
        };

        let mmap = match unsafe { Mmap::map(&file) } {
            Ok(mmap) => mmap,
            Err(e) => {
                let _lock = self.output_mutex.lock().unwrap();
                eprintln!("Error mapping file '{}': {}", file_path.display(), e);
                return;
            }
        };

        let data = match std::str::from_utf8(&mmap[..]) {
            Ok(data) => data,
            Err(e) => {
                let _lock = self.output_mutex.lock().unwrap();
                eprintln!("Error reading file '{}': {}", file_path.display(), e);
                return;
            }
        };

        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let mut line_number = 1;

        // Process the file line by line
        for line in data.split_inclusive('\n') {
            if let Some(mat) = self.regex.find(line) {
                let _lock = self.output_mutex.lock().unwrap();

                if self.colored {
                    // Output file and line number
                    stdout
                        .set_color(ColorSpec::new().set_fg(Some(Color::Ansi256(8))).set_bold(false))
                        .unwrap();
                    write!(
                        &mut stdout,
                        "{}:{}:",
                        file_path.display(),
                        line_number
                    )
                    .unwrap();

                    // Reset color
                    stdout.set_color(ColorSpec::new().set_reset(true)).unwrap();

                    // Output the line with highlighted match
                    let start = mat.start();
                    let end = mat.end();

                    write!(&mut stdout, "{}", &line[..start]).unwrap();

                    stdout
                        .set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))
                        .unwrap();
                    write!(&mut stdout, "{}", &line[start..end]).unwrap();

                    stdout.set_color(ColorSpec::new().set_reset(true)).unwrap();
                    write!(&mut stdout, "{}", &line[end..]).unwrap();
                } else {
                    print!("{}:{}:{}", file_path.display(), line_number, line);
                }
            }

            line_number += 1;
        }
    }
}