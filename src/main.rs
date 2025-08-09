use clap::{Arg, Command};
use satch::is_match;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

fn main() {
    let matches = Command::new("satch")
        .version("0.1.0")
        .author("ushironoko")
        .about("High-performance glob pattern matching CLI tool")
        .arg(
            Arg::new("pattern")
                .help("Glob pattern to match against")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("paths")
                .help("File paths to test (default: read from stdin)")
                .action(clap::ArgAction::Append)
                .num_args(0..)
                .last(true),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .help("List files in current directory matching the pattern")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .help("Search recursively in directories")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Show verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("basename")
                .short('b')
                .long("basename")
                .help("Match against basename only (ignore directory path)")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let pattern = matches.get_one::<String>("pattern").unwrap();
    let list_mode = matches.get_flag("list");
    let recursive = matches.get_flag("recursive");
    let verbose = matches.get_flag("verbose");
    let basename_mode = matches.get_flag("basename");

    if list_mode {
        list_matching_files(pattern, recursive, verbose, basename_mode);
    } else if let Some(paths) = matches.get_many::<String>("paths") {
        for path in paths {
            check_path_match(pattern, path, verbose, basename_mode);
        }
    } else {
        read_from_stdin(pattern, verbose, basename_mode);
    }
}

fn list_matching_files(pattern: &str, recursive: bool, verbose: bool, basename_mode: bool) {
    if recursive {
        list_files_recursive(".", pattern, verbose, basename_mode);
    } else {
        list_files_in_directory(".", pattern, verbose, basename_mode);
    }
}

fn list_files_in_directory(dir: &str, pattern: &str, verbose: bool, basename_mode: bool) {
    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            let test_path = if basename_mode {
                                file_name
                            } else {
                                file_name
                            };
                            
                            if is_match(test_path, pattern) {
                                println!("{}", file_name);
                            } else if verbose {
                                eprintln!("No match: {}", file_name);
                            }
                        }
                    } else if verbose {
                        if let Some(dir_name) = entry.file_name().to_str() {
                            eprintln!("Skipping directory: {}", dir_name);
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading directory {}: {}", dir, e);
        }
    }
}

fn list_files_recursive(dir: &str, pattern: &str, verbose: bool, basename_mode: bool) {
    if let Err(e) = visit_dir(Path::new(dir), pattern, verbose, basename_mode) {
        eprintln!("Error walking directory tree: {}", e);
    }
}

fn visit_dir(dir: &Path, pattern: &str, verbose: bool, basename_mode: bool) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                visit_dir(&path, pattern, verbose, basename_mode)?;
            } else {
                if let Some(path_str) = path.to_str() {
                    let relative_path = if path_str.starts_with("./") {
                        &path_str[2..]
                    } else {
                        path_str
                    };
                    
                    let test_path = if basename_mode {
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(relative_path)
                    } else {
                        relative_path
                    };
                    
                    if is_match(test_path, pattern) {
                        println!("{}", relative_path);
                    } else if verbose {
                        eprintln!("No match: {}", relative_path);
                    }
                }
            }
        }
    }
    Ok(())
}

fn check_path_match(pattern: &str, path: &str, verbose: bool, basename_mode: bool) {
    let test_path = if basename_mode {
        Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
    } else {
        path
    };
    
    let matches = is_match(test_path, pattern);
    
    if matches {
        println!("{}: MATCH", path);
    } else {
        println!("{}: NO MATCH", path);
    }
    
    if verbose {
        println!("  Pattern: {}", pattern);
        println!("  Path: {}", path);
        println!("  Test path: {}", test_path);
    }
}

fn read_from_stdin(pattern: &str, verbose: bool, basename_mode: bool) {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    
    for line in reader.lines() {
        match line {
            Ok(path) => {
                let path = path.trim();
                if !path.is_empty() {
                    check_path_match(pattern, path, verbose, basename_mode);
                }
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                break;
            }
        }
    }
}