mod aurral;
mod config;
mod presence;

use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::{DiscordIpc, DiscordIpcClient};

use aurral::Aurral;
use config::Config;

const HELP: &str = "Discord Rich Presence for Aurral.

Usage: aurral-presence [--init | --help | --version]

  --init      Create config.json and .env in the settings folder
  --help      Show this help
  --version   Show the version

Settings are read from config.json and .env in the current folder when it has
a config.json, otherwise from the settings folder.";

fn main() {
    let dir = config::settings_dir();
    match std::env::args().nth(1).as_deref() {
        None => {}
        Some("--init") => return init(&dir),
        Some("--help" | "-h") => return println!("{HELP}\n\nSettings folder: {}", dir.display()),
        Some("--version" | "-V") => {
            return println!("aurral-presence {}", env!("CARGO_PKG_VERSION"));
        }
        Some(other) => {
            eprintln!("unknown argument: {other}\n\n{HELP}");
            std::process::exit(2);
        }
    }

    let config = Config::load(&dir).unwrap_or_else(|e| {
        eprintln!("config error: {e}");
        eprintln!("settings folder: {}", dir.display());
        eprintln!("run `aurral-presence --init` to create starter files, then fill in .env");
        std::process::exit(1);
    });

    let mut aurral = Aurral::new(config.server_url.clone(), config.api_key.clone());
    let mut discord = DiscordIpcClient::new(&config.discord_client_id);
    let mut connected = false;
    let mut current: Option<String> = None;

    println!("watching {}", config.server_url);
    loop {
        if !connected {
            connected = discord.connect().is_ok();
            if connected {
                println!("connected to discord");
                current = None;
            }
        }

        match aurral.now_playing(now_ms()) {
            Ok(Some(playing)) if connected => {
                if current.as_ref() != Some(&playing.track.id) {
                    println!(
                        "now playing: {} - {}",
                        playing.track.artist, playing.track.title
                    );
                }
                let artwork = config
                    .show_artwork
                    .then(|| aurral.artwork_url(&playing.track))
                    .flatten();
                if let Err(e) = discord.set_activity(presence::build_activity(
                    &config,
                    &playing,
                    artwork.as_deref(),
                )) {
                    eprintln!("discord update failed: {e}");
                    connected = discord.reconnect().is_ok();
                }
                current = Some(playing.track.id);
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

fn init(dir: &std::path::Path) {
    match config::init(dir) {
        Ok(created) => {
            for path in &created {
                println!("created {}", path.display());
            }
            if created.is_empty() {
                println!("settings already exist in {}", dir.display());
            }
            println!(
                "fill in {}, then run aurral-presence",
                dir.join(".env").display()
            );
        }
        Err(e) => {
            eprintln!("could not create settings in {}: {e}", dir.display());
            std::process::exit(1);
        }
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
