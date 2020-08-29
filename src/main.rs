#[macro_use]
mod utils;
mod models;
mod roblox;
mod tray_menu;

use models::*;
use regex::Regex;
use rustcord::{RichPresenceBuilder, Rustcord};
use sysinfo::{ProcessExt, Signal, SystemExt};
use winreg::{enums, RegKey};
use serde_json::Value;
use winapi::um::{winuser::SetWindowTextW, wincon::GetConsoleWindow};
use tray_menu::wide_str;
use std::{
    env, thread, panic, process::exit, time::SystemTime,
    path::{Path, PathBuf}, io::{stdout, Write}
};

// Checks if the current version of rblx_rich_presence is latest
fn is_latest_version() -> bool {
    let client = reqwest::blocking::Client::new();
    let res = client
        .get("https://api.github.com/repos/thelennylord/rblx_rich_presence/tags")
        .header(reqwest::header::USER_AGENT, "rblx_rich_presence")
        .header(reqwest::header::ACCEPT, "application/vnd.github.v3+json")
        .send()
        .unwrap();
    
    if res.status().is_success() {
        let data = res.text().unwrap();
        let tags: Vec<Value> = serde_json::from_str(&data).unwrap();

        if let Some(tag) = tags.first() {
            if tag["name"].as_str().unwrap() == format!("v{}", env!("CARGO_PKG_VERSION")) {
                return true;
            }
        }
    } else {
        println!("[WARN] Could not fetch version; Received status code {:#?}", res.status());
    }
    false
}

fn main() {
    // Set up panic hook for better error handling
    panic::set_hook(Box::new(|panic_info| {
        let handle = thread::current();
        println!("\n\n\
        Well, this is awkward...\n\
        Roblox Rich Presence encountered an issue and had to shut down.\n\
        To help us diagnose and fix this issue, \
        you may report it along with the below error message at https://github.com/thelennylord/rblx_rich_presence/issues\n\n\
        Error message:\n\
        thread '{}' {}", handle.name().unwrap_or("unknown"), panic_info);
        utils::pause();
    }));

    unsafe {
        SetWindowTextW(GetConsoleWindow(), wide_str("Roblox Rich Presence").as_ptr());
    };

    println!("Roblox Rich Presence v{}", env!("CARGO_PKG_VERSION"));
    if is_latest_version() {
        stdout().write_all(b"\n").unwrap();
    } else {
        println!("A new version of Roblox Rich Presence is available!\nDownload the latest release from https://github.com/thelennylord/rblx_rich_presence/releases\n");
    }

    
    println!("Loading config.toml...");
    let mut config = utils::get_config().unwrap();
    println!("Loaded config.toml");
    
    // Close all instances of Roblox if open
    let system = sysinfo::System::new_all();
    for process in system.get_process_by_name("RobloxPlayerBeta.exe") {
        println!("Found another instance of Roblox opened, killing it...");
        process.kill(Signal::Kill);
    }
    
    if config.general.roblox.is_empty() {
        for process in system.get_process_by_name("RobloxPlayerLauncher.exe") {
            println!("Found another instance of Roblox opened, killing it...");
            process.kill(Signal::Kill);
        }
    } else {
        let file_name: &str = Path::new(&config.general.roblox).file_name().unwrap().to_str().unwrap();
        for process in system.get_process_by_name(file_name) {
            println!("Found another instance of Roblox opened, killing it...");
            process.kill(Signal::Kill);
        }
    }
    
    // Extract Roblox path and save it to config, and replace URL Protocol command with rblx_rich_presence.exe
    let hkcr = RegKey::predef(enums::HKEY_CURRENT_USER);
    let rblx_reg = hkcr.open_subkey_with_flags(r"Software\Classes\roblox-player\shell\open\command", enums::KEY_ALL_ACCESS).unwrap();
    let value: String = rblx_reg.get_value("").unwrap();
    let exe_dir: PathBuf = env::current_exe().unwrap();
    let exe_name: &str = exe_dir.file_name().unwrap().to_str().unwrap();

    let re = Regex::new("(\"[^\"]+\"|[^\\s\"]+)").unwrap();
    let group = re.captures(&value).unwrap();
    let mut launcher_path: &str = group.get(0).unwrap().as_str();
    if launcher_path.chars().next().unwrap() == '"' {
        launcher_path = &launcher_path[1..launcher_path.len()-1];
    }

    if !launcher_path.ends_with(exe_name) {
        config.general.roblox = String::from(launcher_path);
        utils::set_config(&config).unwrap();
    }

    rblx_reg.set_value("", &format!("\"{}\" \"%1\"", env::current_exe().unwrap().to_str().unwrap())).unwrap();
    
    // Setup registry values for passing information
    // TODO: Find a more efficient way of doing it
    let (rblx_rp_reg, _) = hkcr.create_subkey(r"Software\rblx_rich_presence").unwrap();
    rblx_rp_reg.set_value("join_data", &"").unwrap();
    rblx_rp_reg.set_value("join_key", &"").unwrap();
    rblx_rp_reg.set_value("proceed", &"false").unwrap();
    
    let discord = Rustcord::init::<DiscordEventHandler>(
        "725360592570941490",
        true,
        None
    ).unwrap();
    
    match env::args().nth(1) {
        Some(value) => {
            println!("Connecting to Roblox...");
            let mut rblx = roblox::Roblox::new()
                .with_roblosecurity(config.general.roblosecurity)
                .with_path(config.general.roblox)
                .with_url(value);
            rblx.generate_and_save_roblosecurity();
            rblx.join_data.game_info = rblx.generate_ticket().or_else(|| {
                println!("[ERROR] Could not generate authentication ticket; Provided .ROBLOSECURITY cookie might be invalid.");
                utils::pause();
                exit(0);
            }).unwrap();

            if !rblx.verify_roblosecurity() {
                println!("[ERROR] Invalid .ROBLOSECURITY cookie in config.toml");
                utils::pause();
                exit(0);
            }
            
            let rblx = rblx.with_additional_info_from_request_type();
            
            println!("Launching Roblox...");
            if let Err(error) = rblx.launch() {
                println!("[ERROR] Could not launch Roblox; {}", error);
                utils::pause();
                exit(1);
            };
            println!("Launched Roblox\nLoading rich presence...");
            
            let join_data = rblx.get_join_data();
            let now = SystemTime::now();
            let presence = RichPresenceBuilder::new()
                .state("In a game")
                .details(&join_data.place_name)
                .large_image_key("logo")
                .large_image_text("Playing ROBLOX")
                .small_image_key("play_status")
                .small_image_text(&join_data.place_name)
                .start_time(now)
                .build();
            discord.update_presence(presence).unwrap();
            utils::watch(discord, rblx, now);
        }
        None => {            
            let mut close = true;
            for _ in 0..20 {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let proceed: String = rblx_rp_reg.get_value("proceed").unwrap();
                if proceed == "true" {
                    rblx_rp_reg.set_value("proceed", &"false").unwrap();
                    close = false;
                    break;
                }
                discord.run_callbacks();
            }
            if close {
                println!("No pending task detected, closing...");
                exit(0);
            }
            println!("Connecting to Roblox...");
            let join_url: String = rblx_rp_reg.get_value("join_data").unwrap();
            let discord = Rustcord::init::<DiscordEventHandler>(
                "725360592570941490",
                true,
                None
            ).unwrap();

            let mut rblx = roblox::Roblox::new()
                .with_roblosecurity(config.general.roblosecurity)
                .with_path(config.general.roblox)
                .with_url(join_url)
                .with_additional_info_from_request_type();
            rblx.generate_and_save_roblosecurity();
            rblx.join_data.game_info = rblx.generate_ticket().or_else(|| {
                println!("[ERROR] Could not generate authentication ticket; Provided .ROBLOSECURITY cookie might be invalid.");
                utils::pause();
                exit(0);
            }).unwrap();

            println!("Launching Roblox...");
            if let Err(error) = rblx.launch() {
                println!("[ERROR] Could not launch Roblox; {}", error);
                utils::pause();
                exit(0);
            };
            println!("Launched Roblox\nLoading rich presence...");

            let join_data = rblx.get_join_data();
            let now = SystemTime::now();
            let presence = RichPresenceBuilder::new()
                .state("In a game")
                .details(&join_data.place_name)
                .large_image_key("logo")
                .large_image_text("Playing ROBLOX")
                .small_image_key("play_status")
                .small_image_text(&join_data.place_name)
                .start_time(now)
                .build();
            rblx_rp_reg.set_value("join_data", &"").unwrap();
            rblx_rp_reg.set_value("join_key", &"").unwrap();

            discord.update_presence(presence).unwrap();
            utils::watch(discord, rblx, now);
        }
    }

    println!("Closing program...");
}
