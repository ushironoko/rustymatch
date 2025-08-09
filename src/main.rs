//! # Satch CLI
//!
//! glob pattern matching command-line tool.
//!
//! A Rust port of [micromatch](https://github.com/micromatch/micromatch) and
//! [picomatch](https://github.com/micromatch/picomatch) with a convenient CLI interface.
//!
//! ## Usage
//!
//! ```bash
//! # Test paths from stdin
//! echo "src/main.rs" | satch --basename "*.rs"   # MATCH
//!
//! # List matching files
//! satch --list --recursive --basename "*.js"     # Find all .js files
//!
//! # Test specific paths
//! satch "*.rs" src/main.rs lib.rs test.js        # Test multiple files
//! ```
//!
//! ## Features
//!
//! - **Pattern matching**: Test file paths against glob patterns
//! - **File listing**: Find files matching patterns in directories
//! - **Recursive search**: Search through directory trees
//! - **Basename matching**: Match against filename only, ignoring path
//! - **Multiple modes**: stdin input, file listing, or direct path testing
//!
//! ## Pattern Support
//!
//! - Basic wildcards: `*`, `?`
//! - Globstars: `**` for recursive directory matching
//! - Character classes: `[abc]`, `[a-z]`, `[^abc]`
//! - Complex patterns: `**/test/**/*.js`

use clap::{Arg, Command};
use satch::is_match;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// Main entry point for the satch CLI tool.
///
/// Parses command-line arguments and dispatches to appropriate functionality:
/// - List mode: Find and list files matching the pattern
/// - Path testing: Test specific paths against the pattern  
/// - Stdin mode: Read paths from stdin and test each one
fn main() {
    let matches = Command::new("satch")
        .version("0.1.0")
        .author("ushironoko")
        .about("glob pattern matching CLI tool")
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

/// Lists files matching the given pattern.
///
/// # Arguments
/// * `pattern` - Glob pattern to match against
/// * `recursive` - If true, search recursively through directories
/// * `verbose` - If true, show verbose output including non-matches
/// * `basename_mode` - If true, match against filename only (ignore directory path)
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

/// Tests a single path against the given pattern and prints the result.
///
/// # Arguments
/// * `pattern` - Glob pattern to match against
/// * `path` - File path to test
/// * `verbose` - If true, show detailed matching information
/// * `basename_mode` - If true, match against filename only (ignore directory path)
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

/// Reads file paths from stdin and tests each one against the pattern.
///
/// # Arguments
/// * `pattern` - Glob pattern to match against
/// * `verbose` - If true, show detailed matching information
/// * `basename_mode` - If true, match against filename only (ignore directory path)
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