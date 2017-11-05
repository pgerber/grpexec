extern crate env_logger;
extern crate grpexec;

use std::os::unix::process::CommandExt;
use std::process::{self, Command};
use std::env;

const DEFAULT_PROGNAME: &str = "grpexec";

fn abort(msg: String) -> ! {
    let program_name = env::args_os().next().map_or_else(
        || DEFAULT_PROGNAME.to_string(),
        |n| n.to_string_lossy().to_string(),
    );
    eprintln!("ERROR: {}", msg);
    eprintln!("Usage: {} GROUP COMMAND [ARG]...", program_name);
    process::exit(127);
}

fn main() {
    env_logger::init().expect("failed to initialize logger");

    // parse args
    let mut args = env::args_os();
    let group = match args.by_ref().skip(1).next() {
        Some(c) => c.into_string().expect("group name is not UTF-8 encoded"),
        None => abort("A group must be specified".to_string()),
    };
    let cmd = match args.by_ref().next() {
        Some(c) => c,
        None => abort("A command must be specified".to_string()),
    };

    // set group and drop privileges
    if let Err(e) = grpexec::drop_privileges_with_group(&group) {
        abort(format!("Failed to change group to {:?}: {}", &group, e));
    }

    // execute command
    let err = Command::new(&cmd).args(args).exec();
    abort(format!("Failed to execute command {:?}: {}", cmd, err));
}
