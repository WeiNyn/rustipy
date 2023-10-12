
use color_print::{cprintln, cprint};
use crate::poetry::check::{check_poetry, check_python, install_poetry};


pub fn check() {
    let (poetry, poetry_version) = check_poetry();
    if !poetry {
        cprintln!("Poetry: <r> ({})</r>", poetry_version);
        println!("Poetry is not installed. Do you want to install it? (y/n): ");
        let mut user_confirm = String::new();
        std::io::stdin().read_line(&mut user_confirm).expect("Failed to read line");
        if user_confirm.trim() == "y" {
            install_poetry();
            let (poetry, poetry_version) = check_poetry();
            if !poetry {
                cprintln!("Poetry: <r> ({})</r>", poetry_version);
            } else {
                cprintln!("Poetry: <g> ({})</g>", poetry_version);
            }
        }
        else {
            cprintln!("Poetry: <r> ({})</r>", poetry_version);
        }
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