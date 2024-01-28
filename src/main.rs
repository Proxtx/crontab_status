use std::env;
use std::io::Read;
use std::process;

#[tokio::main]
async fn main() {
    let mut args_it = env::args();
    args_it.next();
    let program = args_it.next().expect("Expected a program to be run");

    let mut cmd = process::Command::new(program)
        .args(args_it)
        .spawn()
        .expect("Unable to spawn child");
    //cmd.stdout.take().unwrap().read(buf);
    if let Some(mut stdout) = cmd.stdout.take() {
        tokio::spawn(async move {
            loop {
                let mut str = String::new();
                stdout.read_to_string(&mut str).unwrap();
                println!("{}", str);
            }
        });
    }
    if let Some(mut stderr) = cmd.stderr.take() {
        tokio::spawn(async move {
            loop {
                let mut str = String::new();
                stderr.read_to_string(&mut str).unwrap();
                println!("{}", str);
            }
        });
    }
}
