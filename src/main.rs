use std::env;
use std::io::Read;
use std::process;

fn main() {
    let mut args_it = env::args();
    args_it.next();
    let program = args_it.next().expect("Expected a program to be run");

    let cmd = process::Command::new(program)
        .args(args_it)
        .spawn()
        .expect("Unable to spawn child");
    //cmd.stdout.take().unwrap().read(buf);
}
