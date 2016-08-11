extern crate docopt;
extern crate rustc_serialize;

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

fn main() {
    let args: Flags = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
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
