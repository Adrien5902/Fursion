mod command;
mod error;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let command = command::Command::from_str(&args[1]).unwrap();
    let res = (command.function)(&args[1..]).unwrap();

    println!("{:?}", res);
}
