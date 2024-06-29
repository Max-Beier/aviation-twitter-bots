use shuttle_runtime::tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use shuttle_runtime::tokio::sync::mpsc;
use shuttle_runtime::tokio::task;
use shuttle_runtime::tokio::{io::BufReader, net::TcpListener};

use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::{Client, Url};
use serde_json::json;
use sqlx::PgPool;

use crate::types::{AuthProvider, BotType, Session};

#[derive(Debug)]
pub struct XApi {
    pub url: String,
    pub client: Client,
    pub access_token: String,
}

impl XApi {
    pub async fn new_and_authorize(
        client_id: String,
        client_secret: String,
        bot_type: BotType,
        pool: &PgPool,
    ) -> Self {
        let url = "https://api.twitter.com/2".to_string();

        let client_id = ClientId::new(client_id);
        let client_secret = ClientSecret::new(client_secret);

        let auth_url = AuthUrl::new("https://twitter.com/i/oauth2/authorize".to_string())
            .expect("Invalid authorization endpoint.");
        let token_url = TokenUrl::new("https://api.twitter.com/2/oauth2/token".to_string())
            .expect("Invalid token endpoint.");

        let auth_client =
            BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
                .set_redirect_uri(
                    RedirectUrl::new(
                        "http://earth-highest-aircraft.shuttleapp.rs/callback".to_string(),
                    )
                    .expect("Invalid redirect URL"),
                );

        let sessions: Vec<Session> = sqlx::query_as(
            "SELECT * FROM Sessions WHERE Sessions.provider = 'X' AND Sessions.bot_type = $1;",
        )
        .bind(&bot_type)
        .fetch_all(pool)
        .await
        .unwrap();

        if !sessions.is_empty() {
            let session = sessions.first().unwrap();

            return Self {
                client: Client::new(),
                access_token: session.access_token.to_string(),
                url,
            };
        }

        let (pkce_code_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, _) = auth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("users.read".to_string()))
            .add_scope(Scope::new("tweet.read".to_string()))
            .add_scope(Scope::new("tweet.write".to_string()))
            .set_pkce_challenge(pkce_code_challenge)
            .url();

        println!("[{:?}] Browse to: {}", bot_type, auth_url);

        let (tx, mut rx) = mpsc::channel(1);

        task::spawn(async move {
            let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();

            if let Ok((mut stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(&mut stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line).await.unwrap();

                if request_line.starts_with("GET /callback") {
                    let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                    let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

                    let code = url
                        .query_pairs()
                        .find(|(key, _)| key == "code")
                        .map(|(_, code)| AuthorizationCode::new(code.into_owned()))
                        .expect("Code not found.");

                    let state = url
                        .query_pairs()
                        .find(|(key, _)| key == "state")
                        .map(|(_, state)| CsrfToken::new(state.into_owned()))
                        .expect("State not found.");

                    let message = "DONE";
                    let response = format!(
                        "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                        message.len(),
                        message
                    );
                    stream.write_all(response.as_bytes()).await.unwrap();

                    tx.send((code, state)).await.unwrap();
                }
            }
        });

        let (code, _state) = rx.recv().await.unwrap();

        let tokens = auth_client
            .exchange_code(code)
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .unwrap();

        let access_token = tokens.access_token().secret().to_string();

        sqlx::query("INSERT INTO Sessions (provider, access_token) VALUES ($1, $2);")
            .bind(AuthProvider::X)
            .bind(&access_token)
            .execute(pool)
            .await
            .unwrap();

        Self {
            client: Client::new(),
            access_token,
            url,
        }
    }

    pub async fn tweet(&self, text: String) {
        self.client
            .post(format!("{}/tweets", &self.url))
            .header("Content-Type", "application/json")
            .bearer_auth(&self.access_token)
            .body(json!({"text": text}).to_string())
            .send()
            .await
            .unwrap();
    }
}
