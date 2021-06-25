use std::env::var;

use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest, AuthUrl, ClientId, ClientSecret, EmptyExtraTokenFields, RedirectUrl,
    StandardTokenResponse, TokenResponse, TokenUrl,
};

fn main() {
    let client_id = var("CLIENT_ID").expect("CLIENT ID");
    let client_secret = var("CLIENT_SECRET").expect("CLIENT_SECRET");
    let auth_url = var("AUTH_URL").expect("AUTH_URL");
    let token_url = var("TOKEN_URL").expect("TOKEN_URL");

    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(auth_url).expect("not auth url"),
        Some(TokenUrl::new(token_url).expect("not token url")),
    );

    let token_resp: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> =
        serde_json::from_slice(&std::fs::read("token.json").expect("no token.json found"))
            .expect("token.json wrong format");

    let refresh_resp = client
        .exchange_refresh_token(token_resp.refresh_token().unwrap())
        .request(reqwest::http_client)
        .expect("token refresh failed");

    let json = serde_json::to_string_pretty(&refresh_resp).unwrap();
    std::fs::write("token.json", json).unwrap();

    println!("refreshed token, saved to token.json");
}
