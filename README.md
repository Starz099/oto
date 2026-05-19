# Oto

A lightweight, keyboard-first desktop audio mixer overlay written in Rust. Oto offers app-specific volume control and low-latency push-to-talk with initial support for Discord.

## Features

- Voice mixing: per-user voice volume and mute control for voice channels.
- Discord integration: Discord IPC is used for authentication and a persistent socket.
- Native Windows audio: WASAPI-based audio handling for low-latency mixing.


## Quick start

- Download a release from the Releases page or build from source.

Build locally:

```powershell
cargo build --release
```

Run from source:

```powershell
cargo run --release
```

On first run the app will open the Discord authorization popup to obtain an access token. When authorization succeeds the token is saved to the per-user config and Discord features become available.

## Default keyboard shortcuts

- Toggle overlay: `BackQuote`
- Navigate up: `K`
- Navigate down: `J`
- Volume decrease: `H`
- Volume increase: `L`
- Fast step modifier: `LeftShift`
- Mute: `M`
- Jump to top: `GG`
- Jump to bottom: `G`
- Open accordion: `Enter`
- Close accordion: `Escape`
- Push-to-talk mode toggle: `T`
- Push-to-talk hold: `V`

## Keyboard-first design and vim motions

Oto is built to be keyboard-first. Vim-like motions were chosen as default for this because they provide precise navigation with minimal hand movement and strong muscle memory for many users.


## Contributing

- Contributions are welcome. Open an issue or a pull request with a clear description and reproducible steps.

## License

This project is provided under the MIT License.
