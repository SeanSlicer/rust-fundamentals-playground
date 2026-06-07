//! CLI for the todo list: parse args, load, mutate, save, print.
//!
//! Command pattern: arguments are parsed into a `Command` enum first,
//! THEN executed. Parsing and doing stay separate, and the match in
//! `run` is the complete catalog of what the tool can do.

use std::path::PathBuf;
use std::process::ExitCode;

use todo_cli::{load, save, TodoList};

enum Command {
    Add(String),
    List,
    Done(u64),
    Remove(u64),
}

fn parse_args(args: &[String]) -> Result<Command, String> {
    // Slice patterns make small CLI parsing pleasant — each arm is
    // one valid shape of the command line.
    match args {
        [cmd, rest @ ..] if cmd == "add" && !rest.is_empty() => Ok(Command::Add(rest.join(" "))),
        [cmd] if cmd == "list" => Ok(Command::List),
        [] => Ok(Command::List), // bare `todo-cli` = list, the common case
        [cmd, id] if cmd == "done" => parse_id(id).map(Command::Done),
        [cmd, id] if cmd == "remove" => parse_id(id).map(Command::Remove),
        _ => Err(usage()),
    }
}

fn parse_id(text: &str) -> Result<u64, String> {
    text.parse().map_err(|_| format!("'{text}' is not an id"))
}

fn usage() -> String {
    "usage: todo-cli [add <title> | list | done <id> | remove <id>]".to_string()
}

fn print_list(list: &TodoList) {
    if list.items().is_empty() {
        println!("nothing to do!");
        return;
    }
    for item in list.items() {
        let mark = if item.done { "x" } else { " " };
        println!("[{mark}] {:>3}  {}", item.id, item.title);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = parse_args(&args)?;

    // The data file lives next to wherever you run the tool — simple
    // and visible, which suits a learning project. A real tool would
    // use a directories crate to find the platform data dir.
    let path = PathBuf::from("todos.json");
    let mut list = load(&path).map_err(|e| e.to_string())?;

    match command {
        Command::Add(title) => {
            let id = list.add(title);
            println!("added #{id}");
        }
        Command::List => {
            print_list(&list);
            return Ok(()); // nothing changed; skip the save
        }
        Command::Done(id) => match list.mark_done(id) {
            Some(item) => println!("done: {}", item.title),
            None => return Err(format!("no item with id {id}")),
        },
        Command::Remove(id) => match list.remove(id) {
            Some(item) => println!("removed: {}", item.title),
            None => return Err(format!("no item with id {id}")),
        },
    }

    save(&list, &path).map_err(|e| e.to_string())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}
