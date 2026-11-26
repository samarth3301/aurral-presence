use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::blocking::Client;
use serde::Deserialize;

const CLIENT_NAME: &str = "aurral-presence";
const API_VERSION: &str = "1.16.1";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub album: String,
    pub year: Option<u32>,
    pub duration: Option<u64>,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub minutes_ago: u64,
}

#[derive(Deserialize)]
struct Envelope {
    #[serde(rename = "subsonic-response")]
    response: Response,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    status: String,
    error: Option<ApiError>,
    now_playing: Option<NowPlaying>,
}

#[derive(Deserialize)]
struct ApiError {
    code: u32,
    message: String,
}

#[derive(Deserialize)]
struct NowPlaying {
    #[serde(default)]
    entry: Vec<Track>,
}

pub struct Aurral {
    http: Client,
    base_url: String,
    username: String,
    password: String,
    artwork: HashMap<String, Option<String>>,
}

impl Aurral {
    pub fn new(base_url: String, username: String, password: String) -> Self {
        let http = Client::builder()
            .user_agent(CLIENT_NAME)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("http client");
        Self {
            http,
            base_url,
            username,
            password,
            artwork: HashMap::new(),
        }
    }

    pub fn now_playing(&self) -> Result<Option<Track>, String> {
        let salt = format!(
            "{:x}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let token = format!("{:x}", md5::compute(format!("{}{salt}", self.password)));
        let envelope: Envelope = self
            .http
            .get(format!("{}/rest/getNowPlaying.view", self.base_url))
            .query(&[
                ("u", self.username.as_str()),
                ("t", &token),
                ("s", &salt),
                ("v", API_VERSION),
                ("c", CLIENT_NAME),
                ("f", "json"),
            ])
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.json())
            .map_err(|e| e.to_string())?;

        let response = envelope.response;
        if response.status != "ok" {
            return Err(response.error.map_or_else(
                || "unknown error".into(),
                |e| format!("subsonic error {}: {}", e.code, e.message),
            ));
        }
        Ok(pick_own(
            response.now_playing.map(|n| n.entry).unwrap_or_default(),
            &self.username,
        ))
    }

    pub fn artwork_url(&mut self, track: &Track) -> Option<String> {
        let key = format!("{}\u{0}{}", track.artist, track.album);
        if let Some(cached) = self.artwork.get(&key) {
            return cached.clone();
        }
        let url = self.lookup_artwork(track);
        self.artwork.insert(key, url.clone());
        url
    }

    fn lookup_artwork(&self, track: &Track) -> Option<String> {
        #[derive(Deserialize)]
        struct Search {
            results: Vec<Hit>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Hit {
            artwork_url100: Option<String>,
        }

        let (term, entity) = if track.album.is_empty() {
            (format!("{} {}", track.artist, track.title), "song")
        } else {
            (format!("{} {}", track.artist, track.album), "album")
        };
        let search: Search = self
            .http
            .get("https://itunes.apple.com/search")
            .query(&[("term", term.as_str()), ("entity", entity), ("limit", "1")])
            .send()
            .and_then(|r| r.json())
            .ok()?;
        search
            .results
            .into_iter()
            .next()?
            .artwork_url100
            .map(|u| u.replace("100x100bb", "600x600bb"))
    }
}

fn pick_own(entries: Vec<Track>, username: &str) -> Option<Track> {
    entries
        .into_iter()
        .filter(|t| t.username.eq_ignore_ascii_case(username))
        .min_by_key(|t| t.minutes_ago)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_now_playing_and_picks_own_latest_entry() {
        let body = r#"{"subsonic-response":{"status":"ok","version":"1.16.1","nowPlaying":{"entry":[
            {"id":"1","title":"Old","artist":"A","album":"X","username":"sam","minutesAgo":3},
            {"id":"2","title":"Other","artist":"B","album":"Y","username":"friend","minutesAgo":0},
            {"id":"3","title":"New","artist":"C","album":"Z","duration":200,"year":2020,"username":"Sam","minutesAgo":0}
        ]}}}"#;
        let envelope: Envelope = serde_json::from_str(body).unwrap();
        let entries = envelope.response.now_playing.unwrap().entry;
        let track = pick_own(entries, "sam").unwrap();
        assert_eq!(track.id, "3");
        assert_eq!(track.duration, Some(200));
    }

    #[test]
    fn parses_empty_now_playing() {
        let body = r#"{"subsonic-response":{"status":"ok","nowPlaying":{}}}"#;
        let envelope: Envelope = serde_json::from_str(body).unwrap();
        assert!(envelope.response.now_playing.unwrap().entry.is_empty());
    }
}
