use {serde::Deserialize, std::time::SystemTime, url::Url};

#[derive(Deserialize, Debug)]
pub struct Job {
    pub execution_time: CronExecutionTime,
    #[serde(default)]
    pub id: String,
    pub hook: Option<Url>,
}

#[derive(Debug)]
pub enum CronExecutionTime {
    Reboot,
    Timing(TimeValue, TimeValue, TimeValue, TimeValue, TimeValue),
}

impl CronExecutionTime {
    pub fn matches(&self, time: SystemTime) -> bool {
        unimplemented!()
    }

    pub fn now(&self) -> bool {
        self.matches(SystemTime::now())
    }
}

#[derive(Debug)]
pub enum TimeValue {
    Every,
    Explicit(u8),
}

pub enum Status {
    Running,
    Finished,
    Unknown,
    WaitingForResponse(SystemTime),
    ClientError,
}

pub struct JobStatus<'a> {
    job: &'a Job,
    status: Status,
    log: Option<String>,
    client: Option<String>,
    command: Option<String>,
}

impl<'a> JobStatus<'a> {
    pub fn new(job: &'a Job) -> Self {
        Self {
            job,
            status: Status::Unknown,
            log: Some(String::new()),
            client: Some(String::new()),
            command: Some(String::new()),
        }
    }
}
