use exitfailure::ExitFailure;
use structopt::StructOpt;

mod command_options;
mod module_manager;
mod parse_ast;
mod python_def;

use crate::command_options::{add, mv, Options, SubCommand};

fn main() -> Result<(), ExitFailure> {
    let options = Options::from_args();

    match options.subcommand {
        SubCommand::Add(add_options) => add(&add_options),
        SubCommand::Move(move_options) => mv(&move_options),
    }

    Ok(())
}
