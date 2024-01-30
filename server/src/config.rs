use {
    crate::error::ConfigResult,
    serde::{
        de::{self, Visitor},
        Deserialize,
    },
    std::collections::HashMap,
    tokio::{fs::File, io::AsyncReadExt},
};

#[derive(Deserialize)]
pub struct Config {
    pub password: String,
    pub port: u16,
    pub jobs: HashMap<String, Job>,
}

#[derive(Deserialize, Debug)]
pub struct Job {
    pub execution_time: CronExecutionTime,
}

#[derive(Debug)]
pub enum CronExecutionTime {
    Reboot,
    Timing(TimeValue, TimeValue, TimeValue, TimeValue, TimeValue),
}

impl<'de> Deserialize<'de> for CronExecutionTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(CronExecutionTimeVisitor)
    }
}

struct CronExecutionTimeVisitor;
impl<'de> Visitor<'de> for CronExecutionTimeVisitor {
    type Value = CronExecutionTime;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("@reboot or '* * * * *' with * := valid crontab numbers")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v == "@reboot" {
            return Ok(CronExecutionTime::Reboot);
        }
        let mut minute = TimeValue::Every;
        let mut hour = TimeValue::Every;
        let mut day = TimeValue::Every;
        let mut month = TimeValue::Every;
        let mut weekday = TimeValue::Every;

        let mut str_split = v.split(' ');
        for i in 0..5 {
            let parsed = match str_split.next() {
                Some(v) => v,
                None => return Err(E::custom("Expected more time values")),
            };
            let parsed = match parsed {
                "*" => TimeValue::Every,
                v => TimeValue::Explicit(match v.parse::<u8>() {
                    Ok(v) => v,
                    Err(_e) => {
                        return Err(E::custom("unable to parse u8"));
                    }
                }),
            };
            match i {
                0 => {
                    if let TimeValue::Explicit(ref v) = parsed {
                        if v > &59 {
                            return Err(E::custom("minute int is too big"));
                        }
                    }

                    minute = parsed;
                }
                1 => {
                    if let TimeValue::Explicit(ref v) = parsed {
                        if v > &23 {
                            return Err(E::custom("hour int is too big"));
                        }
                    }
                    hour = parsed;
                }
                2 => {
                    if let TimeValue::Explicit(ref v) = parsed {
                        if v > &31 {
                            return Err(E::custom("day int is too big"));
                        }
                        if v < &1 {
                            return Err(E::custom("day int is too small"));
                        }
                    }
                    day = parsed;
                }
                3 => {
                    if let TimeValue::Explicit(ref v) = parsed {
                        if v > &12 {
                            return Err(E::custom("month int is too big"));
                        }
                        if v < &1 {
                            return Err(E::custom("month int is too small"));
                        }
                    }
                    month = parsed;
                }
                4 => {
                    if let TimeValue::Explicit(ref v) = parsed {
                        if v > &7 {
                            return Err(E::custom("weekday int is too big"));
                        }
                    }
                    weekday = parsed;
                }
                _ => {
                    return Err(E::custom("too many or too few time values given"));
                }
            }
        }

        Ok(CronExecutionTime::Timing(minute, hour, day, month, weekday))
    }
}

#[derive(Debug)]
pub enum TimeValue {
    Every,
    Explicit(u8),
}

impl Config {
    pub async fn load() -> ConfigResult<Self> {
        let mut config = String::new();
        File::open("config.toml")
            .await?
            .read_to_string(&mut config)
            .await?;
        Ok(toml::from_str::<Config>(&config)?)
    }
}
