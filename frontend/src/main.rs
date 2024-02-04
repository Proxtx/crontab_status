use leptos::*;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use url::Url;
use web_sys::{wasm_bindgen::JsCast, HtmlInputElement};

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
                .post(
                    Url::parse(&leptos::window().origin())
                        .unwrap()
                        .join("/get-jobs")
                        .unwrap(),
                )
                .body(serde_json::to_string(&request).unwrap())
                .send()
                .await
            {
                Ok(v) => {
                    if let StatusCode::UNAUTHORIZED = v.status() {
                        return ResponseStatus::Unauthorized;
                    }

                    match v.text().await {
                        Ok(v) => v,
                        Err(_) => return ResponseStatus::ParseError,
                    }
                }
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

    view! {{move || match jobs.get() {
        Some(v) => {
            view! {
                {
                    match v {
                        ResponseStatus::Success(v) => {
                            view! {
                                <h1>Status</h1>
                                {
                                    v.into_iter().map(|n| view! {<Job name=n></Job>}).collect_view()
                                }
                            }.into_view()
                        },
                        ResponseStatus::Unauthorized => {
                            view! {
                                <Login />
                            }.into_view()
                        }
                        _ => {

                            view! {<h1>Status</h1>{format!("Error: {:?}", v)}}.into_view()
                        }
                    }
                }
            }.into_view()
        }
        None => {
            view! {"Loading..."}.into_view()
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
            .post(
                Url::parse(&leptos::window().origin())
                    .unwrap()
                    .join("/get-job")
                    .unwrap(),
            )
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
                match job {
                    ResponseStatus::Success(v) => {
                        let (text, class) = match &v.status {
                            Status::ClientError => ("Failed", "statusError"),
                            Status::Finished(_) => ("Operational", "statusFinished"),
                            Status::WaitingForResponse(_) => ("Challenge", "statusWaiting"),
                            Status::Running(_) => ("Running", "statusRunning"),
                            Status::ExpectingResponse => ("Waiting", "statusWaiting"),
                            Status::Unknown => ("Unknown", "statusUnknown"),
                        };
                        view! {
                            <div class=format!("status {}", class)>
                            <a class="statusText">{text}</a>
                            <JobData job_status = v/>
                            </div>

                        }.into_view()
                    }
                    _ => {
                        format!("Error loading job: {}", name).into_view()
                    }
                }
            }
            None => {
                format!("Loading: {}", name).into_view()
            }
        }}</p>
    }
}

#[component]
fn JobData(job_status: JobStatus) -> impl IntoView {
    view! {
        <div class="jobData"><h3>{job_status.job.id}</h3>
        {
            if let Some(v) = job_status.hostname {
                view! {
                    <IconAttribute icon_path="icons/server.svg".to_string() text={v}/>
                }
            }
            else {
                view! {}.into_view()
            }
        }
        <IconAttribute icon_path="icons/timer.svg".to_string() text={job_status.job.execution_time}/>
        {
            if let Some(v) = job_status.command {
                view! {
                    <IconAttribute icon_path="icons/terminal.svg".to_string() text={v}/>
                }
            }
            else {
                view! {}.into_view()
            }
        }
        <Log log=job_status.log.unwrap_or_default()/>
        </div>
    }
}

#[component]
fn IconAttribute(icon_path: String, text: String) -> impl IntoView {
    view! {
        <div class="attribute"><img src={icon_path} /><a>{text}</a></div>
    }
}

#[component]
fn Log(log: String) -> impl IntoView {
    view! {
        <div class="log">{log}</div>
    }
}

#[component]
fn Login() -> impl IntoView {
    view! {
        <div class="jobData">
        <h2>Login</h2>
        <input id="password_input" on:change=|v| {set_password_cookie(v.target().unwrap().dyn_into::<HtmlInputElement>().unwrap().value()); reload()} type="password" />
        </div>
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

fn set_password_cookie(password: String) {
    let html_doc: web_sys::HtmlDocument = document().dyn_into().unwrap();
    let cookie = cookie::Cookie::new("pwd", password);
    html_doc.set_cookie(&cookie.to_string()).unwrap();
}

fn reload() {
    leptos::window().location().reload().unwrap();
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
    Finished(SystemTime),
    Unknown,
    ExpectingResponse,
    WaitingForResponse(SystemTime),
    ClientError,
}
