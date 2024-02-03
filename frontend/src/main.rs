use leptos::*;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use url::Url;
use web_sys::wasm_bindgen::JsCast;
const START_URL: &str = "http://cron.proxtx.de";

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}

#[derive(Clone, Debug)]
enum ResponseStatus<T> {
    Success(T),
    Unauthorized,
    RequestError,
    ParseError,
}

impl<T> Serializable for ResponseStatus<T> {
    fn de(_bytes: &str) -> Result<Self, SerializationError> {
        Ok(ResponseStatus::Unauthorized)
    }
    fn ser(&self) -> Result<String, SerializationError> {
        Ok("".to_string())
    }
}

#[component]
fn App() -> impl IntoView {
    let jobs = create_resource(
        || {},
        |_| async move {
            let request = match GuardedRequest::new(()) {
                Some(v) => v,
                None => return ResponseStatus::Unauthorized,
            };
            let client = reqwest::Client::new();
            let res = match client
                .post(format!("{}/get-jobs", START_URL))
                .body(serde_json::to_string(&request).unwrap())
                .send()
                .await
            {
                Ok(v) => match v.text().await {
                    Ok(v) => v,
                    Err(_) => return ResponseStatus::ParseError,
                },
                Err(_) => {
                    return ResponseStatus::RequestError;
                }
            };

            let jobs: Vec<String> = match serde_json::from_str(&res) {
                Ok(v) => v,
                Err(_) => {
                    return ResponseStatus::ParseError;
                }
            };

            ResponseStatus::Success(jobs)
        },
    );

    view! {<p>"Hello "</p> {move || match jobs.get() {
        Some(v) => {
            view! {
                {
                    match v {
                        ResponseStatus::Success(v) => {
                            view! {
                                {
                                    v.into_iter().map(|n| view! {<Job name=n></Job>}).collect_view()
                                }
                            }.into_view()
                        },
                        _ => {
                            logging::log!("{:?}", v);
                            view! {{format!("Error: {:?}", v)}}.into_view()
                        }
                    }
                }
            }.into_view()
        }
        None => {
            view! {"Not world"}.into_view()
        }
    }}}
}

#[component]
fn Job(name: String) -> impl IntoView {
    let name_cl = name.clone();
    let signal = create_signal(name_cl);
    let job = create_resource(signal.0, |name| async move {
        let request = match GuardedRequest::new(name) {
            Some(v) => v,
            None => return ResponseStatus::Unauthorized,
        };

        let client = reqwest::Client::new();
        let res = match client
            .post(format!("{}/get-job", START_URL))
            .body(serde_json::to_string(&request).unwrap())
            .send()
            .await
        {
            Ok(v) => match v.text().await {
                Ok(v) => v,
                Err(_) => return ResponseStatus::ParseError,
            },
            Err(_) => {
                return ResponseStatus::RequestError;
            }
        };

        logging::log!("{}", res);

        let job: JobStatus = match serde_json::from_str(&res) {
            Ok(v) => v,
            Err(_) => {
                return ResponseStatus::ParseError;
            }
        };

        ResponseStatus::Success(job)
    });

    view! {
        <p>{move || match job.get() {
            Some(job) => {
                format!("{:?}", job)
            }
            None => {
                format!("Loading: {}", name)
            }
        }}</p>
    }
}

#[derive(Serialize, Clone)]
struct GuardedRequest<T> {
    password: String,
    data: T,
}

impl<T> GuardedRequest<T> {
    pub fn new(data: T) -> Option<Self> {
        Some(GuardedRequest {
            password: get_password_cookie()?,
            data,
        })
    }
}

fn get_password_cookie() -> Option<String> {
    let html_doc: web_sys::HtmlDocument = document().dyn_into().unwrap(); //document should always be cast-able to HtmlDocument
    let cookies = html_doc.cookie().unwrap(); //cookies are always present
    for cookie in cookie::Cookie::split_parse(cookies) {
        let cookie = cookie.unwrap(); //we got these cookies from the document
        if cookie.name() == "pwd" {
            return Some(cookie.value().to_string());
        }
    }

    None
}

#[derive(Deserialize, Debug, Clone)]
pub struct Job {
    pub execution_time: String,
    #[serde(default)]
    pub id: String,
    pub hook: Option<Url>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct JobStatus {
    job: Job,
    status: Status,
    log: Option<String>,
    hostname: Option<String>,
    command: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub enum Status {
    Running(SystemTime),
    Finished,
    Unknown,
    ExpectingResponse,
    WaitingForResponse(SystemTime),
    ClientError,
}
