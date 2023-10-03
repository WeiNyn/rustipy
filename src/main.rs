use exitfailure::ExitFailure;
use structopt::StructOpt;

mod command_options;
mod module_manager;
mod parse_ast;
mod python_def;

use crate::command_options::{add, find, mv, view, Options, SubCommand};

fn main() -> Result<(), ExitFailure> {
    let options = Options::from_args();

    match options.subcommand {
        SubCommand::Add(add_options) => add(&add_options),
        SubCommand::Move(move_options) => mv(&move_options),
        SubCommand::Find(find_options) => find(&find_options),
        SubCommand::View(view_options) => view(&view_options),
    }

    Ok(())
}
