use smth::{ExecutionContext, MathInput, msystem};
mod graphic;
use std::{
    env,
    io::{self, Read},
    path::Path,
};

#[macroquad::main("smth — math plots")]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let execute = args.first().map(String::as_str) == Some("--execute");
    let input_args = if execute { &args[1..] } else { &args[..] };
    let parsed = if input_args.first().map(String::as_str) == Some("--file") {
        match input_args.get(1) {
            Some(path) => msystem(MathInput::File(Path::new(path))),
            None => {
                eprintln!("usage: smth [--execute] [math text] | [--execute] --file PATH");
                return;
            }
        }
    } else if !input_args.is_empty() {
        msystem(MathInput::Text(&input_args.join(" ")))
    } else {
        let mut text = String::new();
        if let Err(error) = io::stdin().read_to_string(&mut text) {
            eprintln!("stdin error: {error}");
            return;
        }
        msystem(MathInput::Text(&text))
    };

    let systems = match parsed {
        Ok(value) => value,
        Err(error) => {
            eprintln!("syntax error: {error}");
            return;
        }
    };
    if !execute {
        println!("{systems:#?}");
        return;
    }
    let output = match systems.execute(&ExecutionContext::default()) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("execution error: {error}");
            return;
        }
    };
    println!("valid: {}", output.valid);
    for system in &output.systems {
        println!("{}: {:?}", system.kind, system.valid);
    }
    if output.plots.is_empty() {
        println!("no plot requirements; nothing to display");
        return;
    }
    graphic::show_plots(&output.plots).await;
}
