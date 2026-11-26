# Contributing

Thanks for helping out. Bug reports, fixes and small focused features are all welcome. For a bigger feature, open an issue first so we can agree on the approach before you write code.

## Project layout

```
.
├── src/
│   ├── main.rs        CLI arguments and the poll loop
│   ├── config.rs      settings folder, config.json and .env loading, --init
│   ├── aurral.rs      Aurral API client, play/pause tracking, artwork lookup
│   └── presence.rs    builds the Discord activity from a track and the config
├── examples/config/   ready-made config.json variants
├── assets/            logo and README illustrations
├── .githooks/         commit-msg hook that enforces the commit rules
├── .github/           CI, release workflow, issue and PR templates
├── config.json        default customization, also written by --init
└── .env.example       secrets template, also written by --init
```

## Development

You need a recent stable Rust toolchain (1.85 or newer) and the Discord desktop app to try it end to end.

```sh
cp .env.example .env        # fill in your Discord app ID, Aurral URL and personal API key
cargo run                   # reads config.json and .env from the repo folder
```

Before you push, run the same checks as CI:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Logic changes ship with a test next to the code they change (`#[cfg(test)] mod tests` in the same file). Every file in `examples/config/` must parse; a test checks this.

## Commit rules

Commits follow [Conventional Commits](https://www.conventionalcommits.org/). CI checks every commit in a pull request and the pull request title.

```
<type>(<optional scope>): <summary>

<body: what changed and why>
```

| Type | Use for |
| --- | --- |
| `feat` | A new user-facing capability |
| `fix` | A bug fix |
| `docs` | README, CONTRIBUTING, comments-only changes |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `perf` | A performance improvement |
| `test` | Adding or fixing tests |
| `build` | Cargo.toml, dependencies, release profile |
| `ci` | GitHub Actions and repo automation |
| `chore` | Anything else that doesn't touch the shipped binary |
| `revert` | Reverting an earlier commit |

Suggested scopes: `config`, `aurral`, `presence`, `cli`, `release`.

- Write the summary in the imperative, lower case, no trailing period, under 72 characters: `fix(aurral): treat an early save as a pause`.
- Use the body to explain why. Wrap it at about 72 characters.
- Mark a breaking change with `!` after the type or scope (`feat(config)!: rename largeText`) and describe the migration in the body.
- One logical change per commit. Keep formatting-only or unrelated changes in their own commits.

Turn on the hook once and git checks each message as you commit:

```sh
git config core.hooksPath .githooks
```

## Branches and pull requests

- Branch from `main` and name it after the type: `fix/resume-detection`, `feat/small-image-template`, `docs/windows-setup`.
- Keep a pull request to one change, fill in the template, and make sure CI is green.
- Update `CHANGELOG.md` under **Unreleased** for anything a user would notice.

## Releases

Maintainers release by bumping `version` in `Cargo.toml`, moving the **Unreleased** notes in `CHANGELOG.md` under the new version, committing as `chore(release): vX.Y.Z`, then pushing a matching tag:

```sh
git tag vX.Y.Z
git push origin vX.Y.Z
```

The release workflow checks that the tag matches `Cargo.toml`, runs the tests, creates the GitHub release and attaches binaries for macOS, Linux and Windows.
