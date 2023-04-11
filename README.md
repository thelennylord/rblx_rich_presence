<h1 align="center">
    Roblox Rich Presence: Rewrite (Indev)
</h1>
<div align="center">
    <a href="https://github.com/thelennylord/rblx_rich_presence/actions/workflows/build.yml"><img src="https://github.com/thelennylord/rblx_rich_presence/workflows/build/badge.svg?branch=rewrite" alt="Build"></a>
</div>
<div align="center">
    Adds Discord rich presence support to Roblox. The rewrite is still in development, so expect bugs and missing features!
</div>
<div>&nbsp;</div>

## Features
- Display the game you're currently playing
- Display your Roblox username
- Join your friends directly from Discord
- Invite your friends from Discord
- Timestamp of when you've begun to play

## Installation
As this project is currently in the middle of a rewrite, you can download it through the latest [GitHub action workflow](https://github.com/thelennylord/rblx_rich_presence/actions/workflows/build.yml) or by compiling it yourself.

Releases will be made available once the rewrite has been completed.
## Compiling from source
### Prerequisite:
- Go v1.20.2

Clone the repository to your desired location. Open a terminal of your choice and `cd` into the directory and execute the following command to build:
```sh
go build
```
The build output should reside in the same directory with the name `rblx_rich_presence.exe`

## FAQ

### Does it support macOS/Linux?
Currently, it only supports Windows 7 upto Windows 11, however support for macOS and Linux are planned.

## Warning
This program saves your `.ROBLOSECURITY` in your operating system's keyring for the purpose of making Discord invites work and for displaying game information.

## License
[GNU General Public License v3.0](https://github.com/thelennylord/rblx_rich_presence/blob/rewrite/LICENSE)