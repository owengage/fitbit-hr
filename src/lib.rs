use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HeartResponse {
    pub activities_heart_intraday: ActivitiesIntraday,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ActivitiesIntraday {
    pub dataset: Vec<HeartReading>,
}

#[derive(Serialize, Deserialize)]
pub struct HeartReading {
    pub time: NaiveTime,
    pub value: usize,
}
