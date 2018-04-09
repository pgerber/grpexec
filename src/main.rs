extern crate env_logger;
extern crate grpexec;

use std::borrow::Cow;
use std::os::unix::process::CommandExt;
use std::process::{self, Command};
use std::env;

const DEFAULT_PROGNAME: &str = "grpexec";

fn abort(msg: &str, show_usage: bool) -> ! {
    let program_name = env::args_os().next();
    let program_name = program_name.as_ref().map_or_else(
        || Cow::Borrowed(DEFAULT_PROGNAME),
        |n| n.to_string_lossy(),
    );
    eprintln!("ERROR: {}", msg);
    if show_usage {
        eprintln!("Usage: {} GROUP COMMAND [ARG]...", program_name);
    }
    process::exit(127);
}

fn main() {
    env_logger::init();

    // parse args
    let mut args = env::args_os();
    let group = match args.by_ref().nth(1) {
        Some(c) => c.into_string().expect("group name is not UTF-8 encoded"),
        None => abort("A group must be specified", true),
    };
    let cmd = match args.by_ref().next() {
        Some(c) => c,
        None => abort("A command must be specified", true),
    };

    // set group and drop privileges
    if let Err(e) = grpexec::drop_privileges_with_group(&group) {
        abort(&format!("Failed to change group to {:?}: {}", &group, e), false);
    }

    // execute command
    let err = Command::new(&cmd).args(args).exec();
    abort(&format!("Failed to execute command {:?}: {}", cmd, err), false);
}
