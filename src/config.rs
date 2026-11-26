use std::env;
use std::time::Duration;

use discord_rich_presence::activity::{ActivityType, StatusDisplayType};

pub struct Config {
    pub discord_client_id: String,
    pub server_url: String,
    pub username: String,
    pub password: String,
    pub poll_interval: Duration,
    pub activity_type: ActivityType,
    pub status_display: StatusDisplayType,
    pub name: Option<String>,
    pub details: String,
    pub state: String,
    pub large_text: String,
    pub fallback_image: String,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
    pub show_artwork: bool,
    pub show_timestamps: bool,
    pub buttons: Vec<(String, String)>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let buttons = [1, 2]
            .into_iter()
            .filter_map(|n| {
                Some((
                    optional(&format!("BUTTON{n}_LABEL"))?,
                    optional(&format!("BUTTON{n}_URL"))?,
                ))
            })
            .collect();

        Ok(Self {
            discord_client_id: required("DISCORD_CLIENT_ID")?,
            server_url: required("AURRAL_URL")?.trim_end_matches('/').to_string(),
            username: required("AURRAL_USERNAME")?,
            password: required("AURRAL_PASSWORD")?,
            poll_interval: Duration::from_secs(
                or("POLL_INTERVAL_SECS", "10")
                    .parse()
                    .map_err(|_| "POLL_INTERVAL_SECS must be a number")?,
            ),
            activity_type: match or("ACTIVITY_TYPE", "listening").to_lowercase().as_str() {
                "playing" => ActivityType::Playing,
                "watching" => ActivityType::Watching,
                "competing" => ActivityType::Competing,
                _ => ActivityType::Listening,
            },
            status_display: match or("STATUS_DISPLAY", "details").to_lowercase().as_str() {
                "name" => StatusDisplayType::Name,
                "state" => StatusDisplayType::State,
                _ => StatusDisplayType::Details,
            },
            name: optional("ACTIVITY_NAME"),
            details: or("DETAILS_FORMAT", "{title}"),
            state: or("STATE_FORMAT", "by {artist}"),
            large_text: or("LARGE_TEXT_FORMAT", "{album}"),
            fallback_image: or("LARGE_IMAGE", "aurral"),
            small_image: optional("SMALL_IMAGE"),
            small_text: optional("SMALL_TEXT"),
            show_artwork: flag("SHOW_ARTWORK", true),
            show_timestamps: flag("SHOW_TIMESTAMPS", true),
            buttons,
        })
    }
}

fn optional(key: &str) -> Option<String> {
    env::var(key).ok().filter(|v| !v.trim().is_empty())
}

fn required(key: &str) -> Result<String, String> {
    optional(key).ok_or_else(|| format!("{key} is not set"))
}

fn or(key: &str, default: &str) -> String {
    optional(key).unwrap_or_else(|| default.to_string())
}

fn flag(key: &str, default: bool) -> bool {
    optional(key).map_or(default, |v| {
        matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on")
    })
}
