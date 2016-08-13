use std::io::{BufRead, BufReader};
use std::fs::{self, File};

use docopt::Docopt;

use penv;

#[derive(Debug, RustcDecodable)]
pub struct Flags {
    arg_state: Option<String>,
    arg_pattern: String,
}

const USAGE: &'static str = "
Simple Tickets, search in tickets

Usage:
    stick-search [options] <pattern>
    stick-search --help

Options:
    -h, --help          Display this message
    -s, --state STATE   Search in the specified state
";

pub fn execute(args: &[String]) -> i32 {
    let flags: Flags = Docopt::new(USAGE)
        .unwrap()
        .options_first(true)
        .argv(args.into_iter())
        .decode()
        .unwrap_or_else(|e| e.exit());

    let expdir = match flags.arg_state {
        Some(ref state) => penv::state_dir(state),
        None => penv::issues_dir(),
    };
    let dir = match expdir {
        Some(d) => d,
        None => {
            println!("the current directory does not belong to a project");
            return 1;
        }
    };
    let tickets = match fs::read_dir(dir) {
        Ok(t) => t,
        Err(e) => {
            println!("error trying to open the issues directory\n{}", e);
            return 1;
        }
    };

    let mut title_match = Vec::new();
    let mut contents_match = Vec::new();
    for ticket in tickets {
        let ticket = match ticket {
            Ok(de) => de,
            Err(_) => {
                continue;
            }
        };
        let mut f = match File::open(ticket.path()) {
            Ok(f) => BufReader::new(f),
            Err(_) => {
                continue;
            }
        };

        let mut first = true;
        let mut line = String::new();
        while let Ok(len) = f.read_line(&mut line) {
            if len == 0 {
                break;
            }
            if line.contains(&flags.arg_pattern) {
                if let Ok(name) = ticket.file_name().into_string() {
                    if first {
                        title_match.push(name);
                    } else {
                        contents_match.push(name);
                    }
                    break; // don't need to search further
                }
            }
            first = false;
            line.clear();
        }
    }

    if !title_match.is_empty() {
        println!("Matches in title:");
        for issue in title_match {
            println!("  {}", issue);
        }
    }
    if !contents_match.is_empty() {
        println!("\nMatches in contents:");
        for issue in contents_match {
            println!("  {}", issue);
        }
    }

    0
}
