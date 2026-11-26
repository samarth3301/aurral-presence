use discord_rich_presence::activity::{Activity, Assets, Button, Timestamps};

use crate::aurral::{Playing, Track};
use crate::config::Config;

pub fn build_activity<'a>(
    config: &'a Config,
    playing: &Playing,
    artwork: Option<&'a str>,
) -> Activity<'a> {
    let track = &playing.track;
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
        let started = playing.started_ms / 1000;
        let mut timestamps = Timestamps::new().start(started);
        if let Some(duration) = track.duration_ms.filter(|d| *d > 0) {
            timestamps = timestamps.end(started + (duration / 1000) as i64);
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
    let duration = track
        .duration_ms
        .map(|ms| format!("{}:{:02}", ms / 60_000, ms / 1000 % 60))
        .unwrap_or_default();
    template
        .replace("{title}", &track.title)
        .replace("{artist}", &track.artist)
        .replace("{album}", &track.album)
        .replace("{duration}", &duration)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_placeholders() {
        let track = Track {
            id: "1".into(),
            title: "Song".into(),
            artist: "Band".into(),
            album: "Record".into(),
            duration_ms: Some(185_000),
        };
        assert_eq!(
            render("{title} · {artist} · {album} {duration}", &track),
            "Song · Band · Record 3:05"
        );
    }

    #[test]
    fn clamps_text_to_discord_limits() {
        assert_eq!(discord_text("a"), "a ");
        assert_eq!(discord_text(&"x".repeat(300)).chars().count(), 128);
    }
}
