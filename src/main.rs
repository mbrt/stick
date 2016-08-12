extern crate docopt;
extern crate rustc_serialize;

use std::collections::BTreeSet;
use std::env;
use std::error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use docopt::Docopt;

#[derive(Debug, RustcDecodable)]
pub struct Flags {
    flag_list: bool,
    flag_version: bool,
    flag_verbose: u32,
    flag_quiet: Option<bool>,
    arg_command: String,
    arg_args: Vec<String>,
}

const USAGE: &'static str = "
Simple Tickets

Usage:
    stick <command> [args...]
    stick [options]

Options:
    -h, --help          Display this message
    -V, --version       Print version info and exit
    --list              List installed commands
    -v, --verbose ...   Use verbose output
    -q, --quiet         No output printed to stdout

Some common stick commands are (see all commands with --list):
    new         Create a new ticket
    move        Change the state of a ticket
    search      Search in tickets
    list        List tickets
    info        Show information about a ticket

See 'stick help <command>' for more information on a specific command.
";
const MODULE_DIR: &'static str = "/usr/lib/stick";

fn main() {
    let flags: Flags = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if flags.flag_version {
        println!("{}", version());
        return;
    }

    if flags.flag_list {
        println!("Installed Commands:");
        for command in list_commands() {
            println!("    {}", command);
        }
        return;
    }
}

pub fn version() -> String {
    format!("stick {}",
            match option_env!("CFG_VERSION") {
                Some(s) => s.to_string(),
                None => {
                    format!("{}.{}.{}{}",
                            env!("CARGO_PKG_VERSION_MAJOR"),
                            env!("CARGO_PKG_VERSION_MINOR"),
                            env!("CARGO_PKG_VERSION_PATCH"),
                            option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""))
                }
            })
}

/// List all runnable commands. find_command should always succeed
/// if given one of returned command.
fn list_commands() -> BTreeSet<String> {
    let prefix = "stick-";
    let suffix = env::consts::EXE_SUFFIX;
    let mut commands = BTreeSet::new();
    for dir in search_directories() {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            _ => continue,
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let filename = match path.file_name().and_then(|s| s.to_str()) {
                Some(filename) => filename,
                _ => continue,
            };
            if !filename.starts_with(prefix) || !filename.ends_with(suffix) {
                continue;
            }
            if is_executable(entry.path()) {
                let end = filename.len() - suffix.len();
                commands.insert(filename[prefix.len()..end].to_string());
            }
        }
    }

    // TODO add here predef commands
    commands
}

fn execute_subcommand(cmd: &str, args: &[String]) -> io::Result<()> {
    let command_exe = format!("stick-{}{}", cmd, env::consts::EXE_SUFFIX);
    let path = search_directories()
        .iter()
        .map(|dir| dir.join(&command_exe))
        .find(|file| is_executable(file));
    let command = match path {
        Some(command) => command,
        None => {
            return Err(not_found(match find_closest(cmd) {
                    Some(closest) => {
                        format!("no such subcommand: `{}`\n\n\tDid you mean `{}`?\n",
                                cmd,
                                closest)
                    }
                    None => format!("no such subcommand: `{}`", cmd),
                })
                .into())
        }
    };

    // TODO execute command
    return Ok(());
}

#[cfg(unix)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    use std::os::unix::prelude::*;
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
#[cfg(windows)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    fs::metadata(path).map(|metadata| metadata.is_file()).unwrap_or(false)
}

fn search_directories() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from(MODULE_DIR)];
    if let Some(val) = env::var_os("HOME") {
        let mut p = PathBuf::from(&val);
        p.push(".stick");
        p.push("bin");
        dirs.push(p);
    }
    if let Some(val) = env::var_os("PATH") {
        dirs.extend(env::split_paths(&val));
    }
    dirs
}

fn find_closest(cmd: &str) -> Option<String> {
    let cmds = list_commands();
    // Only consider candidates with a lev_distance of 3 or less so we don't
    // suggest out-of-the-blue options.
    let mut filtered = cmds.iter()
        .map(|c| (lev_distance(&c, cmd), c))
        .filter(|&(d, _)| d < 4)
        .collect::<Vec<_>>();
    filtered.sort_by(|a, b| a.0.cmp(&b.0));
    filtered.get(0).map(|slot| slot.1.clone())
}

// taken from cargo prj
pub fn lev_distance(me: &str, t: &str) -> usize {
    use std::cmp;

    if me.is_empty() {
        return t.chars().count();
    }
    if t.is_empty() {
        return me.chars().count();
    }

    let mut dcol = (0..t.len() + 1).collect::<Vec<_>>();
    let mut t_last = 0;

    for (i, sc) in me.chars().enumerate() {

        let mut current = i;
        dcol[0] = current + 1;

        for (j, tc) in t.chars().enumerate() {

            let next = dcol[j + 1];

            if sc == tc {
                dcol[j + 1] = current;
            } else {
                dcol[j + 1] = cmp::min(current, next);
                dcol[j + 1] = cmp::min(dcol[j + 1], dcol[j]) + 1;
            }

            current = next;
            t_last = j;
        }
    }

    dcol[t_last + 1]
}

fn io_err<E>(kind: io::ErrorKind, e: E) -> io::Error
    where E: Into<Box<error::Error + Send + Sync>>
{
    io::Error::new(kind, e)
}

fn not_found<E>(msg: E) -> io::Error
    where E: Into<Box<error::Error + Send + Sync>>
{
    io_err(io::ErrorKind::NotFound, msg)
}
