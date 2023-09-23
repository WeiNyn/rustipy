use structopt::StructOpt;
use failure::ResultExt;

use crate::module_manager::{ModuleManager, ModuleType};

#[derive(StructOpt)]
pub enum SubCommand {
    #[structopt(name = "add", about = "Add a module")]
    Add(AddOptions),

    #[structopt(name = "mv", about = "move a module")]
    Move(MoveOptions),
}

#[derive(StructOpt)]
pub struct AddOptions {
    #[structopt()]
    /// The name of the module to add
    module: String,

    #[structopt(short = "f", long = "file")]
    /// Is the module a file?
    is_file: bool,

    #[structopt(short = "c", long = "contains")]
    /// List of modules that this module contains (files only)
    contains: Option<Vec<String>>,
}

#[derive(StructOpt)]
pub struct MoveOptions {
    #[structopt()]
    /// The name of the module to move
    module: String,

    #[structopt()]
    /// The name of the module to move to
    to: String,
}

#[derive(StructOpt)]
pub struct Options {
    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

pub fn add(options: &AddOptions) {
    if options.is_file && options.contains.is_some() {
        panic!("Files cannot contain other modules: {:?}", options.contains);
    }

    let module = &options.module;
    let module_type = if options.is_file {
        ModuleType::File
    } else {
        ModuleType::Directory
    };

    let mut module_manager = ModuleManager::new(module, module_type, true)
        .with_context(|e| {
            format!(
                "Failed to create module manager for module {}: {}",
                module, e
            )
        })
        .unwrap();

    module_manager
        .build()
        .with_context(|e| {
            format!(
                "Failed to build module manager for module {}: {}",
                module, e
            )
        })
        .unwrap();

    if options.contains.is_some() {
        for sub_module in options.contains.as_ref().unwrap() {
            module_manager
                .add_sub_module(&sub_module, ModuleType::File, true)
                .with_context(|e| format!("Failed to add sub module {}: {}", sub_module, e))
                .unwrap();
        }
    }
}

pub fn mv(options: &MoveOptions) {
    let module = &options.module;
    let to = &options.to;

    let mut module_manager = ModuleManager::new(module, ModuleType::Directory, false)
        .with_context(|e| {
            format!(
                "Failed to create module manager for module {}: {}",
                module, e
            )
        })
        .unwrap();

    module_manager
        .mv(to)
        .with_context(|e| format!("Failed to move module {} to {}: {}", module, to, e))
        .unwrap();
}