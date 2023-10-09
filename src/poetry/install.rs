use failure::{Error, ResultExt};
use std::process::Command;

fn check_poetry() -> bool {
    let output = Command::new("poetry").arg("--version").output();

    return match output {
        Ok(output) => {
            let output = String::from_utf8_lossy(&output.stdout);
            output.contains("Poetry")
        }
        Err(_) => false,
    };
}

fn check_python() -> bool {
    let output = Command::new("python").arg("--version").output();

    return match output {
        Ok(output) => {
            let output = String::from_utf8_lossy(&output.stdout);
            output.contains("Python")
        }
        Err(_) => false,
    };
}

fn install_poetry() -> Result<(), Error> {
    let output = Command::new("curl")
        .arg("-sSL")
        .arg("https://install.python-poetry.org")
        .arg("|")
        .arg("python")
        .arg("-")
        .output()
        .with_context(|_| "Failed to install poetry")?;

    let output = String::from_utf8_lossy(&output.stdout);
    println!("{}", output);

    Ok(())
}