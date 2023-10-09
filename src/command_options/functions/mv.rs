use failure::ResultExt;
use crate::module_manager::{ModuleManager, ModuleType};
use crate::command_options::options::MoveOptions;

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
