use std::env;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;

fn main() {
    let stdin = io::stdin();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        match input
            .trim()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .as_slice()
        {
            ["exit"] => break,
            ["exit", code] => std::process::exit(code.parse().unwrap()),
            ["echo", args @ ..] => cmd_echo(args),
            ["type", args @ ..] => cmd_type(args),
            ["pwd"] => cmd_pwd(),
            ["cd","~"] => {
                if let Some(home) = std::env::var_os("HOME") {
                    if let Err(e) = std::env::set_current_dir(home) {
                        let msg = e.to_string();
                        println!("cd: {}", msg.split(" (").next().unwrap_or("unknown error"));
                    }
                } else {
                    println!("cd: HOME not set");
                }
            }
            ["cd", dir] => {
                if let Err(e) = std::env::set_current_dir(dir) {
                    let msg = e.to_string();
                    println!(
                        "cd: {}: {}",
                        dir,
                        msg.split(" (").next().unwrap_or("unknown error")
                    );
                }
            }
            [command, args @ ..] => {
                let path = std::env::var("PATH").unwrap_or_default();
                let mut found = false;
                for dir in path.split(':') {
                    let full_path = format!("{}/{}", dir, command);
                    if let Ok(metadata) = std::fs::metadata(&full_path) {
                        if metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0) {
                            let mut child = std::process::Command::new(&full_path)
                                .arg0(command)
                                .args(args)
                                .spawn()
                                .unwrap();
                            let _ = child.wait();
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    println!("{}: command not found", command);
                }
            }
            _ => {}
        }
    }
}

fn cmd_pwd() {
    let current = env::current_dir();
    match current {
        Ok(path_buf) => println!("{}", path_buf.display()),
        Err(e) => eprintln!("Error getting current directory: {}", e),
    }
}

fn cmd_echo(args: &[&str]) {
    println!("{}", args.join(" "));
}

fn cmd_type(args: &[&str]) {
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    let args_len = args.len();

    if args_len == 0 {
        return;
    }

    if args_len > 1 {
        println!("type: too many arguments");
        return;
    }

    let command = args[0];

    if ["type", "echo", "exit", "pwd", "cd"].contains(&command) {
        println!("{} is a shell builtin", command);
        return;
    }

    if let Ok(path_var) = env::var("PATH") {
        for dir in path_var.split(':') {
            let full_path = format!("{}/{}", dir, command);
            if let Ok(metadata) = fs::metadata(&full_path) {
                if metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0) {
                    println!("{} is {}", command, full_path);
                    return;
                }
            }
        }
    }
    println!("{}: not found", command);
}
