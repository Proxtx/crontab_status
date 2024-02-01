use {
    chrono::{DateTime, Datelike, Timelike, Utc},
    serde::Deserialize,
    std::time::SystemTime,
    url::Url,
};

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
