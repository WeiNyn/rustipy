use exitfailure::ExitFailure;
use structopt::StructOpt;

mod module_manager;
mod command_options;

use crate::command_options::{Options, SubCommand, add, mv};


fn main() -> Result<(), ExitFailure> {
    let options = Options::from_args();

    match options.subcommand {
        SubCommand::Add(add_options) => add(&add_options),
        SubCommand::Move(move_options) => mv(&move_options),
    }

    Ok(())
}
