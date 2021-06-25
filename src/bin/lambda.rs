use std::env::var;

use chrono::Datelike;
use lambda_runtime::{handler_fn, Context, Error};
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    AuthUrl, ClientId, ClientSecret, EmptyExtraTokenFields, StandardTokenResponse, TokenResponse,
    TokenUrl,
};
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3Client, S3};
use rusoto_secretsmanager::{GetSecretValueRequest, SecretsManager, SecretsManagerClient};
use serde::Deserialize;
use serde_json::Value;
use tokio::io::AsyncReadExt;

#[derive(Deserialize)]
struct SecretData {
    client_secret: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

type TokenResp = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

async fn fetch_token(bucket_name: String, key: String) -> Result<TokenResp, Error> {
    // Get an S3 client
    let s3 = S3Client::new(Region::EuWest2);
    let token_obj = s3
        .get_object(GetObjectRequest {
            bucket: bucket_name,
            key,
            ..Default::default()
        })
        .await?;

    let mut buf = vec![];
    token_obj
        .body
        .ok_or("no body")?
        .into_async_read()
        .read_to_end(&mut buf)
        .await?;

    Ok(serde_json::from_slice(&buf)?)
}

async fn refresh_token(client: &BasicClient, token: TokenResp) -> Result<TokenResp, Error> {
    Ok(client
        .exchange_refresh_token(token.refresh_token().unwrap())
        .request_async(oauth2::reqwest::async_http_client)
        .await?)
}

async fn put_as_json<T: serde::Serialize>(
    obj: &T,
    bucket: String,
    key: String,
) -> Result<(), Error> {
    let json = serde_json::to_string_pretty(obj)?.into_bytes();

    let s3 = S3Client::new(Region::EuWest2);

    s3.put_object(PutObjectRequest {
        bucket,
        key,
        body: Some(json.into()),
        ..Default::default()
    })
    .await?;

    Ok(())
}

async fn get_secrets(arn: String) -> Result<SecretData, Error> {
    let sm = SecretsManagerClient::new(Region::EuWest2);
    let secret_data = sm
        .get_secret_value(GetSecretValueRequest {
            secret_id: arn,
            ..Default::default()
        })
        .await?
        .secret_string
        .ok_or("client secret was not a string")?;
    Ok(serde_json::from_str(&secret_data)?)
}

async fn func(_event: Value, _: Context) -> Result<(), Error> {
    let client_id = var("CLIENT_ID").expect("CLIENT ID");
    let client_secret_arn = var("CLIENT_SECRET_ARN").expect("CLIENT_SECRET_ARN");
    let auth_url = var("AUTH_URL").expect("AUTH_URL");
    let token_url = var("TOKEN_URL").expect("TOKEN_URL");
    let bucket_name = var("BUCKET_NAME").expect("BUCKET_NAME");
    let token_key = "token.json".to_owned();

    let secret_data = get_secrets(client_secret_arn).await?;

    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(secret_data.client_secret)),
        AuthUrl::new(auth_url)?,
        Some(TokenUrl::new(token_url)?),
    );

    // Get the current token, assume it has expired and refresh it, then write
    // the refreshed token back to S3 for next time.
    let token_resp = fetch_token(bucket_name.clone(), token_key.clone()).await?;
    let token_resp = refresh_token(&client, token_resp).await?;
    put_as_json(&token_resp, bucket_name.clone(), token_key).await?;
    println!("refreshed token, saved to token.json");

    let today = chrono::Local::today();
    let days = today.num_days_from_ce();
    let yesterday = chrono::NaiveDate::from_num_days_from_ce(days - 1);
    let date = yesterday.format("%Y-%m-%d").to_string();

    println!("requesting heart rate data for {}", date);

    let url = format!(
        "https://api.fitbit.com/1/user/-/activities/heart/date/{}/1d.json",
        date
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", token_resp.access_token().secret()),
        )
        .send()
        .await?;

    if resp.status().is_success() {
        let heart: Value = serde_json::from_slice(&resp.bytes().await?)?;
        put_as_json(&heart, bucket_name.clone(), format!("days/{}.json", date)).await?;
    }

    Ok(())
}
