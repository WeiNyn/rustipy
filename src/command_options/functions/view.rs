use failure::ResultExt;
use crate::module_manager::{ModuleManager, ModuleType};
use crate::module_manager;
use crate::command_options::options::ViewOptions;


pub fn view(options: &ViewOptions) {
    match &options.module {
        Some(module) => {
            let file_path = module_manager::ModuleManager::module_2_path(module, &ModuleType::File)
                .with_context(|e| format!("Failed to convert module to path: {}", e))
                .unwrap();

            let module_type = match file_path.exists() {
                true => ModuleType::File,
                false => ModuleType::Directory,
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

            module_manager.mprint(String::new(), options.code);
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
                    let module = module_manager::ModuleManager::path_2_module(
                        &m.to_str().unwrap().to_string(),
                    )
                    .with_context(|e| format!("Failed to convert path to module: {}", e))
                    .unwrap();

                    let sub_options = ViewOptions {
                        module: Some(module),
                        code: options.code.clone(),
                    };

                    view(&sub_options)
                })
                .collect::<Vec<_>>();
        }
    }
}
