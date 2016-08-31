extern crate docopt;
extern crate regex;
extern crate rustc_serialize;

use std::collections::BTreeSet;
use std::cmp;
use std::env;
use std::error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use docopt::Docopt;

mod penv;
mod ticket;

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
    stick <command> [<args>...]
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

const MODULE_DIR: &'static str = "/usr/lib/stick-modules";

macro_rules! each_subcommand{
    ($mac:ident) => {
        $mac!(search);
    }
}

macro_rules! declare_mod {
    ($name:ident) => ( pub mod $name; )
}
each_subcommand!(declare_mod);


fn main() {
    let ecode = execute_main(env::args(), true);
    process::exit(ecode);
}

fn execute_main<A, T>(args: A, options_first: bool) -> i32
    where A: IntoIterator<Item = T>,
          T: AsRef<str>
{
    let flags: Flags = Docopt::new(USAGE)
        .unwrap()
        .options_first(options_first)
        .argv(args.into_iter())
        .help(true)
        .decode()
        .unwrap_or_else(|e| e.exit());

    if flags.flag_version {
        println!("{}", version());
        return 0;
    }

    if flags.flag_list {
        println!("Installed Commands:");
        for command in list_commands() {
            println!("    {}", command);
        }
        return 0;
    }

    let args = match &flags.arg_command[..] {
        // For the commands `stick` and `stick help`, re-execute ourselves as
        // `stick -h` so we can go through the normal process of printing the
        // help message.
        "" | "help" if flags.arg_args.is_empty() => {
            let args = &["stick".to_string(), "-h".to_string()];
            return execute_main(args, false);
        }

        // For `stick help -h` and `stick help --help`, print out the help
        // message for `stick help`
        "help" if flags.arg_args[0] == "-h" || flags.arg_args[0] == "--help" => {
            let args = &["stick".to_string(), "-h".to_string()];
            return execute_main(args, false);
        }

        // For `stick help foo`, print out the usage message for the specified
        // subcommand by executing the command with the `-h` flag.
        "help" => vec!["stick".to_string(), flags.arg_args[0].clone(), "-h".to_string()],

        // For all other invocations, we're of the form `stick foo args...`. We
        // use the exact environment arguments to preserve tokens like `--` for
        // example.
        _ => env::args().collect(),
    };

    // try to execute the builtin command if present
    if let Some(r) = try_execute_builtin(&args[1..]) {
        return r;
    }

    // search for an exeternal subcommand otherwise
    match execute_subcommand(&args[1], &args[2..]) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            1
        }
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

    macro_rules! add_cmd {
        ($cmd:ident) => ({ commands.insert(stringify!($cmd).replace("_", "-")); })
    }
    each_subcommand!(add_cmd);

    commands
}

fn try_execute_builtin(args: &[String]) -> Option<i32> {
    macro_rules! cmd {
        ($name:ident) => (if args[0] == stringify!($name).replace("_", "-") {
            return Some($name::execute(args));
        })
    }
    each_subcommand!(cmd);

    None
}

fn execute_subcommand(cmd: &str, args: &[String]) -> io::Result<i32> {
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

    process::Command::new(command)
        .args(args)
        .spawn()
        .and_then(|mut c| c.wait())
        .map(|e| e.code().unwrap_or(1))
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
        p.push("modules");
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

// taken from cargo
pub fn lev_distance(me: &str, t: &str) -> usize {
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
