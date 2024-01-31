mod config;
mod cron;
mod error;

#[tokio::main]
async fn main() {
    let cfg = config::Config::load().await.unwrap();
    let exec = &cfg.jobs.get("test").unwrap().execution_time;
    println!("Hello, world: {:?}", exec);
}
