use std::{env::var, io::Write};

use oauth2::{
    basic::BasicClient, reqwest, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, RedirectUrl, Scope, TokenUrl,
};

fn main() {
    let client_id = var("CLIENT_ID").expect("CLIENT ID");
    let client_secret = var("CLIENT_SECRET").expect("CLIENT_SECRET");
    let auth_url = var("AUTH_URL").expect("AUTH_URL");
    let token_url = var("TOKEN_URL").expect("TOKEN_URL");
    let redirect_url = var("REDIRECT_URL").expect("REDIRECT_URL");

    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(auth_url).expect("not auth url"),
        Some(TokenUrl::new(token_url).expect("not token url")),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).expect("not redirect url"));

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("heartrate".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    println!("Browse to: {}", auth_url);
    print!("Enter code: ");
    std::io::stdout().flush().unwrap();

    let mut code = String::new();
    std::io::stdin().read_line(&mut code).unwrap();
    let code = code.trim().to_owned();

    let token_result = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request(reqwest::http_client)
        .expect("token request failed");

    let json = serde_json::to_string_pretty(&token_result).unwrap();
    std::fs::write("token.json", json).unwrap();

    println!("obtained token, saved to token.json");
}
