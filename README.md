# aurral-presence

Discord Rich Presence for [Aurral](https://aurral.org). Shows the track you are playing on your Aurral server in your Discord status, with album art and a progress bar.

## How it works

Aurral's web player (including music.aurral.org) saves your play queue to the server every few seconds while music is playing. This app reads it from `GET /api/play-queue` with your personal API key and shows the current track. When the queue stops updating (paused, closed or finished), the presence is cleared within about 45 seconds.

## Setup

1. Create an application at the [Discord developer portal](https://discord.com/developers/applications) and copy its Application ID. Optionally upload an image named `aurral` under Rich Presence > Art Assets as the fallback cover.
2. In Aurral, open Settings > Profile > API access and create a key. Use your own key, not the instance `AURRAL_API_KEY`: playback is per user.
3. Put the secrets in `.env`:

   ```sh
   cp .env.example .env
   ```

4. Adjust `config.json` to taste, then run it with Discord desktop open:

   ```sh
   cargo run --release
   ```

## Secrets (`.env`)

| Variable | Purpose |
| --- | --- |
| `DISCORD_CLIENT_ID` | Discord application ID |
| `AURRAL_URL` | Aurral server URL |
| `AURRAL_API_KEY` | Your personal Aurral API key |

## Customization (`config.json`)

Every field is optional; missing ones use the defaults below. Unknown keys are rejected so typos surface at startup.

```json
{
  "pollIntervalSecs": 10,
  "showTimestamps": true,
  "activity": {
    "type": "listening",
    "statusDisplay": "details",
    "name": "Aurral",
    "details": "{title}",
    "state": "by {artist}",
    "largeText": "{album}"
  },
  "images": {
    "showArtwork": true,
    "fallback": "aurral",
    "small": null,
    "smallText": null
  },
  "buttons": [
    { "label": "Open Aurral", "url": "https://music.aurral.org" }
  ]
}
```

| Field | Default | Purpose |
| --- | --- | --- |
| `pollIntervalSecs` | `10` | Seconds between checks |
| `showTimestamps` | `true` | Elapsed / remaining progress bar |
| `activity.type` | `listening` | `listening`, `playing`, `watching`, `competing` |
| `activity.statusDisplay` | `details` | Member-list text: `name`, `state`, `details` |
| `activity.name` | app name | Overrides the activity name |
| `activity.details` | `{title}` | First line |
| `activity.state` | `by {artist}` | Second line |
| `activity.largeText` | `{album}` | Cover hover text |
| `images.showArtwork` | `true` | Look up album art on the iTunes Search API |
| `images.fallback` | `aurral` | Cover when no art is found: asset key or URL |
| `images.small` / `images.smallText` | none | Small badge image and hover text |
| `buttons` | none | Up to two `{ "label", "url" }` buttons |

Templates accept `{title}`, `{artist}`, `{album}` and `{duration}`.

Artwork lookups send the artist and album name to Apple's public search API. Set `images.showArtwork` to `false` to keep everything on your server. Aurral's own cover URLs need your API key, so they are never sent to Discord.

## Notes

- Playback from third-party Subsonic apps shows up only if the app syncs its play queue to the server.
