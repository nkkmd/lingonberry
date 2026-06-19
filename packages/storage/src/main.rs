use std::env;
use std::process;

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{}", error);
        process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("usage: lingonberry-storage <capabilities|run>".to_string());
    };

    match command {
        "capabilities" => {
            println!("{{\"status\":\"ok\",\"service\":\"storage\"}}");
            Ok(())
        }
        "run" => {
            println!("storage node stub");
            Ok(())
        }
        _ => Err(format!("unknown command: {}", command)),
    }
}

