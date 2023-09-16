use failure::{Error, ResultExt};
use fs_extra::dir::{move_dir, CopyOptions};
use regex::Regex;
use std::fs::{create_dir_all, rename, File};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

pub enum ModuleType {
    File,
    Directory,
}

impl PartialEq for ModuleType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ModuleType::File, ModuleType::File) => true,
            (ModuleType::Directory, ModuleType::Directory) => true,
            _ => false,
        }
    }
}

pub struct ModuleManager {
    path: PathBuf,
    module: String,
    classes: Vec<String>,
    functions: Vec<String>,
    vars: Vec<String>,
    module_type: ModuleType,
}

impl ModuleManager {
    /// Creates a new ModuleManager from a module and a module type.
    /// Path should in format "path.to.module".
    /// Path can be a file or a directory.
    /// If path is a directory, it will search for __init__.py file.
    /// If path is a file, it will search for a file with the same name but with .py extension.
    pub fn new(module: &str, module_type: ModuleType) -> Result<Self, Error> {
        let mut path = String::new();

        for component in module.split(".") {
            path.push_str(component);
            path.push_str("/");
        }

        if module_type == ModuleType::Directory {
            path.push_str("__init__.py");
        } else {
            path.pop();
            path.push_str(".py");
        }

        let mut module_manager = Self {
            path: PathBuf::from(path),
            module: module.to_owned(),
            classes: Vec::new(),
            functions: Vec::new(),
            vars: Vec::new(),
            module_type,
        };

        Ok(module_manager)
    }

    fn make_tree(path: &Path) -> Result<(), Error> {
        if !path.parent().is_none() {
            create_dir_all(path.parent().unwrap()).with_context(|e| {
                format!("Could not create directory {}: {}", path.display(), e)
            })?;
        }

        if !path.ends_with(".py") {
            File::create(path)
                .with_context(|e| format!("Could not create file {}: {}", path.display(), e))?;
        }

        Ok(())
    }

    fn path_2_module(path: &str) -> Result<String, Error> {
        let path = PathBuf::from(path);
        let mut module = String::new();

        for component in path.components() {
            let component = component.as_os_str().to_str().unwrap();

            if component == "__init__.py" {
                module.pop();
                break;
            }

            if component.ends_with(".py") {
                module.push_str(&component[..component.len() - 3]);
                break;
            }

            module.push_str(component);
            module.push_str(".");
        }

        Ok(module)
    }

    fn module_2_path(module: &str, module_type: &ModuleType) -> Result<PathBuf, Error> {
        let mut path = String::new();

        for component in module.split(".") {
            path.push_str(component);
            path.push_str("/");
        }

        if *module_type == ModuleType::Directory {
            path.push_str("__init__.py");
        } else {
            path.pop();
            path.push_str(".py");
        }

        Ok(PathBuf::from(path))
    }

    fn read_file(path: &Path) -> Result<String, Error> {
        let mut file = File::open(path)
            .with_context(|e| format!("Could not open file {}: {}", path.display(), e))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|e| format!("Could not read file {}: {}", path.display(), e))?;

        Ok(contents)
    }

    fn find_classes(self: &Self) -> Result<Vec<String>, Error> {
        let mut classes = Vec::new();
        let contents = Self::read_file(&self.path)?;

        let re = Regex::new(r"(?m)^class\s+(\w+)\s*:\s*$")?;
        for cap in re.captures_iter(&contents) {
            classes.push(cap[1].replace("class", "").trim().to_owned());
        }
        Ok(classes)
    }

    fn find_functions(self: &Self) -> Result<Vec<String>, Error> {
        let mut functions = Vec::new();
        let contents = Self::read_file(&self.path)?;

        let re = Regex::new(r"(?m)^def\s+(\w+)\s*\(.*\)\s*:\s*$")?;
        for cap in re.captures_iter(&contents) {
            functions.push(cap[1].replace("def", "").trim().to_owned());
        }
        Ok(functions)
    }

    fn find_vars(self: &Self) -> Result<Vec<String>, Error> {
        let mut vars = Vec::new();
        let contents = Self::read_file(&self.path)?;

        let re = Regex::new(r"(?m)^\s*(\w+)\s*(?::\s*\w+\s*)?=\s*.*$")?;
        contents
            .lines()
            .filter(|l| !l.starts_with(" ") & !l.starts_with("  "))
            .for_each(|l| {
                for cap in re.captures_iter(l) {
                    vars.push(cap[1].trim().to_owned());
                }
            });

        Ok(vars)
    }

    pub fn build(self: &Self) -> Result<(), Error> {
        Self::make_tree(&self.path)?;
        Ok(())
    }

    pub fn reload(self: &mut Self) -> Result<(), Error> {
        self.classes = self.find_classes()?;
        self.functions = self.find_functions()?;
        self.vars = self.find_vars()?;
        Ok(())
    }

    pub fn mv(self: &mut Self, to: &str) -> Result<(), Error> {
        let mut new_path = Self::module_2_path(to, &self.module_type)?;
        Self::make_tree(&new_path)?;

        if self.module_type == ModuleType::Directory {
            println!("Moving {} to {}", self.path.display(), new_path.display());

            move_dir(
                &self.path.parent().unwrap(),
                &new_path.parent().unwrap(),
                &CopyOptions::default().content_only(true).overwrite(true),
            )
            .with_context(|e| format!("Could not move directory {}: {}", self.path.display(), e))?;
        } else {
            println!("Renaming {} to {}", self.path.display(), new_path.display());

            rename(&self.path, &new_path).with_context(|e| {
                format!("Could not rename file {}: {}", self.path.display(), e)
            })?;
        }

        self.path = new_path;
        self.module = to.to_owned();
        self.reload()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;
    #[test]
    fn test_create() {
        let module_manager = ModuleManager::new("tests.test_module", ModuleType::File).unwrap();
        assert_eq!(module_manager.module, "tests.test_module");
        assert_eq!(module_manager.path, PathBuf::from("tests/test_module.py"));

        let module_manager = ModuleManager::new("tests", ModuleType::Directory).unwrap();
        assert_eq!(module_manager.module, "tests");
        assert_eq!(module_manager.path, PathBuf::from("tests/__init__.py"));
    }

    #[test]
    fn test_find_classes() {
        let module_manager = ModuleManager::new("tests.test_module", ModuleType::File).unwrap();
        let classes = module_manager.find_classes().unwrap();
        assert_eq!(classes, vec!["TestClass"]);
    }

    #[test]
    fn test_find_functions() {
        let module_manager = ModuleManager::new("tests.test_module", ModuleType::File).unwrap();
        let functions = module_manager.find_functions().unwrap();
        assert_eq!(functions, vec!["test_function"]);
    }

    #[test]
    fn test_find_vars() {
        let module_manager = ModuleManager::new("tests.test_module", ModuleType::File).unwrap();
        let vars = module_manager.find_vars().unwrap();
        assert_eq!(vars, vec!["test_var", "test_var2", "TEST_CONST"]);
    }

    #[test]
    fn test_make_tree() {
        let path = PathBuf::from("tests/test_module.py");
        ModuleManager::make_tree(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_path_2_module() {
        let path = "tests/test_module.py";
        let module = ModuleManager::path_2_module(path).unwrap();
        assert_eq!(module, "tests.test_module");

        let path = "tests/test_module/__init__.py";
        let module = ModuleManager::path_2_module(path).unwrap();
        assert_eq!(module, "tests.test_module");
    }

    #[test]
    fn test_module_2_path() {
        let module = "tests.test_module";
        let path = ModuleManager::module_2_path(module, &ModuleType::File).unwrap();
        assert_eq!(path, PathBuf::from("tests/test_module.py"));

        let module = "tests.test_module";
        let path = ModuleManager::module_2_path(module, &ModuleType::Directory).unwrap();
        assert_eq!(path, PathBuf::from("tests/test_module/__init__.py"));
    }

    #[test]
    fn test_mv() {
        let mut module_manager = ModuleManager::new("tests.test_mv", ModuleType::File).unwrap();
        module_manager.mv("tests.test_mv2").unwrap();
        assert_eq!(module_manager.module, "tests.test_mv2");
        assert_eq!(module_manager.path, PathBuf::from("tests/test_mv2.py"));

        module_manager.mv("tests.test_mv").unwrap();
        assert_eq!(module_manager.module, "tests.test_mv");
        assert_eq!(module_manager.path, PathBuf::from("tests/test_mv.py"));

        let mut module_manager =
            ModuleManager::new("tests.test_mv", ModuleType::Directory).unwrap();
        module_manager.mv("tests.test_mv2").unwrap();
        assert_eq!(module_manager.module, "tests.test_mv2");
        assert_eq!(
            module_manager.path,
            PathBuf::from("tests/test_mv2/__init__.py")
        );

        module_manager.mv("tests.test_mv").unwrap();
        assert_eq!(module_manager.module, "tests.test_mv");
        assert_eq!(
            module_manager.path,
            PathBuf::from("tests/test_mv/__init__.py")
        );
    }

    #[test]
    fn test_build() {
        let module_manager = ModuleManager::new("tests.test_build", ModuleType::File).unwrap();
        module_manager.build().unwrap();
        assert!(module_manager.path.exists());
        remove_file(module_manager.path).unwrap();

        let module_manager = ModuleManager::new("tests.test_build", ModuleType::Directory).unwrap();
        module_manager.build().unwrap();
        assert!(module_manager.path.exists());
        remove_file(module_manager.path).unwrap();
    }
}
