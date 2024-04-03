use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader};
use std::process::Command;

#[derive(Serialize, Deserialize)]
struct Config {
    commands: HashMap<String, String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage:");
        println!("summon <command>");
        println!("summon <-add | -a> <command_name> <PATH|pwd>");
        println!("summon <-remove | -r> <command_name>");
        println!("summon <-show | -s>");
        return;
    }

    let mut config = match load_config() {
        Ok(c) => c,
        Err(e) => {
            println!("Error loading config.json: {}", e);
            return;
        }
    };

    let command = &args[1];

    match command.as_str() {
        "-add" | "-a" => add_command(&args, &mut config),
        "-remove" | "-r" => remove_command(&args, &mut config),
        "-show" | "-s" => show_command(&args, &config),
        _ => open_file(&args, &config),
    }
}

fn add_command(args: &[String], config: &mut Config) {
    if args.len() != 4 {
        println!("Usage: summon <-add | -a> <command_name> <PATH>");
        return;
    }
    let target = &args[2];
    let path = &args[3];
    if path == "pwd" {
        if let Ok(pwd) = env::current_dir() {
            config
                .commands
                .insert(target.clone(), pwd.display().to_string());
            if let Err(e) = save_config(&config) {
                println!("Error saving config: {}", e);
            } else {
                println!("Command '{}' added with current directory as path.", target);
            }
        } else {
            println!("Error getting current directory.");
        }
    } else {
        config.commands.insert(target.clone(), path.clone());
        if let Err(e) = save_config(&config) {
            println!("Error saving config: {}", e);
        } else {
            println!("Command '{}' added with path '{}'.", target, path);
        }
    }
}

fn remove_command(args: &[String], config: &mut Config) {
    if args.len() != 3 {
        println!("Usage: summon <-remove | -r> <command_name>");
        return;
    }
    let target = &args[2];
    if let Some(_) = config.commands.remove(target) {
        if let Err(e) = save_config(&config) {
            println!("Error saving config: {}", e);
        } else {
            println!("Command '{}' removed.", target);
        }
    } else {
        println!("Command '{}' not found.", target);
    }
}

fn show_command(args: &[String], config: &Config) {
    if args.len() != 2 {
        println!("Usage: summon <-show | -s>");
        return;
    }
    println!("Present commands:");
    for (command, path) in &config.commands {
        println!("{} -> {}", command, path);
    }
}

fn open_file(args: &[String], config: &Config) {
    if args.len() != 2 {
        println!("Usage: summon <command>");
        return;
    }
    let command = &args[1];

    let mut found = false;
    if let Some(path) = config.commands.get(command) {
        open_this(path);
        found = true;
    }

    if !found {
        println!("Command '{}' not found.", command);
    }
}

fn open_this(file_path: &str) {
    let file_path = format!("\"{}\"", file_path);
    let result = Command::new("powershell")
        .args(&["start", &file_path])
        .spawn();

    match result {
        Ok(_) => {
            println!("{}", file_path);
        }
        Err(_) => {
            println!("Error: Failed to open file or folder.");
        }
    }
}

fn load_config() -> io::Result<Config> {
    let exe_path = env::current_exe()?;
    let mut config_path = exe_path.clone();
    config_path.set_file_name("config.json");

    if !config_path.exists() {
        return Ok(Config {
            commands: HashMap::new(),
        });
    }

    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

fn save_config(config: &Config) -> io::Result<()> {
    let exe_path = env::current_exe()?;
    let mut config_path = exe_path.clone();
    config_path.set_file_name("config.json");

    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(config_path)?;
    serde_json::to_writer_pretty(file, config)?;
    Ok(())
}