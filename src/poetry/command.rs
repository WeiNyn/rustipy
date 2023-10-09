use failure::{Error, ResultExt};
use std::process::Command;

/// TODO: Add more option such as '--schema', '--config_file'
fn create_project(name: &String) -> Result<(), Error> {
    let output = Command::new("poetry")
        .arg("new")
        .arg(name)
        .output()
        .with_context(|_| "Failed to create project")?;

    let output = String::from_utf8_lossy(&output.stdout);
    println!("{}", output);

    Ok(())
}