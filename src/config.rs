use std::env;
use std::fs;
use std::time::Duration;

use discord_rich_presence::activity::{ActivityType, StatusDisplayType};
use serde::Deserialize;

pub const CONFIG_PATH: &str = "config.json";

pub struct Config {
    pub discord_client_id: String,
    pub server_url: String,
    pub api_key: String,
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

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Settings {
    poll_interval_secs: u64,
    show_timestamps: bool,
    activity: ActivitySettings,
    images: ImageSettings,
    buttons: Vec<ButtonSettings>,
}

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct ActivitySettings {
    r#type: Kind,
    status_display: Display,
    name: Option<String>,
    details: String,
    state: String,
    large_text: String,
}

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct ImageSettings {
    show_artwork: bool,
    fallback: String,
    small: Option<String>,
    small_text: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ButtonSettings {
    label: String,
    url: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum Kind {
    #[default]
    Listening,
    Playing,
    Watching,
    Competing,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum Display {
    Name,
    State,
    #[default]
    Details,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            poll_interval_secs: 10,
            show_timestamps: true,
            activity: ActivitySettings::default(),
            images: ImageSettings::default(),
            buttons: Vec::new(),
        }
    }
}

impl Default for ActivitySettings {
    fn default() -> Self {
        Self {
            r#type: Kind::default(),
            status_display: Display::default(),
            name: None,
            details: "{title}".into(),
            state: "by {artist}".into(),
            large_text: "{album}".into(),
        }
    }
}

impl Default for ImageSettings {
    fn default() -> Self {
        Self {
            show_artwork: true,
            fallback: "aurral".into(),
            small: None,
            small_text: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let settings = match fs::read_to_string(CONFIG_PATH) {
            Ok(raw) => parse(&raw)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Settings::default(),
            Err(e) => return Err(format!("cannot read {CONFIG_PATH}: {e}")),
        };
        let Settings {
            poll_interval_secs,
            show_timestamps,
            activity,
            images,
            buttons,
        } = settings;

        if buttons.len() > 2 {
            return Err("Discord allows at most 2 buttons".into());
        }

        Ok(Self {
            discord_client_id: required("DISCORD_CLIENT_ID")?,
            server_url: required("AURRAL_URL")?.trim_end_matches('/').to_string(),
            api_key: required("AURRAL_API_KEY")?,
            poll_interval: Duration::from_secs(poll_interval_secs.max(1)),
            activity_type: match activity.r#type {
                Kind::Listening => ActivityType::Listening,
                Kind::Playing => ActivityType::Playing,
                Kind::Watching => ActivityType::Watching,
                Kind::Competing => ActivityType::Competing,
            },
            status_display: match activity.status_display {
                Display::Name => StatusDisplayType::Name,
                Display::State => StatusDisplayType::State,
                Display::Details => StatusDisplayType::Details,
            },
            name: activity.name.filter(|n| !n.trim().is_empty()),
            details: activity.details,
            state: activity.state,
            large_text: activity.large_text,
            fallback_image: images.fallback,
            small_image: images.small.filter(|s| !s.trim().is_empty()),
            small_text: images.small_text.filter(|s| !s.trim().is_empty()),
            show_artwork: images.show_artwork,
            show_timestamps,
            buttons: buttons.into_iter().map(|b| (b.label, b.url)).collect(),
        })
    }
}

fn parse(raw: &str) -> Result<Settings, String> {
    serde_json::from_str(raw).map_err(|e| format!("invalid {CONFIG_PATH}: {e}"))
}

fn required(key: &str) -> Result<String, String> {
    env::var(key)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| format!("{key} is not set"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_config_parses() {
        let settings = parse(include_str!("../config.json")).unwrap();
        assert!(settings.buttons.len() <= 2);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let settings = parse(r#"{"activity":{"type":"playing"}}"#).unwrap();
        assert!(matches!(settings.activity.r#type, Kind::Playing));
        assert_eq!(settings.activity.details, "{title}");
        assert_eq!(settings.poll_interval_secs, 10);
    }

    #[test]
    fn rejects_typos() {
        assert!(parse(r#"{"activty":{}}"#).is_err());
    }
}
