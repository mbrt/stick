use std::io::{BufRead, BufReader};
use std::fs::{self, File};
use std::path::Path;

use docopt::Docopt;

use penv;
use ticket;

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

    let env = match penv::Environment::new() {
        Some(e) => e,
        None => {
            println!("the current directory does not belong to a project");
            return 1;
        }
    };

    let dir = match flags.arg_state {
        Some(ref state) => env.state_dir(state),
        None => env.issues_dir(),
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
    let title_parser = ticket::TitleParser::new();
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
                let fname = ticket.file_name();
                let fpath: &Path = fname.as_ref();
                let name = match fpath.file_stem() {
                    None => None,
                    Some(name) => name.to_os_string().into_string().ok(),
                };
                if let Some(name) = name {
                    if first {
                        let title = title_parser.parse(&name, &line);
                        title_match.push((name, title));
                    } else {
                        contents_match.push((name, line));
                    }
                    break; // don't need to search further
                }
            }
            first = false;
            line.clear();
        }
    }

    let mut need_spacer = false;
    if !title_match.is_empty() {
        println!("Matches in title:");
        for issue in title_match {
            println!("  {}: {}", issue.0, issue.1.descr);
        }
        need_spacer = true;
    }
    if !contents_match.is_empty() {
        if need_spacer {
            println!("");
        }
        println!("Matches in contents:");
        for issue in contents_match {
            println!("  {}: {}", issue.0, issue.1);
        }
    }

    0
}
