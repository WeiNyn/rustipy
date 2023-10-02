use color_print::cprintln;
use failure::ResultExt;
use structopt::StructOpt;

use crate::module_manager::{self, ModuleManager, ModuleType};

#[derive(StructOpt)]
pub enum SubCommand {
    #[structopt(name = "add", about = "Add a module")]
    Add(AddOptions),

    #[structopt(name = "mv", about = "move a module")]
    Move(MoveOptions),

    #[structopt(name = "find", about = "find a module")]
    Find(FindOptions),
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
pub struct FindOptions {
    #[structopt()]
    /// The name of the module to find
    query: String,

    #[structopt()]
    /// The name of the module to find
    module: Option<String>,

    #[structopt(short = "i", long = "is_file")]
    /// Is the module a file?
    is_file: bool,

    #[structopt(short = "f", long = "function")]
    /// find functions
    function: bool,

    #[structopt(short = "c", long = "class")]
    /// find classes
    class: bool,

    #[structopt(short = "v", long = "variable")]
    /// find variables
    variable: bool,
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

pub fn find(options: &FindOptions) {
    match &options.module {
        Some(module) => {
            let query = &options.query;

            cprintln!(
                "󱁴 Searching for <green>{}</green> in <blue>{}</blue>",
                query,
                module
            );

            let module_type = if options.is_file {
                ModuleType::File
            } else {
                ModuleType::Directory
            };

            let mut module_manager = ModuleManager::new(module, module_type, false)
                .with_context(|e| {
                    format!(
                        "Failed to create module manager for module {}: {}",
                        module, e
                    )
                })
                .unwrap();

            module_manager
                .reload()
                .with_context(|e| {
                    format!(
                        "Failed to reload module manager for module {}: {}",
                        module, e
                    )
                })
                .unwrap();

            let mut result = Vec::new();
            let is_find_all = !options.function && !options.class && !options.variable;

            if options.function || is_find_all {
                result.extend(module_manager.find_functions(query))
            }

            if options.class || options.function || is_find_all {
                result.extend(module_manager.find_classes(query))
            }

            if options.variable || is_find_all {
                result.extend(module_manager.find_vars(query))
            }

            for r in result {
                cprintln!("{}", r);
            }
        }
        None => {
            let _ = module_manager::ModuleManager::travel_root()
                .unwrap()
                .filter(|m| {
                    if m.file_name().unwrap() == "__init__.py" {
                        if m.iter().count() != 3 {
                            return false;
                        } else {
                            return true;
                        }
                    } else if m.iter().count() != 2 {
                        return false;
                    } else {
                        return true;
                    }
                })
                .map(|m| {
                    let is_file = m.file_name().unwrap() != "__init__.py";
                    let module = module_manager::ModuleManager::path_2_module(
                        &m.to_str().unwrap().to_string(),
                    )
                    .with_context(|e| format!("Failed to convert path to module: {}", e))
                    .unwrap();

                    let sub_options = FindOptions {
                        query: options.query.clone(),
                        module: Some(module),
                        is_file: is_file,
                        function: options.function.clone(),
                        class: options.class.clone(),
                        variable: options.variable.clone(),
                    };

                    find(&sub_options)
                })
                .collect::<Vec<_>>();
        }
    }
}
