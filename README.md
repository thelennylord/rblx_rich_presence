# Roblox Rich Presence
Adds Discord rich presence support to Roblox

## Features
- Display the game you're currently playing
- Display your Roblox username
- Join your friends directly from Discord
- Invite your friends from Discord
- Timestamp of when you've begun to play

## Installation
1. Download the latest release from https://github.com/thelennylord/rblx_rich_presence/releases
2. Extract `rblx_rich_presence.zip` into a folder
3. Open `config.toml` with a text editor of your choice and enter your `.ROBLOSECURITY` cookie in the `rolosecurity` field
4. Run `rblx_rich_presence.exe` if you're running it for the first time

## Compiling from source
Compiling `rblx_rich_presence` requires
- Rust v1.45.0 stable release or newer 

To compile in dev build, run the following:
```
cargo build
```

To compile in release build, run the following:
```
cargo build --release
```

Program will be built at `./target/debug` or `./target/release` depending on the build mode.


## FAQ

### Why does the program need my .ROBLOSECURITY cookie?
Your `.ROBLOSECURITY` cookie is required for the joining/sending invites through Discord feature and for displaying game information. Your cookie is never shared anywhere else and is used solely for this purpose. If you don't trust me, feel free to download the source from the repository and compile it yourself.

### Does it support macOS?
Currently, it only supports Windows 7 and 10, and there are no plans to implement it for macOS. However, if you are experienced with macOS, feel free to make your own port for macOS!

### Help! I don't see an option to send an invite through Discord.
This happens when the program fails to find the server you're in. Once it finds the server you're in, you'll be able to send invites again. If you feel this is not the case, then you can check by right-clicking `rblx_rich_presence` in your tray menu and clicking Debug.

### Help! I get an error '400 Bad Request'. What do I do?
This error usually occurs when the `.ROBLOSECURITY` you've set in the config is invalid. Recheck if you have entered the cookie properly, and get your `.ROBLOSECURITY` cookie again from Roblox.

### I found a bug, where do I report it?
If you've found a bug, feel free to create an issue about it. Be sure to post the output from the console. (To view the console, right-click `rblx_rich_presence` in the tray menu and click Debug)

## License
[MIT](https://github.com/thelennylord/rblx_rich_presence/blob/master/LICENSE)