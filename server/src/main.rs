mod config;
mod cron;
mod error;

use config::Config;
use cron::ClientUpdate;
use cron::JobManager;
use rocket::http::Status;
use rocket::post;
use rocket::routes;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;

#[rocket::launch]
async fn rocket() -> _ {
    let config = match Config::load().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let manager = cron::JobManager::new(config.clone().jobs);

    let figment = rocket::Config::figment().merge(("port", config.port));
    rocket::custom(figment)
        .manage(manager)
        .manage(config)
        .mount("/", routes![job_update])
}

#[derive(Deserialize)]
struct GuardedRequest<T> {
    password: String,
    data: T,
}

#[post("/job-update", data = "<update>")]
async fn job_update(
    config: &State<Config>,
    manager: &State<JobManager>,
    update: Json<GuardedRequest<ClientUpdate>>,
) -> Status {
    let guard = update.into_inner();
    if guard.password != config.password {
        return Status::Unauthorized;
    }
    match manager.update(guard.data).await {
        Ok(_) => Status::Ok,
        Err(_e) => Status::NotFound,
    }
}
