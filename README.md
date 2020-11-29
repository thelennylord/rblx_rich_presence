<h1 align="center">
    Roblox Rich Presence
</h1>
<div align="center">
    <a href="https://github.com/thelennylord/rblx_rich_presence/actions"><img src="https://github.com/thelennylord/rblx_rich_presence/workflows/Rust/badge.svg" alt="Rust"></a>
</div>
<div align="center">
    Adds Discord rich presence support to Roblox
</div>
<div>&nbsp;</div>

## Features
- Display the game you're currently playing
- Display your Roblox username
- Join your friends directly from Discord
- Invite your friends from Discord
- Timestamp of when you've begun to play

## Installation
1. Download the latest release from https://github.com/thelennylord/rblx_rich_presence/releases
2. Extract `rblx_rich_presence.zip` into a folder
3. Run `rblx_rich_presence.exe` if you're running it for the first time
4. If the program closes on its own, then it has been successfully installed.

## Compiling from source
Compiling `rblx_rich_presence` requires
- Rust v1.45.0 stable release or newer 

To compile in debug build, run the following:
```
cargo build
```

To compile in release build, run the following:
```
cargo build --release
```

Program will be built at `./target/debug` or `./target/release` depending on the build mode.

## FAQ

### Does it support macOS?
Currently, it only supports from Windows 7 upto Windows 10, and there are no plans to implement it for macOS. However, if you are experienced with macOS, feel free to make your own port for macOS!

### Help! I don't see an option to send an invite through Discord.
This happens when the program fails to find the server you're in. Once it finds the server you're in, you'll be able to send invites again. If you feel this is not the case, then you can check by right-clicking `Roblox Rich Presence` in your tray menu and clicking Debug.

### Help! I get an error '400 Bad Request'. What do I do?
This error usually occurs when the `.ROBLOSECURITY` saved in the config is invalid. Recheck if you have entered the cookie properly, and get your `.ROBLOSECURITY` cookie again from Roblox.

### I found a bug, where do I report it?
If you've found a bug, feel free to create an issue about it. Be sure to post the output from the console. (To view the console, right-click `Roblox Rich Presence` in the tray menu and click Debug)

## Warning
This program saves your `.ROBLOSECURITY` *locally* in its config file for the purpose of making Discord invites work and for displaying game information.

## License
[MIT](https://github.com/thelennylord/rblx_rich_presence/blob/master/LICENSE)