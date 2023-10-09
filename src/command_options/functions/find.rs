use failure::ResultExt;
use color_print::{cprintln, cprint};
use crate::module_manager::{ModuleManager, ModuleType};
use crate::module_manager;
use crate::command_options::options::FindOptions;


pub fn find(options: &FindOptions) {
    match &options.module {
        Some(module) => {
            let query = &options.query;

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

            let (find_vars, find_functions, find_classes) =
                match !options.function && !options.class && !options.variable {
                    true => (true, true, true),
                    false => (options.variable, options.function, options.class),
                };

            let displays = module_manager
                .find(
                    query,
                    String::new(),
                    find_vars,
                    find_functions,
                    find_classes,
                )
                .with_context(|e| format!("Failed to find module {}: {}", module, e))
                .unwrap();

            if displays.len() > 0 {
                cprintln!(
                    "<Y><s>󱁴 Searching for <blink>[{}]</blink> in <B>{}</B></s></Y>",
                    query,
                    module
                );
            }

            for display in displays {
                cprint!("{}", display)
            }
        }
        None => {
            let _ = module_manager::ModuleManager::travel_root(None, Some(2))
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
