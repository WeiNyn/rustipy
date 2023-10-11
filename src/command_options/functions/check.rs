
use color_print::cprintln;
use crate::poetry::check::{check_poetry, check_python};


pub fn check() {
    let (poetry, poetry_version) = check_poetry();
    if !poetry {
        cprintln!("Poetry: <r> ({})</r>", poetry_version);
    } else {
        cprintln!("Poetry: <g> ({})</g>", poetry_version);
    }

    let (python, python_version) = check_python();
    if !python {
        cprintln!("Python: <r> ({})</r>", python_version);
    } else {
        cprintln!("Python: <g> ({})</g>", python_version);
    }
}