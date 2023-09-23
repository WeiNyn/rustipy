use failure::{Error, ResultExt};
use fs_extra::dir::{move_dir, CopyOptions};
use log::{debug, info};
use regex::Regex;
use std::fs::{create_dir_all, rename, File};
use std::io::ErrorKind;
use std::{
    io::Read,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug)]
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
    sub_modules: Vec<ModuleManager>,
}

impl ModuleManager {
    /// Creates a new ModuleManager from a module and a module type.
    /// Path should in format "path.to.module".
    /// Path can be a file or a directory.
    /// If path is a directory, it will search for __init__.py file.
    /// If path is a file, it will search for a file with the same name but with .py extension.
    /// #Example
    /// ```
    /// use module_manager::{ModuleManager, ModuleType};
    ///
    /// let module_manager = ModuleManager::new("path.to.module", ModuleType::File, true).unwrap();
    /// ```
    /// #Errors
    /// Returns an error if the module manager could not be created.
    /// #Panics
    /// Panics if the module type is ModuleType::File and the module contains other modules.
    /// #Notes
    /// If build is true, it will create the module and reload it.
    pub fn new(module: &str, module_type: ModuleType, build: bool) -> Result<Self, Error> {
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
            module_type: module_type,
            sub_modules: Vec::new(),
        };

        if build {
            module_manager.build()?;
            module_manager.reload()?;
        }

        Ok(module_manager)
    }

    fn travel_root() -> Result<impl Iterator<Item = PathBuf>, Error> {
        let path = std::env::current_dir()?;

        let iter = WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter(|e| {
                let e = e.as_ref().unwrap();
                e.file_type().is_file()
                    && match e.path().extension() {
                        Some(extension) => extension == "py",
                        None => false,
                    }
            })
            .map(|e| e.unwrap().into_path());

        Ok(iter)
    }

    fn replace_in_root(old: &str, new: &str) -> Result<(), Error> {
        let files_iter = Self::travel_root()
            .with_context(|e| format!("Could not travel root directory: {}", e))?;

        for file in files_iter {
            debug!("Replacing in {}", file.display());
            let contents = Self::read_file(&file)
                .with_context(|e| format!("Could not read file {}: {}", file.display(), e))?;

            let pattern = format!(
                r"(?m)(\s+|=|:|\(|\[|\{{)({})(\s+|\.)",
                old.replace(".", r"\.")
            );

            let new_contents = Regex::new(&pattern)
                .with_context(|e| format!("Could not create regex {}: {}", pattern, e))?
                .replace_all(&contents, |caps: &regex::Captures| {
                    let mut replacement = String::new();
                    replacement.push_str(&caps[1]);
                    replacement.push_str(&new);
                    replacement.push_str(&caps[3]);
                    replacement
                })
                .to_string();

            std::fs::write(&file, new_contents)
                .with_context(|e| format!("Could not write to file {}: {}", file.display(), e))?;
        }

        Ok(())
    }

    fn make_tree(path: &Path) -> Result<(), Error> {
        if path.exists() {
            info!("{} already exists", path.display());
            return Ok(());
        }

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
                return Ok(module);
            }

            if component.ends_with(".py") {
                module.push_str(&component[..component.len() - 3]);
                return Ok(module);
            }

            module.push_str(component);
            module.push_str(".");
        }

        return Result::Err(Error::from(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Invalid path",
        )));
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

        let re = Regex::new(r"(?m)^class\s+(\w+)\s*(\(\s*(\w|\.)+\s*\))?\s*:\s*$")?;
        for cap in re.captures_iter(&contents) {
            classes.push(cap[1].replace("class", "").trim().to_owned());
        }
        Ok(classes)
    }

    fn find_functions(self: &Self) -> Result<Vec<String>, Error> {
        let mut functions = Vec::new();
        let contents = Self::read_file(&self.path)?;

        let re = Regex::new(
            r"(?m)^def\s+(\w+)\s*\((?:\s*\w+\s*:\s*(\w|\.)+\s*(?:=\s*.+)?\s*,?\s*)*\)\s*(?:->\s*(\w|\.)+\s*)?:\s*$",
        )?;
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

    fn get_sub_modules(self: &mut Self) -> Result<Vec<ModuleManager>, Error> {
        let mut sub_modules = Vec::new();

        let files_iter = Self::travel_root()
            .with_context(|e| format!("Could not travel root directory: {}", e))?;

        for file in files_iter {
            match Self::path_2_module(file.to_str().unwrap()) {
                Ok(module) => {
                    if module.starts_with(&self.module) && module != self.module {
                        let mut sub_module_manager = Self::new(&module, ModuleType::File, false)?;
                        sub_module_manager.reload()?;
                        sub_modules.push(sub_module_manager);
                    }
                }
                Err(_) => {}
            }
        }

        Ok(sub_modules)
    }

    pub fn build(self: &Self) -> Result<(), Error> {
        Self::make_tree(&self.path)
            .with_context(|e| format!("Could not make tree for {}: {}", self.path.display(), e))?;
        Ok(())
    }

    pub fn reload(self: &mut Self) -> Result<(), Error> {
        self.classes = self.find_classes()?;
        self.functions = self.find_functions()?;
        self.vars = self.find_vars()?;
        self.sub_modules = self.get_sub_modules()?;
        Ok(())
    }

    pub fn mv(self: &mut Self, to: &str) -> Result<(), Error> {
        let new_path = Self::module_2_path(to, &self.module_type)?;
        Self::make_tree(&new_path)?;

        if self.module_type == ModuleType::Directory {
            debug!("Moving {} to {}", self.path.display(), new_path.display());

            move_dir(
                &self.path.parent().unwrap(),
                &new_path.parent().unwrap(),
                &CopyOptions::default().content_only(true).overwrite(true),
            )
            .with_context(|e| format!("Could not move directory {}: {}", self.path.display(), e))?;
        } else {
            debug!("Renaming {} to {}", self.path.display(), new_path.display());

            rename(&self.path, &new_path).with_context(|e| {
                format!("Could not rename file {}: {}", self.path.display(), e)
            })?;
        }

        Self::replace_in_root(&self.module, to)
            .with_context(|e| format!("Could not replace in root directory: {}", e))?;

        self.path = new_path;
        self.module = to.to_owned();
        self.reload()?;
        Ok(())
    }

    pub fn add_sub_module(
        self: &mut Self,
        sub_module: &str,
        module_type: ModuleType,
        build: bool,
    ) -> Result<(), Error> {
        if self.module_type == ModuleType::File {
            return Result::Err(Error::from(std::io::Error::new(
                ErrorKind::Unsupported,
                "Files cannot contain other modules",
            )));
        }

        let sub_module_manager = Self::new(
            format!("{}.{}", &self.module, sub_module).as_str(),
            module_type,
            build,
        )?;

        self.sub_modules.push(sub_module_manager);
        Ok(())
    }

    pub fn get_classes(self: &Self) -> Vec<String> {
        let mut classes = self
            .classes
            .clone()
            .into_iter()
            .map(|c| format!("{}.{}", &self.module, c))
            .collect::<Vec<String>>();

        for sub_module in &self.sub_modules {
            classes.append(&mut sub_module.get_classes());
        }

        classes
    }

    pub fn get_functions(self: &Self) -> Vec<String> {
        let mut functions = self
            .functions
            .clone()
            .into_iter()
            .map(|f| format!("{}.{}", &self.module, f))
            .collect::<Vec<String>>();

        for sub_module in &self.sub_modules {
            functions.append(&mut sub_module.get_functions());
        }

        functions
    }

    pub fn get_vars(self: &Self) -> Vec<String> {
        let mut vars = self
            .vars
            .clone()
            .into_iter()
            .map(|v| format!("{}.{}", &self.module, v))
            .collect::<Vec<String>>();

        for sub_module in &self.sub_modules {
            vars.append(&mut sub_module.get_vars());
        }

        vars
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{remove_dir_all, remove_file};

    use super::*;

    #[test]
    fn test_create() {
        let module_manager =
            ModuleManager::new("tests.test_create", ModuleType::File, false).unwrap();
        assert_eq!(module_manager.module, "tests.test_create");
        assert_eq!(module_manager.path, PathBuf::from("tests/test_create.py"));

        let module_manager =
            ModuleManager::new("tests.test_create", ModuleType::Directory, false).unwrap();
        assert_eq!(module_manager.module, "tests.test_create");
        assert_eq!(
            module_manager.path,
            PathBuf::from("tests/test_create/__init__.py")
        );
    }

    #[test]
    fn test_find_classes() {
        let module_manager =
            ModuleManager::new("tests.test_module", ModuleType::File, false).unwrap();
        let classes = module_manager.find_classes().unwrap();
        assert_eq!(classes, vec!["TestClass", "TestClass2"]);
    }

    #[test]
    fn test_find_functions() {
        let module_manager =
            ModuleManager::new("tests.test_module", ModuleType::File, false).unwrap();
        let functions = module_manager.find_functions().unwrap();
        assert_eq!(functions, vec!["test_function"]);
    }

    #[test]
    fn test_find_vars() {
        let module_manager =
            ModuleManager::new("tests.test_module", ModuleType::File, false).unwrap();
        let vars = module_manager.find_vars().unwrap();
        assert_eq!(vars, vec!["test_var", "test_var2", "TEST_CONST"]);
    }

    #[test]
    fn test_build() {
        let module_manager =
            ModuleManager::new("tests.test_build", ModuleType::File, false).unwrap();
        module_manager.build().unwrap();
        assert!(module_manager.path.exists());
        remove_file("tests/test_build.py").unwrap();

        let module_manager =
            ModuleManager::new("tests.test_build", ModuleType::Directory, false).unwrap();
        module_manager.build().unwrap();
        assert!(module_manager.path.exists());
        remove_dir_all("tests/test_build").unwrap();
    }

    #[test]
    fn test_add_sub_module() {
        let mut module_manager =
            ModuleManager::new("tests.test_add_sub_module", ModuleType::Directory, true).unwrap();
        module_manager
            .add_sub_module("sub_module", ModuleType::File, true)
            .unwrap();

        assert_eq!(module_manager.sub_modules.len(), 1);
        assert_eq!(
            module_manager.sub_modules[0].module,
            "tests.test_add_sub_module.sub_module"
        );
        assert_eq!(
            module_manager.sub_modules[0].path,
            PathBuf::from("tests/test_add_sub_module/sub_module.py")
        );
        assert_eq!(module_manager.sub_modules[0].module_type, ModuleType::File);

        remove_dir_all("tests/test_add_sub_module").unwrap();
    }

    #[test]
    fn test_add_sub_module_panic() {
        let mut module_manager =
            ModuleManager::new("tests.test_add_sub_module", ModuleType::File, true).unwrap();
        match module_manager.add_sub_module("sub_module", ModuleType::Directory, true) {
            Ok(_) => {
                panic!("Should panic")
            }
            Err(e) => {
                assert_eq!(e.to_string(), "Files cannot contain other modules");
                remove_file("tests/test_add_sub_module.py").unwrap();
            }
        };
    }

    #[test]
    #[ignore]
    fn test_mv() {
        let mut module_manager =
            ModuleManager::new("tests.test_mv", ModuleType::File, true).unwrap();
        module_manager.mv("tests.test_mv2").unwrap();
        assert_eq!(module_manager.module, "tests.test_mv2");
        assert_eq!(module_manager.path, PathBuf::from("tests/test_mv2.py"));

        let check_content = ModuleManager::read_file(Path::new("tests/test_check_mv.py"))
            .expect("Could not read file");
        assert_eq!(check_content, "from tests.test_mv2 import *\nimport tests.test_mv2.abc as abc\ntest_var:tests.test_mv2.abc.ABC = tests.test_mv2.abc.ABC()");

        module_manager.mv("tests.test_mv").unwrap();
        assert_eq!(module_manager.module, "tests.test_mv");
        assert_eq!(module_manager.path, PathBuf::from("tests/test_mv.py"));

        let check_content = ModuleManager::read_file(Path::new("tests/test_check_mv.py"))
            .expect("Could not read file");
        assert_eq!(check_content, "from tests.test_mv import *\nimport tests.test_mv.abc as abc\ntest_var:tests.test_mv.abc.ABC = tests.test_mv.abc.ABC()");

        let mut module_manager =
            ModuleManager::new("tests.test_mv", ModuleType::Directory, true).unwrap();
        module_manager.mv("tests.test_mv2").unwrap();
        assert_eq!(module_manager.module, "tests.test_mv2");
        assert_eq!(
            module_manager.path,
            PathBuf::from("tests/test_mv2/__init__.py")
        );

        let check_content = ModuleManager::read_file(Path::new("tests/test_check_mv.py"))
            .expect("Could not read file");
        assert_eq!(check_content, "from tests.test_mv2 import *\nimport tests.test_mv2.abc as abc\ntest_var:tests.test_mv2.abc.ABC = tests.test_mv2.abc.ABC()");

        module_manager.mv("tests.test_mv").unwrap();
        assert_eq!(module_manager.module, "tests.test_mv");
        assert_eq!(
            module_manager.path,
            PathBuf::from("tests/test_mv/__init__.py")
        );

        let check_content = ModuleManager::read_file(Path::new("tests/test_check_mv.py"))
            .expect("Could not read file");
        assert_eq!(check_content, "from tests.test_mv import *\nimport tests.test_mv.abc as abc\ntest_var:tests.test_mv.abc.ABC = tests.test_mv.abc.ABC()");

        remove_file("tests/test_mv.py").unwrap();
        remove_dir_all("tests/test_mv").unwrap();
    }

    #[test]
    fn test_get_classes() {
        let mut module_manager = ModuleManager::new("tests", ModuleType::Directory, false).unwrap();
        module_manager.reload().unwrap();
        let classes = module_manager.get_classes();
        assert_eq!(
            classes,
            vec![
                "tests.test_module.TestClass",
                "tests.test_module.TestClass2"
            ]
        );
    }

    #[test]
    fn test_get_functions() {
        let mut module_manager = ModuleManager::new("tests", ModuleType::Directory, false).unwrap();
        module_manager.reload().unwrap();
        let functions = module_manager.get_functions();
        assert_eq!(functions, vec!["tests.test_module.test_function"]);
    }

    #[test]
    fn test_get_vars() {
        let mut module_manager = ModuleManager::new("tests", ModuleType::Directory, false).unwrap();
        module_manager.reload().unwrap();
        let vars = module_manager.get_vars();
        assert_eq!(
            vars,
            vec![
                "tests.test_module.test_var",
                "tests.test_module.test_var2",
                "tests.test_module.TEST_CONST",
                "tests.test_check_mv.test_var"
            ]
        );
    }
}
