use clap::Parser;
use std::env;
use std::str;
use tokio::process::Command;

#[tokio::main]
async fn main() {
    let mut args_it = env::args();
    args_it.next();
    let program = args_it.next().expect("Expected a program to be run");

    let output = Command::new(program).args(args_it).output();

    let output = output.await.unwrap();
    println!("{}", str::from_utf8(&output.stdout).unwrap());
    println!("{}", str::from_utf8(&output.stderr).unwrap());
}
