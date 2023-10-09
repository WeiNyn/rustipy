use failure::ResultExt;

use crate::module_manager::{ModuleManager, ModuleType};
use crate::command_options::options::AddOptions;


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
