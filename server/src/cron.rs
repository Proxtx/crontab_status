use {
    crate::error::{ConfigError, ConfigResult},
    chrono::{DateTime, Datelike, Timelike, Utc},
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
        sync::Arc,
        time::{Duration, SystemTime},
    },
    tokio::{sync::RwLock, time::sleep},
    url::Url,
};

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Job {
    #[serde(skip_serializing)]
    pub execution_time: CronExecutionTime,
    #[serde(default)]
    pub id: String,
    pub hook: Option<Url>,
}

#[derive(Debug, Clone)]
pub enum CronExecutionTime {
    Reboot,
    Timing(TimeValue, TimeValue, TimeValue, TimeValue, TimeValue),
}

impl CronExecutionTime {
    pub fn matches(&self, time: DateTime<Utc>) -> bool {
        let timing = match self {
            CronExecutionTime::Reboot => return false,
            CronExecutionTime::Timing(t1, t2, t3, t4, t5) => (t1, t2, t3, t4, t5),
        };
        let fits = (
            match timing.0 {
                TimeValue::Every => true,
                TimeValue::Explicit(t) => &(time.minute() as u8) == t,
            },
            match timing.1 {
                TimeValue::Every => true,
                TimeValue::Explicit(t) => &(time.hour() as u8) == t,
            },
            match timing.2 {
                TimeValue::Every => true,
                TimeValue::Explicit(t) => &(time.day() as u8) == t,
            },
            match timing.3 {
                TimeValue::Every => true,
                TimeValue::Explicit(t) => &(time.month() as u8) == t,
            },
            match timing.4 {
                TimeValue::Every => true,
                TimeValue::Explicit(t) => {
                    let weekday = &((time.weekday() as u8) + 1);
                    weekday == t || (weekday == &7 && t == &0)
                }
            },
        );

        fits.0 && fits.1 && fits.3 && (fits.2 || fits.4)
    }

    pub fn now(&self) -> bool {
        self.matches(Utc::now())
    }
}

#[derive(Debug, Clone)]
pub enum TimeValue {
    Every,
    Explicit(u8),
}

#[derive(Clone, Serialize, Debug)]
pub enum Status {
    Running(SystemTime),
    Finished,
    Unknown,
    ExpectingResponse,
    WaitingForResponse(SystemTime),
    ClientError,
}

#[derive(Clone, Serialize, Debug)]
pub struct JobStatus {
    job: Job,
    status: Status,
    log: Option<String>,
    hostname: Option<String>,
    command: Option<String>,
}

impl JobStatus {
    pub fn new(job: Job) -> Self {
        Self {
            job,
            status: Status::Unknown,
            log: Some(String::new()),
            hostname: Some(String::new()),
            command: Some(String::new()),
        }
    }

    pub fn update(&mut self) {
        match self.status {
            Status::Finished | Status::Unknown => {
                if self.job.execution_time.now() {
                    self.status = Status::ExpectingResponse
                }
            }
            Status::ExpectingResponse => {
                if let Some(ref url) = self.job.hook {
                    let url = url.clone();
                    call_hook(url)
                }
                self.status = Status::WaitingForResponse(SystemTime::now())
            }
            _ => {}
        }
    }

    pub fn client_update(&mut self, update: ClientUpdate) {
        self.hostname = Some(update.hostname);
        self.command = Some(update.command);
        match update.update {
            Update::StartingJob => {
                self.log = None;
                self.status = Status::Running(SystemTime::now());
            }
            Update::FinishedJob(log) => {
                self.log = Some(log);
                self.status = Status::Finished;
            }
            Update::Error(err) => {
                self.log = Some(err);
                if let Some(v) = &self.job.hook {
                    call_hook(v.clone());
                }
                self.status = Status::ClientError;
            }
        }
    }
}

pub struct JobManager {
    jobs: Arc<HashMap<String, RwLock<JobStatus>>>,
}

impl JobManager {
    pub fn new(config_jobs: HashMap<String, Job>) -> Self {
        let mut jobs = HashMap::new();
        for (key, job) in config_jobs {
            jobs.insert(key.clone(), RwLock::new(JobStatus::new(job)));
        }

        let jobs = Arc::new(jobs);

        let auto_update_jobs_clone = jobs.clone();
        tokio::spawn(async move {
            loop {
                for (_, job) in auto_update_jobs_clone.iter() {
                    job.write().await.update();
                }
                sleep(Duration::from_secs(60)).await;
            }
        });
        Self { jobs }
    }

    pub async fn update(&self, update: ClientUpdate) -> ConfigResult<()> {
        let mut job = self
            .jobs
            .get(&update.job_id)
            .ok_or(ConfigError::ClientNotFound)?
            .write()
            .await;
        job.client_update(update);

        Ok(())
    }

    pub fn get_jobs(&self) -> Vec<&String> {
        self.jobs.keys().collect()
    }

    pub async fn get_job(&self, job: &str) -> Option<JobStatus> {
        match self.jobs.get(job) {
            None => None,
            Some(v) => Some(v.read().await.clone()),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ClientUpdate {
    job_id: String,
    hostname: String,
    command: String,
    update: Update,
}

#[derive(Deserialize, Debug)]
enum Update {
    StartingJob,
    FinishedJob(String),
    Error(String),
}

fn call_hook(hook: Url) {
    tokio::spawn(async move {
        if let Err(e) = reqwest::get(hook).await {
            println!("Error calling hook: {}", e)
        }
    });
}
