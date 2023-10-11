use color_print::cprintln;
use failure::{Error, ResultExt};
use rustpython_parser::ast::{
    Arg, ArgWithDefault, Expr, Ranged, Stmt, StmtAnnAssign, StmtAssign, StmtClassDef,
    StmtFunctionDef,
};
use rustpython_parser::{ast, Parse};
use std::{io::Read, path::PathBuf};

use crate::python_def::{ArgType, Attribute, Class, Method};

pub fn parse_ast(
    path: &PathBuf,
    source_path: Option<String>,
) -> Result<(Vec<Stmt>, String), Error> {
    let mut contents = String::new();
    std::fs::File::open(path)
        .with_context(|_| format!("Could not open file {:?}", path))?
        .read_to_string(&mut contents)
        .with_context(|_| format!("Could not read file {:?}", path))?;

    let source_path = source_path.unwrap_or(String::from("./"));

    let ast = ast::Suite::parse(&contents, &source_path)
        .with_context(|_| format!("Could not parse file {:?}", path));

    return match ast {
        Ok(ast) => Ok((ast, contents)),
        Err(e) => {
            cprintln!("<R> Error: {}</R>", e);
            Ok((Vec::new(), contents))
        }
    };
}

pub fn parse_root_ast(
    ast: Vec<Stmt>,
    original_code: &String,
    path: &String,
) -> Result<(Vec<Class>, Vec<Method>, Vec<Attribute>), Error> {
    let mut classes = Vec::new();
    let mut functions = Vec::new();
    let mut attributes = Vec::new();

    for stmt in ast {
        match stmt {
            Stmt::ClassDef(c) => classes.push(parse_class_def(&c, original_code, path)?),
            Stmt::FunctionDef(f) => functions.push(parse_function_def(&f, original_code, path)?),
            Stmt::AsyncFunctionDef(f) => {
                let _f = StmtFunctionDef {
                    name: f.name,
                    args: f.args,
                    body: f.body,
                    decorator_list: f.decorator_list,
                    returns: f.returns,
                    type_comment: f.type_comment,
                    range: f.range,
                    type_params: f.type_params,
                };

                let mut function = parse_function_def(&_f, original_code, path)?;
                function.set_async(true);

                functions.push(function)
            }
            Stmt::Assign(a) => attributes.extend(parse_assign(&a, original_code, path)?),
            Stmt::AnnAssign(a) => {
                let attribute = parse_ann_assign(&a, original_code, path)
                    .with_context(|e| format!("Error parsing attribute: {}", e))?;

                if attribute.is_some() {
                    attributes.push(attribute.unwrap());
                }
            }
            _ => {}
        }
    }

    return Ok((classes, functions, attributes));
}

fn parse_assign(
    assign: &StmtAssign,
    original_code: &String,
    path: &String,
) -> Result<Vec<Attribute>, Error> {
    let names = assign
        .targets
        .iter()
        .filter(|e| match e {
            Expr::Name(_) => true,
            _ => false,
        })
        .map(|n| match n {
            Expr::Name(n) => n.id.to_string(),
            _ => panic!("This should never happen"),
        });

    let value_range = assign.value.range();
    let value = original_code[value_range].to_string();

    let mut attributes = Vec::new();

    for name in names {
        attributes.push(Attribute::new(
            path.to_string(),
            name,
            None,
            Some(value.clone()),
            ArgType::Not,
        ));
    }

    Ok(attributes)
}

fn parse_ann_assign(
    ann_assign: &StmtAnnAssign,
    original_code: &String,
    path: &String,
) -> Result<Option<Attribute>, Error> {
    let name = match *ann_assign.target.clone() {
        Expr::Name(n) => Some(n.id.to_string()),
        _ => None,
    };

    if name.is_none() {
        return Ok(None);
    }

    let type_range = ann_assign.annotation.range();
    let type_ = Some(original_code[type_range].to_string());

    let value = match &ann_assign.value {
        Some(v) => Some(original_code[v.range()].to_string()),
        None => None,
    };

    Ok(Some(Attribute::new(
        path.to_string(),
        name.unwrap(),
        type_,
        value,
        ArgType::Not,
    )))
}

fn parse_function_def(
    function_def: &StmtFunctionDef,
    original_code: &String,
    path: &String,
) -> Result<Method, Error> {
    let name = function_def.name.to_string();

    let args = function_def
        .args
        .args
        .iter()
        .map(
            |a| match parse_arg_with_default(a, &original_code, ArgType::Arg, path) {
                Ok(a) => a,
                Err(e) => panic!("Error parsing argument: {}", e),
            },
        )
        .collect::<Vec<Attribute>>();

    let var_arg = match &function_def.args.vararg {
        Some(a) => match parse_arg(&a, &original_code, ArgType::VarArg, path) {
            Ok(a) => Some(a),
            Err(e) => panic!("Error parsing argument: {}", e),
        },
        None => None,
    };

    let kw_only = function_def
        .args
        .kwonlyargs
        .iter()
        .map(
            |a| match parse_arg_with_default(a, &original_code, ArgType::KeywordOnly, path) {
                Ok(a) => a,
                Err(e) => panic!("Error parsing argument: {}", e),
            },
        )
        .collect::<Vec<Attribute>>();

    let kw_arg = match &function_def.args.kwarg {
        Some(a) => match parse_arg(&a, &original_code, ArgType::Keyword, path) {
            Ok(a) => Some(a),
            Err(e) => panic!("Error parsing argument: {}", e),
        },
        None => None,
    };

    let return_type = match &function_def.returns {
        Some(r) => Some(
            original_code[r.range()]
                .trim()
                .trim_end_matches(":")
                .to_string(),
        ),
        None => None,
    };

    let mut arguments = args;
    if var_arg.is_some() {
        arguments.push(var_arg.unwrap());
    }
    arguments.extend(kw_only);
    if kw_arg.is_some() {
        arguments.push(kw_arg.unwrap());
    }

    Ok(Method::new(path.to_string(), name, return_type, arguments))
}

fn parse_arg_with_default(
    arg: &ArgWithDefault,
    original_code: &String,
    arg_type: ArgType,
    path: &String,
) -> Result<Attribute, Error> {
    let def = arg.def.clone();
    let name = def.arg.to_string();
    let type_ = def.annotation.map(|a| original_code[a.range()].to_string());

    let default = arg.default.clone();

    let default_value = match default {
        Some(v) => Some(original_code[v.range()].to_string()),
        None => None,
    };

    Ok(Attribute::new(
        path.to_string(),
        name,
        type_,
        default_value,
        arg_type,
    ))
}

fn parse_arg(
    arg: &Arg,
    original_code: &String,
    arg_type: ArgType,
    path: &String,
) -> Result<Attribute, Error> {
    let name = arg.arg.to_string();
    let type_ = arg
        .annotation
        .clone()
        .map(|a| original_code[a.range()].to_string());

    Ok(Attribute::new(
        path.to_string(),
        name,
        type_,
        None,
        arg_type,
    ))
}

fn parse_class_def(
    class_def: &StmtClassDef,
    original_code: &String,
    path: &String,
) -> Result<Class, Error> {
    let name = class_def.name.to_string();

    let bases = class_def
        .bases
        .iter()
        .map(|b| original_code[b.range()].to_string())
        .collect::<Vec<String>>();

    let mut methods = Vec::new();

    for stmt in &class_def.body {
        match stmt {
            Stmt::FunctionDef(f) => methods.push(
                parse_function_def(f, original_code, path)
                    .with_context(|e| format!("Error parsing method: {}", e))?,
            ),
            Stmt::AsyncFunctionDef(f) => {
                let f = f.clone();
                let _f = StmtFunctionDef {
                    name: f.name,
                    args: f.args,
                    body: f.body,
                    decorator_list: f.decorator_list,
                    returns: f.returns,
                    type_comment: f.type_comment,
                    range: f.range,
                    type_params: f.type_params,
                };

                let mut method = parse_function_def(&_f, original_code, path)?;
                method.set_async(true);

                methods.push(method)
            }
            _ => {}
        }
    }

    Ok(Class::new(path.to_string(), name, methods, bases))
}
