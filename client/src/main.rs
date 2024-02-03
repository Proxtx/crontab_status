use clap::Arg;
use clap::Args;
use clap::Command;
use clap::Parser;
use serde::Serialize;
use std::str;
use tokio::process::Command as TokioCommand;
use url::Url;

#[derive(Parser, Debug)]
#[command(
    author = "Proxtx",
    version = "1.0",
    about = "This tool is the client counterpart for Proxtx/crontab_status. It reports the status of your crontab to the server.",
    long_about = "Visit this page for more information: https://github.com/Proxtx/crontab_status"
)]
struct Cli {
    #[arg(short, long)]
    id: String,

    #[arg(short, long)]
    password: String,

    #[arg(short, long)]
    address: Url,
}

#[tokio::main]
async fn main() {
    let job_command =
        Command::new("job_command").arg(Arg::new("job").last(true).required(true).num_args(1..));
    let cli = Cli::augment_args(job_command);
    let args = cli.get_matches();
    let id = args.get_one::<String>("id").expect("Invalid type for 'ID'");
    let password = args
        .get_one::<String>("password")
        .expect("Invalid type for 'password'");
    let command = args
        .get_many::<String>("job")
        .expect("Invalid type for '<job>'")
        .cloned()
        .collect::<Vec<String>>();
    let mut address = args
        .get_one::<Url>("address")
        .expect("Invalid type for url! Did not provide a correct url. https://example.com/")
        .clone();
    address.set_path("/job-update");

    let mut request = GuardedRequest {
        password: password.clone(),
        data: ClientUpdate {
            job_id: id.clone(),
            command: command.join(" "),
            hostname: gethostname::gethostname()
                .into_string()
                .expect("was unable to get hostname!"),
            update: Update::StartingJob,
        },
    };

    let init_request = request.clone();
    let init_address = address.clone();
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        if let Err(e) =
            client
                .post(init_address)
                .body(serde_json::to_string(&init_request).expect(
                    "Was unable to send request. This is an internal error. Contact Proxtx",
                ))
                .send()
                .await
        {
            println!("Was unable to send request: {}", e)
        }
    });

    for arg in command.iter() {
        println!("{}", arg);
    }

    let mut command_it = command.iter();
    let program = command_it.next().expect("Expected a program to be run");

    let output = TokioCommand::new(program)
        .args(command_it)
        .output()
        .await
        .map_err(|e| println!("Fail to start program: {}", e))
        .expect("");

    let success = output.status.success();
    let stdout = str::from_utf8(&output.stdout).expect("Failed to get stdout of program");
    let stderr = str::from_utf8(&output.stderr).expect("Failed to get stderr of program");

    let response_update = match success {
        true => Update::FinishedJob(String::from(stdout)),
        false => Update::Error(String::from(stderr)),
    };

    request.data.update = response_update;

    let client = reqwest::Client::new();
    if let Err(e) = client
        .post(address)
        .body(
            serde_json::to_string(&request)
                .expect("Failed to send request. Internal Error. Concat Proxtx"),
        )
        .send()
        .await
    {
        println!("Error sending final request: {}", e)
    }
}

#[derive(Serialize, Clone)]
struct GuardedRequest<T> {
    password: String,
    data: T,
}

#[derive(Serialize, Debug, Clone)]
pub struct ClientUpdate {
    job_id: String,
    hostname: String,
    command: String,
    update: Update,
}

#[derive(Serialize, Debug, Clone)]
enum Update {
    StartingJob,
    FinishedJob(String),
    Error(String),
}
