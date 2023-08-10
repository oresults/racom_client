use std::str::FromStr;
use chrono::{NaiveTime, Timelike};
use serde_derive::Serialize;

#[derive(PartialEq, Clone, Debug, Serialize)]
pub enum Channel {
    BLUE
}

#[derive(Debug, Serialize)]
pub struct PunchesRequest {
    pub api_token: String,
    pub records: Vec<Punch>
}

#[derive(Debug, Clone, Serialize)]
pub struct Punch {
    pub channel: Channel,
    pub code: i16,
    pub card: i32,
    pub time: u32,
}

impl FromStr for Punch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_string();
        let card_num = s[0..8].trim().parse::<i32>()
            .map_err(|e| format!("Failed to parse card number: {:?}, error: {:?}", s[0..8].to_string(), e))?;
        let code = s[9..13].trim().parse::<i16>()
            .map_err(|e| format!("Failed to parse code: {:?}, error: {:?}", s[9..13].to_string(), e))?;
        let time = NaiveTime::parse_from_str(&s[14..22].trim(), "%H:%M:%S")
            .map_err(|e| format!("Failed to parse time: {:?}, error: {:?}", s[14..22].to_string(), e))?;
        //let millis = s[23..24].parse::<i64>()
        //    .map_err(|e| format!("Failed to parse milliseconds: {:?}, error: {:?}", s[23..24].to_string(), e))? * 100;
        //time += Duration::milliseconds(millis);

        let punch =  Punch {
            channel: Channel::BLUE,
            code,
            card: card_num,
            time: time.num_seconds_from_midnight()
        };

        Ok(punch)
    }
}