#[macro_use]
mod utils;
mod models;
mod roblox;
mod tray_menu;

use models::*;
use rustcord::{RichPresenceBuilder, Rustcord};
use std::env;
use std::time::SystemTime;
use std::io::{stdout, Write};
use sysinfo::{ProcessExt, Signal, SystemExt};
use winreg::{enums, RegKey};
use serde_json::Value;

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
        println!("WARN: Could not fetch version; Received status code {:#?}", res.status());
    }
    false
}

fn main() {
    println!("Roblox Rich Presence v{}", env!("CARGO_PKG_VERSION"));
    if is_latest_version() {
        stdout().write_all(b"\n").unwrap();
    } else {
        println!("A new version of Roblox Rich Presence is available!\nDownload the latest release from https://github.com/thelennylord/rblx_rich_presence/releases\n");
    }

    // Close all instances of Roblox if open
    let system = sysinfo::System::new_all();
    for process in system.get_process_by_name("RobloxPlayerBeta.exe") {
        println!("Found another instance of Roblox opened, killing it...");
        process.kill(Signal::Kill);
    }
    for process in system.get_process_by_name("RobloxPlayerLauncher.exe") {
        println!("Found another instance of Roblox opened, killing it...");
        process.kill(Signal::Kill);
    }

    println!("Opening config.toml...");
    let mut config = log_fail!(utils::get_config(), "Error occurred while reading config.toml.");
    println!("Opened config.toml");
    
    // Extract Roblox path and save it to config, and replace URL Protocol command with rblx_rich_presence.exe
    let hkcr = RegKey::predef(enums::HKEY_CURRENT_USER);
    let rblx_reg = log_fail!(hkcr.open_subkey_with_flags(r"Software\Classes\roblox-player\shell\open\command", enums::KEY_ALL_ACCESS));
    let value: String = log_fail!(rblx_reg.get_value(""));
    
    if value.ends_with("RobloxPlayerLauncher.exe\" %1") {
        config.general.roblox = value[1..&value.len()-4].to_string();
        log_fail!(utils::set_config(&config));
    }
    log_fail!(rblx_reg.set_value("", &format!("\"{}\" \"%1\"", log_fail!(std::env::current_exe()).to_str().unwrap())));
    
    // Setup registry values for passing information
    // TODO: Find a more efficient way of doing it
    let (rblx_rp_reg, _) = log_fail!(hkcr.create_subkey(r"Software\rblx_rich_presence"));
    log_fail!(rblx_rp_reg.set_value("join_data", &""));
    log_fail!(rblx_rp_reg.set_value("join_key", &""));
    log_fail!(rblx_rp_reg.set_value("proceed", &"false"));
    
    let discord = log_fail!(Rustcord::init::<DiscordEventHandler>(
        "725360592570941490",
        true,
        None
    ));
    
    match env::args().nth(1) {
        Some(value) => {
            println!("Connecting to Roblox...");
            let rblx = roblox::Roblox::new()
            .with_roblosecurity(config.general.roblosecurity)
            .with_path(config.general.roblox)
            .with_url(value)
            .with_additional_info_from_request_type();
            
            println!("Launching Roblox...");
            log_fail!(rblx.launch());
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
            log_fail!(discord.update_presence(presence));
            utils::watch(discord, rblx, now);
        }
        None => {            
            let mut close = true;
            for _ in 0..20 {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let proceed: String = log_fail!(rblx_rp_reg.get_value("proceed"));
                if proceed == "true" {
                    log_fail!(rblx_rp_reg.set_value("proceed", &"false"));
                    close = false;
                    break;
                }
                discord.run_callbacks();
            }
            if close {
                println!("No pending task detected, closing...");
                std::process::exit(0);
            }
            println!("Connecting to Roblox...");
            let join_url: String = log_fail!(rblx_rp_reg.get_value("join_data"));
            let discord = log_fail!(Rustcord::init::<DiscordEventHandler>(
                "725360592570941490",
                true,
                None
            ));

            let rblx = roblox::Roblox::new()
                .with_roblosecurity(config.general.roblosecurity)
                .with_path(config.general.roblox)
                .with_url(join_url)
                .with_additional_info_from_request_type();
            println!("Launching Roblox...");
            log_fail!(rblx.launch());
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
            log_fail!(rblx_rp_reg.set_value("join_data", &""));
            log_fail!(rblx_rp_reg.set_value("join_key", &""));

            log_fail!(discord.update_presence(presence));
            utils::watch(discord, rblx, now);
        }
    }

    println!("Closing program...");
}
