use std::env;
use std::fs;

fn main() {
    println!("Logs");

    let args: Vec<String> = env::args().collect();

    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("Git directory initialised");
    } else {
        println!("Unknown command: {}", args[1]);
    }
}
