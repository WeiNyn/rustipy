use std::process::Command;
use regex::Regex;

fn extract_version(output: String) -> String {
    let re = Regex::new(r"(\d+\.\d+\.\d+)").unwrap();
    let caps = re.captures(&output).unwrap();
    return caps[1].to_string();
}

pub fn check_poetry() -> (bool, String) {
    let output = Command::new("poetry").arg("--version").output();

    return match output {
        Ok(output) => {
            let output = String::from_utf8_lossy(&output.stdout);


            (output.contains("Poetry"), extract_version(output.to_string()))
        }
        Err(_) => (false, "Not installed".to_string()),
    };
}

pub fn check_python() -> (bool, String) {
    let output = Command::new("python").arg("--version").output();

    return match output {
        Ok(output) => {
            let output = String::from_utf8_lossy(&output.stdout);
            (output.contains("Python"), extract_version(output.to_string()))
        }
        Err(_) => (false, "Not installed".to_string()),
    };
}