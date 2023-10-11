use exitfailure::ExitFailure;
use structopt::StructOpt;

mod ast;
mod command_options;
mod module_manager;
mod poetry;
mod python_def;

use command_options::functions::{add::add, find::find, mv::mv, view::view, check::check};
use command_options::options::{Options, SubCommand};

fn main() -> Result<(), ExitFailure> {
    let options = Options::from_args();

    match options.subcommand {
        SubCommand::Add(add_options) => add(&add_options),
        SubCommand::Move(move_options) => mv(&move_options),
        SubCommand::Find(find_options) => find(&find_options),
        SubCommand::View(view_options) => view(&view_options),
        SubCommand::Check(_) => check(),
    }

    Ok(())
}
