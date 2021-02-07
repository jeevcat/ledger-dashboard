use chrono::{Duration, NaiveDateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: String,
    expiration_time: Option<NaiveDateTime>,
    #[serde(skip_serializing)]
    expires_in: Option<f32>,
}

impl TokenData {
    pub fn update_expiration_time(&mut self) {
        if let Some(expires_in) = self.expires_in {
            let expires_in = Duration::milliseconds((expires_in * 1_000.) as i64);
            info!(
                "New token will expire in {}m{}s",
                expires_in.num_minutes(),
                expires_in.num_seconds() - expires_in.num_minutes() * 60
            );
            // Expire early
            let expires_in: Duration = expires_in / 2;
            info!(
                "Prematurely expire the token in {}m{}s",
                expires_in.num_minutes(),
                expires_in.num_seconds() - expires_in.num_minutes() * 60
            );
            self.expiration_time = Some(Utc::now().naive_utc() + expires_in);
        }
    }

    pub fn is_valid(&self) -> bool {
        let now = Utc::now().naive_utc();
        if let Some(expiration) = self.expiration_time {
            if now >= expiration {
                return false;
            }
        }
        !self.access_token.is_empty()
    }
}
