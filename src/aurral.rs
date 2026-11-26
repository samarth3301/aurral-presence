use std::collections::HashMap;
use std::time::Duration;

use reqwest::blocking::Client;
use serde::Deserialize;

// The web player re-saves the queue at least every 30s while playing; older means paused or closed.
const STALE_AFTER_MS: i64 = 45_000;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub album: String,
    pub duration_ms: Option<u64>,
}

pub struct Playing {
    pub track: Track,
    pub started_ms: i64,
}

#[derive(Deserialize)]
struct QueueResponse {
    queue: Option<Queue>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Queue {
    changed: i64,
    tracks: Vec<Track>,
    current_index: usize,
    position_ms: i64,
}

pub struct Aurral {
    http: Client,
    base_url: String,
    api_key: String,
    artwork: HashMap<String, Option<String>>,
}

impl Aurral {
    pub fn new(base_url: String, api_key: String) -> Self {
        let http = Client::builder()
            .user_agent("aurral-presence")
            .timeout(Duration::from_secs(10))
            .build()
            .expect("http client");
        Self {
            http,
            base_url,
            api_key,
            artwork: HashMap::new(),
        }
    }

    pub fn now_playing(&self, now_ms: i64) -> Result<Option<Playing>, String> {
        let response = self
            .http
            .get(format!("{}/api/play-queue", self.base_url))
            .header("X-Api-Key", &self.api_key)
            .send()
            .map_err(|e| e.to_string())?;
        match response.status().as_u16() {
            401 | 403 => return Err("Aurral rejected the API key".into()),
            code if code >= 400 => return Err(format!("Aurral returned HTTP {code}")),
            _ => {}
        }
        let body: QueueResponse = response.json().map_err(|e| e.to_string())?;
        Ok(body.queue.and_then(|q| current(q, now_ms)))
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

fn current(queue: Queue, now_ms: i64) -> Option<Playing> {
    if now_ms - queue.changed > STALE_AFTER_MS {
        return None;
    }
    let track = queue.tracks.into_iter().nth(queue.current_index)?;
    let started_ms = queue.changed - queue.position_ms;
    let ended = track
        .duration_ms
        .is_some_and(|d| now_ms > started_ms + d as i64 + 5_000);
    (!ended).then_some(Playing { track, started_ms })
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000_000;

    fn queue(changed: i64) -> Queue {
        let body = format!(
            r#"{{"queue":{{"changed":{changed},"changedBy":"web","currentIndex":1,"positionMs":20000,"tracks":[
                {{"id":"1","title":"First","artist":"A","album":"X","durationMs":180000,"streamPath":"/a"}},
                {{"id":"2","title":"Second","artist":"B","album":"Y","durationMs":200000,"artworkUrl":null}}
            ]}}}}"#
        );
        serde_json::from_str::<QueueResponse>(&body)
            .unwrap()
            .queue
            .unwrap()
    }

    #[test]
    fn picks_current_track_and_start_time() {
        let playing = current(queue(NOW - 5_000), NOW).unwrap();
        assert_eq!(playing.track.title, "Second");
        assert_eq!(playing.started_ms, NOW - 25_000);
    }

    #[test]
    fn stale_queue_means_paused() {
        assert!(current(queue(NOW - 60_000), NOW).is_none());
    }

    #[test]
    fn empty_queue_parses() {
        let body: QueueResponse = serde_json::from_str(r#"{"queue":null}"#).unwrap();
        assert!(body.queue.is_none());
    }
}
