mod aurral;
mod config;

use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::activity::{Activity, Assets, Button, Timestamps};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};

use aurral::{Aurral, Track};
use config::Config;

fn main() {
    let _ = dotenvy::dotenv();
    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("config error: {e}");
        std::process::exit(1);
    });

    let mut aurral = Aurral::new(
        config.server_url.clone(),
        config.username.clone(),
        config.password.clone(),
    );
    let mut discord = DiscordIpcClient::new(&config.discord_client_id);
    let mut connected = false;
    let mut current: Option<(String, i64)> = None;

    println!("watching {} as {}", config.server_url, config.username);
    loop {
        if !connected {
            connected = discord.connect().is_ok();
            if connected {
                println!("connected to discord");
                current = None;
            }
        }

        match aurral.now_playing() {
            Ok(Some(track)) if connected => {
                let started = match &current {
                    Some((id, started)) if *id == track.id && !finished(*started, &track) => {
                        *started
                    }
                    _ => {
                        println!("now playing: {} - {}", track.artist, track.title);
                        now() - track.minutes_ago as i64 * 60
                    }
                };
                let artwork = config
                    .show_artwork
                    .then(|| aurral.artwork_url(&track))
                    .flatten();
                if let Err(e) = discord.set_activity(build_activity(
                    &config,
                    &track,
                    started,
                    artwork.as_deref(),
                )) {
                    eprintln!("discord update failed: {e}");
                    connected = discord.reconnect().is_ok();
                }
                current = Some((track.id, started));
            }
            Ok(None) if current.take().is_some() => {
                println!("playback stopped");
                if connected && discord.clear_activity().is_err() {
                    connected = discord.reconnect().is_ok();
                }
            }
            Err(e) => eprintln!("aurral poll failed: {e}"),
            _ => {}
        }

        thread::sleep(config.poll_interval);
    }
}

fn build_activity<'a>(
    config: &'a Config,
    track: &Track,
    started: i64,
    artwork: Option<&'a str>,
) -> Activity<'a> {
    let mut assets = Assets::new()
        .large_image(artwork.unwrap_or(&config.fallback_image))
        .large_text(discord_text(&render(&config.large_text, track)));
    if let Some(image) = &config.small_image {
        assets = assets.small_image(image.as_str());
    }
    if let Some(text) = &config.small_text {
        assets = assets.small_text(text.as_str());
    }

    let mut activity = Activity::new()
        .activity_type(config.activity_type.clone())
        .status_display_type(config.status_display.clone())
        .details(discord_text(&render(&config.details, track)))
        .state(discord_text(&render(&config.state, track)))
        .assets(assets);

    if let Some(name) = &config.name {
        activity = activity.name(name.as_str());
    }
    if config.show_timestamps {
        let mut timestamps = Timestamps::new().start(started);
        if let Some(duration) = track.duration.filter(|d| *d > 0) {
            timestamps = timestamps.end(started + duration as i64);
        }
        activity = activity.timestamps(timestamps);
    }
    if !config.buttons.is_empty() {
        activity = activity.buttons(
            config
                .buttons
                .iter()
                .map(|(label, url)| Button::new(label.as_str(), url.as_str()))
                .collect(),
        );
    }
    activity
}

fn render(template: &str, track: &Track) -> String {
    template
        .replace("{title}", &track.title)
        .replace("{artist}", &track.artist)
        .replace("{album}", &track.album)
        .replace(
            "{year}",
            &track.year.map(|y| y.to_string()).unwrap_or_default(),
        )
        .replace(
            "{duration}",
            &track
                .duration
                .map(|d| format!("{}:{:02}", d / 60, d % 60))
                .unwrap_or_default(),
        )
}

// Discord rejects text fields shorter than 2 or longer than 128 characters.
fn discord_text(text: &str) -> String {
    let text = text.trim();
    let mut text: String = if text.chars().count() > 128 {
        text.chars().take(127).chain(['…']).collect()
    } else {
        text.to_string()
    };
    while text.chars().count() < 2 {
        text.push(' ');
    }
    text
}

fn finished(started: i64, track: &Track) -> bool {
    track
        .duration
        .is_some_and(|d| now() > started + d as i64 + 30)
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track() -> Track {
        Track {
            id: "1".into(),
            title: "Song".into(),
            artist: "Band".into(),
            album: "Record".into(),
            year: Some(1999),
            duration: Some(185),
            username: "sam".into(),
            minutes_ago: 0,
        }
    }

    #[test]
    fn renders_placeholders() {
        assert_eq!(
            render("{title} · {artist} · {album} ({year}) {duration}", &track()),
            "Song · Band · Record (1999) 3:05"
        );
    }

    #[test]
    fn clamps_text_to_discord_limits() {
        assert_eq!(discord_text("a"), "a ");
        assert_eq!(discord_text(&"x".repeat(300)).chars().count(), 128);
    }
}
