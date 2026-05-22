# Oto

A lightweight, keyboard-first desktop audio mixer overlay for windows written in Rust, featuring app-specific volume control and zero-latency global push-to-talk, currently supporting Discord.

## Demo
https://github.com/user-attachments/assets/842a8917-2b43-42c0-854a-5d4815255a61

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

On first run, you will need to provide your own Discord Client ID and Client Secret in the settings. This is because Discord application verification is restrictive for this type of application.

## Discord Setup (Bring Your Own Credentials)

To use the Discord integration features, you must create your own Discord application:

1.  Go to the [Discord Developer Portal](https://discord.com/developers/applications).
2.  Click **New Application** and give it a name (e.g., "Oto Mixer").
3.  In the left sidebar, go to **OAuth2**.
4.  Under **Redirects**, add `http://127.0.0.1` and click **Save Changes**.
5.  Copy the **Client ID** from the **General Information** or **OAuth2** page.
6.  Go to the **OAuth2** page and click **Reset Secret** (or **Copy** if you haven't reset it yet) to get your **Client Secret**.
7.  Open Oto, go to **Settings** (gear icon), and scroll down to **Discord API**.
8.  Paste your **Client ID** and **Client Secret**.
9.  Click **Save Changes and Restart App**.
10. On restart, Oto will trigger the Discord authorization popup to obtain an access token.

## Default keyboard shortcuts

- Toggle overlay: ``BackQuote`
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
