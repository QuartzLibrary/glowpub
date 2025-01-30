use std::{io, sync::OnceLock};

use crate::api::GlowficError;
use crate::types::Token;

static TOKEN: OnceLock<Token> = OnceLock::new();

impl Token {
    pub fn try_global() -> Option<Self> {
        TOKEN.get().cloned()
    }
    pub async fn global_or_prompt(
    ) -> io::Result<Result<Result<Self, Vec<GlowficError>>, reqwest::Error>> {
        if let Some(token) = Self::try_global() {
            return Ok(Ok(Ok(token)));
        }

        let token = Self::prompt_user().await;

        match &token {
            Err(e) => {
                log::error!("Failed to retrieve user credentials: {e}.");
            }
            Ok(Err(e)) => {
                log::error!("Connection error while fetching auth token: {e}");
            }
            Ok(Ok(Err(e))) => {
                log::error!("Failed to fetch auth token: {e:?}");
            }
            Ok(Ok(Ok(token))) => {
                log::info!("Setting token {}", &token.token);
                drop(TOKEN.set(token.clone()));
            }
        }

        token
    }
}
impl Token {
    async fn prompt_user() -> io::Result<Result<Result<Self, Vec<GlowficError>>, reqwest::Error>> {
        pub fn read_input() -> io::Result<String> {
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer)?;
            Ok(buffer.trim_end().to_string())
        }

        println!("Login Required.");
        println!("Please enter your username:");
        let username = read_input()?;

        println!("Please enter your password:");
        let password = read_input()?;

        Ok(Self::get(&username, &password).await)
    }
}
