use std::io::Write;

use oauth2::{basic::BasicTokenType, EmptyExtraTokenFields, StandardTokenResponse, TokenResponse};

use fitbit_hr::HeartResponse;

fn main() {
    let token_resp: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> =
        serde_json::from_slice(&std::fs::read("token.json").expect("no token.json found"))
            .expect("token.json wrong format");

    let client = reqwest::blocking::Client::new();

    let resp = client
        .get("https://api.fitbit.com/1/user/-/activities/heart/date/2021-06-23/1d.json")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", token_resp.access_token().secret()),
        )
        .send()
        .unwrap();

    let heart: HeartResponse = serde_json::from_slice(&resp.bytes().unwrap()).unwrap();

    let mut csv = std::fs::File::create("heart.csv").unwrap();

    csv.write("time, heartrate\n".as_bytes()).unwrap();

    for reading in heart.activities_heart_intraday.dataset {
        csv.write(format!("{}, {}\n", reading.time, reading.value).as_bytes())
            .unwrap();
    }
}
