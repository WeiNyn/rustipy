use color_print::cformat;

pub trait PythonDef {
    fn get_type(&self) -> String;

    fn get_definition_code(&self) -> String;

    fn find(&self, query: &str, include_file_name: Option<bool>) -> String;
}

#[derive(Debug, Clone)]
pub struct Class {
    pub path: String,
    pub name: String,
    pub methods: Vec<Method>,
    base_classes: Vec<String>,
    pub definition_code: String,
}

impl Class {
    pub fn new(
        path: String,
        name: String,
        methods: Vec<Method>,
        base_classes: Vec<String>,
    ) -> Class {
        let mut class = Class {
            path: path,
            name: name,
            methods: methods,
            base_classes: base_classes,
            definition_code: String::from(""),
        };

        class.definition_code = class.get_definition_code();
        class
    }
}

impl PythonDef for Class {
    fn get_type(&self) -> String {
        String::from("CLASS")
    }

    fn get_definition_code(&self) -> String {
        let mut code = String::from("class ");
        code.push_str(&self.name);
        if self.base_classes.len() > 0 {
            code.push_str("(");
            code.push_str(&self.base_classes.join(", "));
            code.push_str(")");
        }
        code.push_str(":\n");

        for m in &self.methods {
            code.push_str("    ");
            code.push_str(&m.definition_code);
            code.push_str("\n");
        }

        code
    }

    fn find(&self, query: &str, include_file_name: Option<bool>) -> String {
        let mut result = String::new();

        let mut class_def_str = cformat!("<red>class</red> <yellow>{}</yellow>", self.name.clone());
        if self.base_classes.len() > 0 {
            class_def_str.push_str(&cformat!("(<blue>{}</blue>)", self.base_classes.join(", ")));
        }
        class_def_str.push_str(":\n");
        class_def_str = class_def_str.replace(query, cformat!("<bg:green>{}</bg:green>", query).as_str());

        let mut function_defs = String::new();
        for m in &self.methods {
            let function_def = m.find(query, Some(false));
            if function_def.len() > 0 {
                function_defs.push_str("    ");
                function_defs.push_str(&function_def);
            }
        }

        if self.name.contains(query) || function_defs.len() > 0 {
            if include_file_name.is_some() && include_file_name.unwrap() {
                result.push_str(&cformat!("<red><bg:blue> [{}]</bg:blue></red>\n", self.path));
            }
            result.push_str(&class_def_str);
            result.push_str(&function_defs);
        }

        result
    }
}

#[derive(Debug, Clone)]
pub struct Method {
    pub path: String,
    pub name: String,
    return_type: Option<String>,
    arguments: Vec<Attribute>,
    pub definition_code: String,
}

impl Method {
    pub fn new(
        path: String,
        name: String,
        return_type: Option<String>,
        arguments: Vec<Attribute>,
    ) -> Method {
        let mut method = Method {
            path: path,
            name: name,
            return_type: return_type,
            arguments: arguments,
            definition_code: String::from(""),
        };

        method.definition_code = method.get_definition_code();
        method
    }
}

impl PythonDef for Method {
    fn get_type(&self) -> String {
        String::from("METHOD")
    }

    fn get_definition_code(&self) -> String {
        let mut code = String::from("def ");
        code.push_str(&self.name);
        code.push_str("(");
        code.push_str(
            &self
                .arguments
                .iter()
                .map(|a| a.definition_code.clone())
                .collect::<Vec<String>>()
                .join(", "),
        );
        code.push_str(")");
        if self.return_type.is_some() {
            code.push_str(" -> ");
            code.push_str(&self.return_type.clone().unwrap());
        }
        code.push_str(":\n");
        code
    }

    fn find(&self, query: &str, include_file_name: Option<bool>) -> String {
        let mut result = String::new();

        let mut method_def_str =
            cformat!("<red>def</red> <magenta>{}</magenta>", self.name.clone());
        method_def_str.push_str("(");
        method_def_str.push_str(
            &self
                .arguments
                .iter()
                .map(|a| a.definition_code.clone())
                .collect::<Vec<String>>()
                .join(", "),
        );
        method_def_str.push_str(")");
        if self.return_type.is_some() {
            method_def_str.push_str(&format!(" -> {}", self.return_type.clone().unwrap()));
        }
        method_def_str.push_str(":\n");
        method_def_str = method_def_str.replace(query, cformat!("<bg:green>{}</bg:green>", query).as_str());

        if self.name.contains(query) {
            if include_file_name.is_some() && include_file_name.unwrap() {
                result.push_str(&cformat!("<red><bg:blue> [{}]</bg:blue></red>\n", self.path));
            }
            result.push_str(&method_def_str);
        }

        result
    }
}

#[derive(Debug, Clone)]
pub enum ArgType {
    Not,
    Arg,
    Keyword,
    KeywordOnly,
    VarArg,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub path: String,
    pub name: String,
    type_: Option<String>,
    default: Option<String>,
    pub definition_code: String,
    pub arg_type: ArgType,
}

impl Attribute {
    pub fn new(
        path: String,
        name: String,
        type_: Option<String>,
        default: Option<String>,
        arg_type: ArgType,
    ) -> Attribute {
        let mut attribute = Attribute {
            path: path,
            name: name,
            type_: type_,
            default: default,
            definition_code: String::from(""),
            arg_type: arg_type,
        };

        attribute.definition_code = attribute.get_definition_code();
        attribute
    }
}

impl PythonDef for Attribute {
    fn get_type(&self) -> String {
        String::from("ARGUMENT")
    }

    fn get_definition_code(&self) -> String {
        match self.arg_type {
            ArgType::VarArg => String::from("*") + &self.name,
            ArgType::Keyword => String::from("**") + &self.name,
            _ => {
                let mut code = String::from(&self.name);
                if self.type_.is_some() {
                    code.push_str(": ");
                    code.push_str(&self.type_.clone().unwrap());
                }
                if self.default.is_some() {
                    code.push_str(" = ");
                    code.push_str(&self.default.clone().unwrap());
                }
                code
            }
        }
    }

    fn find(&self, query: &str, include_file_name: Option<bool>) -> String {
        let mut result = String::new();

        let mut arg_def_str = self.definition_code.clone();
        arg_def_str = arg_def_str.replace(query, cformat!("<bg:green>{}</bg:green>", query).as_str());

        if self.name.contains(query) {
            if include_file_name.is_some() && include_file_name.unwrap() {
                result.push_str(&cformat!("<red><bg:blue> [{}]</bg:blue></red>\n", self.path));
            }
            result.push_str(&arg_def_str);
            result.push('\n');
        }

        result
    }
}
