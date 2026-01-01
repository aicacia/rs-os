use std::io;

use clap::CommandFactory;
use clap_complete::{Shell, generate};

use super::args::CliCommand;

pub fn run(shell: Shell) {
  generate(
    shell,
    &mut CliCommand::command(),
    env!("CARGO_CRATE_NAME"),
    &mut io::stdout(),
  );
}
