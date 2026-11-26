# aurral-presence

Discord Rich Presence for [Aurral](https://aurral.org). Shows the track you are playing on your Aurral server in your Discord status, with album art and a progress bar.

## How it works

Every few seconds it calls Aurral's Subsonic endpoint `getNowPlaying` with your account and keeps the entry for your username. Aurral fills that list from its web player (including music.aurral.org) and from Subsonic clients that scrobble. When nothing is playing, the presence is cleared.

## Setup

1. Create an application at the [Discord developer portal](https://discord.com/developers/applications) and copy its Application ID. Optionally upload an image named `aurral` under Rich Presence > Art Assets as the fallback cover.
2. Copy the example config and fill it in:

   ```sh
   cp .env.example .env
   ```

3. Run it with Discord desktop open:

   ```sh
   cargo run --release
   ```

## Configuration

All settings live in `.env`; see `.env.example` for every option.

| Variable | Default | Purpose |
| --- | --- | --- |
| `DISCORD_CLIENT_ID` | required | Discord application ID |
| `AURRAL_URL` | required | Aurral server URL |
| `AURRAL_USERNAME` / `AURRAL_PASSWORD` | required | Aurral account |
| `POLL_INTERVAL_SECS` | `10` | Seconds between checks |
| `ACTIVITY_TYPE` | `listening` | `listening`, `playing`, `watching`, `competing` |
| `STATUS_DISPLAY` | `details` | Member-list text: `name`, `state`, `details` |
| `ACTIVITY_NAME` | app name | Overrides the activity name |
| `DETAILS_FORMAT` | `{title}` | First line |
| `STATE_FORMAT` | `by {artist}` | Second line |
| `LARGE_TEXT_FORMAT` | `{album}` | Cover hover text |
| `SHOW_ARTWORK` | `true` | Look up album art on the iTunes Search API |
| `SHOW_TIMESTAMPS` | `true` | Elapsed / remaining progress bar |
| `LARGE_IMAGE` | `aurral` | Fallback cover: asset key or URL |
| `SMALL_IMAGE` / `SMALL_TEXT` | none | Small badge image and hover text |
| `BUTTON1_LABEL` / `BUTTON1_URL` | none | Up to two buttons |

Templates accept `{title}`, `{artist}`, `{album}`, `{year}` and `{duration}`.

Artwork lookups send the artist and album name to Apple's public search API. Set `SHOW_ARTWORK=false` to keep everything on your server. Aurral's own cover URLs need your credentials, so they are never sent to Discord.

## Notes

- Aurral marks a track as playing for its duration, so a paused track stays visible until that window ends.
- The password is sent as a salted token, never in plain text.
