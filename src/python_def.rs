pub trait PythonDef {
    fn get_type(&self) -> String;

    fn get_definition_code(&self) -> String {
        String::from("")
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    name: String,
    methods: Vec<Method>,
    base_classes: Vec<String>,
    pub definition_code: String,
}

impl Class {
    pub fn new(name: String, methods: Vec<Method>, base_classes: Vec<String>) -> Class {
        let mut class = Class {
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

        code.push('\n');
        code.push('\n');

        for m in &self.methods {
            code.push_str("    ");
            code.push_str(&m.definition_code);
            code.push_str("\n");
        }

        code
    }
}

#[derive(Debug, Clone)]
pub struct Method {
    name: String,
    return_type: Option<String>,
    arguments: Vec<Attribute>,
    pub definition_code: String,
}

impl Method {
    pub fn new(name: String, return_type: Option<String>, arguments: Vec<Attribute>) -> Method {
        let mut method = Method {
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
    name: String,
    type_: Option<String>,
    default: Option<String>,
    pub definition_code: String,
    pub arg_type: ArgType,
}

impl Attribute {
    pub fn new(
        name: String,
        type_: Option<String>,
        default: Option<String>,
        arg_type: ArgType,
    ) -> Attribute {
        let mut attribute = Attribute {
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
}
