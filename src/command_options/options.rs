use structopt::StructOpt;

#[derive(StructOpt)]
pub enum SubCommand {
    #[structopt(name = "add", about = "Add a module")]
    Add(AddOptions),

    #[structopt(name = "mv", about = "move a module")]
    Move(MoveOptions),

    #[structopt(name = "find", about = "find a module")]
    Find(FindOptions),

    #[structopt(name = "view", about = "view a module")]
    View(ViewOptions),
}

#[derive(StructOpt)]
pub struct AddOptions {
    #[structopt()]
    /// The name of the module to add
    pub module: String,

    #[structopt(short = "f", long = "file")]
    /// Is the module a file?
    pub is_file: bool,

    #[structopt(short = "c", long = "contains")]
    /// List of modules that this module contains (files only)
    pub contains: Option<Vec<String>>,
}

#[derive(StructOpt)]
pub struct MoveOptions {
    #[structopt()]
    /// The name of the module to move
    pub module: String,

    #[structopt()]
    /// The name of the module to move to
    pub to: String,
}

#[derive(StructOpt)]
pub struct FindOptions {
    #[structopt()]
    /// The name of the module to find
    pub query: String,

    #[structopt()]
    /// The name of the module to find
    pub module: Option<String>,

    #[structopt(short = "i", long = "is_file")]
    /// Is the module a file?
    pub is_file: bool,

    #[structopt(short = "f", long = "function")]
    /// find functions
    pub function: bool,

    #[structopt(short = "c", long = "class")]
    /// find classes
    pub class: bool,

    #[structopt(short = "v", long = "variable")]
    /// find variables
    pub variable: bool,
}

#[derive(StructOpt)]
pub struct ViewOptions {
    #[structopt()]
    /// The name of the module to view
    pub module: Option<String>,

    #[structopt(short = "c", long = "code")]
    /// Show the definitions code
    pub code: bool,
}

#[derive(StructOpt)]
pub struct Options {
    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}
