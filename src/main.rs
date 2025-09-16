use std::process::Command;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    command: Option<String>,
}

fn main() {
    let args = Cli::parse();
    let commands = match args.command {
        Some(command) => vec![command],
        None => vec![],
    };
    let output = Command::new("git")
        .arg("-c")
        .arg("color.ui=always")
        .args(commands)
        .output()
        .expect("failed to execute process");
    let stdout = std::str::from_utf8(&output.stdout.trim_ascii()).unwrap();
    let stderr = std::str::from_utf8(&output.stderr.trim_ascii()).unwrap();
    if stdout.len() > 0 {
        println!("{}", stdout)
    }
    if stderr.len() > 0 {
        println!("{}", stderr)
    }
}
