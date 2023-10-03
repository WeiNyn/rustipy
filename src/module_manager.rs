use crate::parse_ast::{parse_ast, parse_root_ast};
use crate::python_def::{Attribute, Class, Method, PythonDef};
use color_print::cformat;
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct ModuleManager {
    path: PathBuf,
    module: String,
    classes: Vec<Class>,
    functions: Vec<Method>,
    vars: Vec<Attribute>,
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

    pub fn travel_root(
        prefix: Option<String>,
        max_dept: Option<usize>,
    ) -> Result<impl Iterator<Item = PathBuf>, Error> {
        let prefix = match prefix {
            Some(prefix) => "./".to_owned() + &prefix,
            None => String::from("./"),
        };

        let mut iter = WalkDir::new(prefix).follow_links(true);

        if max_dept.is_some() {
            iter = iter.max_depth(max_dept.unwrap());
        }

        let iter = iter
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
        let files_iter = Self::travel_root(None, None)
            .with_context(|e| format!("Could not travel root directory: {}", e))?;

        for file in files_iter {
            debug!("Replacing in {}", file.display());
            let mut contents = Self::read_file(&file)
                .with_context(|e| format!("Could not read file {}: {}", file.display(), e))?;

            // Handle normal import: import old -> new
            let pattern = Regex::new(&format!(r"import\s+{}((\.((\w|_)+(\d|\w|_)*))+|\s+)", old))
                .with_context(|e| format!("Could not create regex: {}", e))?;

            contents = pattern
                .replace_all(&contents, |caps: &regex::Captures| {
                    let mut replacement = String::from("import ");
                    replacement.push_str(new);

                    let after = caps.get(1);
                    match after {
                        Some(after) => {
                            replacement.push_str(after.as_str());
                        }
                        None => {}
                    }

                    replacement
                })
                .to_string();

            // Handle from import: from old import -> from new import
            let pattern = Regex::new(&format!(r"from\s+{}(\.((\w|_)+(\d|\w|_)*))*\s+import", old))
                .with_context(|e| format!("Could not create regex: {}", e))?;

            contents = pattern
                .replace_all(&contents, |caps: &regex::Captures| {
                    let mut replacement = String::from("from ");
                    replacement.push_str(new);

                    let after = caps.get(1);
                    match after {
                        Some(after) => {
                            replacement.push_str(after.as_str());
                        }
                        None => {}
                    }

                    replacement.push_str(" import");
                    replacement
                })
                .to_string();

            // Handle module mapping: old. -> new.
            let pattern = Regex::new(&format!(r"{}\.", old))
                .with_context(|e| format!("Could not create regex: {}", e))?;

            contents = pattern
                .replace_all(&contents, format!("{}.", new).as_str())
                .to_string();

            std::fs::write(&file, contents)
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

    pub fn path_2_module(path: &str) -> Result<String, Error> {
        let path = PathBuf::from(path);
        let mut module = String::new();

        for component in path.components() {
            let component = component.as_os_str().to_str().unwrap();

            if component == "." {
                continue;
            }

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

    pub fn module_2_path(module: &str, module_type: &ModuleType) -> Result<PathBuf, Error> {
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

    fn get_sub_modules(self: &mut Self) -> Result<Vec<ModuleManager>, Error> {
        if self.module_type == ModuleType::File {
            return Ok(Vec::new());
        }

        let mut sub_modules = Vec::new();

        let files_iter = Self::travel_root(
            Some(self.path.parent().unwrap().to_str().unwrap().to_string()),
            Some(2),
        )
        .with_context(|e| format!("Could not travel root directory: {}", e))?;

        let accepted_root = self.path.parent().unwrap();
        for file in files_iter {
            let module_type = if file.ends_with("__init__.py") {
                ModuleType::Directory
            } else {
                ModuleType::File
            };

            if module_type == ModuleType::File
                && file.strip_prefix("./").unwrap().parent().unwrap() != accepted_root
            {
                continue;
            }

            match Self::path_2_module(file.to_str().unwrap()) {
                Ok(module) => {
                    if module.starts_with(&self.module) && module != self.module {
                        let mut sub_module_manager = Self::new(&module, module_type, false)?;
                        sub_module_manager.reload()?;
                        sub_modules.push(sub_module_manager);
                    }
                }
                Err(e) => {
                    println!("Could not convert path to module: {}", e);
                }
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
        let (ast, original_code) = parse_ast(&self.path, None).with_context(|e| {
            format!(
                "Could not parse file {}: {}",
                self.path.display(),
                e.to_string()
            )
        })?;
        let (classes, functions, vars) = parse_root_ast(
            ast,
            &original_code,
            &self.path.to_str().unwrap().to_string(),
        )
        .with_context(|e| format!("Could not parse root ast: {}", e))?;

        self.classes = classes;
        self.functions = functions;
        self.vars = vars;
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

    pub fn find(
        self: &Self,
        query: &String,
        prefix: String,
        find_vars: bool,
        find_functions: bool,
        find_classes: bool,
    ) -> Result<Vec<String>, Error> {
        let mut display = String::new();
        display.push_str(&prefix);
        display.push_str("│――");

        let display_path = match self.module_type {
            ModuleType::Directory => self.path.parent().unwrap().to_str().unwrap(),
            ModuleType::File => self.path.to_str().unwrap(),
        };

        let file_path = cformat!(
            "{}/{}",
            std::env::current_dir().unwrap().to_str().unwrap(),
            display_path
        );
        match self.module_type {
            ModuleType::File => {
                display.push_str(cformat!("📄 <green!>{}</green!>\n", file_path).as_str())
            }
            ModuleType::Directory => {
                display.push_str(cformat!("📁 <blue!>{}</blue!>\n", file_path).as_str())
            }
        }

        let sub_prefix = format!("{}│  ", prefix);
        let mut found = false;
        let mut displays = Vec::new();
        displays.push(display);

        if find_vars {
            for var in self.vars.clone() {
                let found_var = var.find(query, None, Some(&sub_prefix));
                if found_var.len() > 0 {
                    found = true;
                    displays.push(found_var);
                }
            }
        }

        if find_functions {
            for function in self.functions.clone() {
                let found_function = function.find(query, None, Some(&sub_prefix));
                if found_function.len() > 0 {
                    found = true;
                    displays.push(found_function);
                }
            }
        }

        if find_classes || find_functions {
            for class in self.classes.clone() {
                let found_class = class.find(query, None, Some(&sub_prefix));
                if found_class.len() > 0 {
                    found = true;
                    displays.push(found_class);
                }
            }
        }

        if self.module_type == ModuleType::Directory {
            for sub_module in &self.sub_modules {
                let sub_displays = sub_module
                    .find(
                        query,
                        format!("{}│  ", prefix),
                        find_vars,
                        find_functions,
                        find_classes,
                    )
                    .with_context(|e| format!("Could not find in sub module: {}", e))?;

                if sub_displays.len() > 0 {
                    found = true;
                    displays.extend(sub_displays)
                }
            }

            displays.push(format!("{}│  *\n", prefix));
        }

        return match found {
            true => Ok(displays),
            false => Ok(Vec::new()),
        };
    }

    pub fn mprint(self: &Self, prefix: String, show_code: bool) {
        let mut display = String::new();
        display.push_str(&prefix);
        display.push_str("│――");
        let display_name = &self.module.split(".").last().unwrap();
        match self.module_type {
            ModuleType::File => {
                display.push_str(cformat!("📄 <green>{}</green>", display_name).as_str())
            }
            ModuleType::Directory => {
                display.push_str(cformat!("📁 <blue>{}</blue>", display_name).as_str())
            }
        }

        println!("{}", display);

        if show_code {
            let sub_prefix = format!("{}│  ", prefix);

            for function in self.functions.clone() {
                print!("{}", function.find("", None, Some(&sub_prefix)))
            }

            for class in self.classes.clone() {
                print!("{}", class.find("", None, Some(&sub_prefix)))
            }
        }

        if self.module_type == ModuleType::Directory {
            for sub_module in &self.sub_modules {
                sub_module.mprint(format!("{}│  ", prefix), show_code);
            }

            println!("{}│  *", prefix);
        }
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
    #[ignore = "Need to test separately"]
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
    #[ignore = "Need to test separately"]
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
    #[ignore = "Need to test separately"]
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
    fn test_mprint() {
        let module_manager = ModuleManager::new("tests", ModuleType::Directory, true).unwrap();
        module_manager.mprint(String::from(""), true);
    }
}
