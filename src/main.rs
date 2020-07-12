#[macro_use]
mod utils;
mod models;
mod roblox;
mod tray_menu;

use base64::encode;
use models::*;
use rustcord::{RichPresenceBuilder, Rustcord};
use std::env;
use std::time::SystemTime;
use sysinfo::{ProcessExt, Signal, SystemExt};
use winreg::{enums, RegKey};

fn main() {
    // Close roblox if open

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
    let mut config = log_fail!(utils::get_config(), "Error occurred while reading config.toml");
    println!("Opened config.toml");

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

            let join_secret = format!(
                "{{\"place_id\": {}, \"job_id\": \"{}\"}}",
                &join_data.place_id, &join_data.job_id
            );
            let now = SystemTime::now();
            let presence = RichPresenceBuilder::new()
                .state("In a game")
                .details(&join_data.place_name)
                .large_image_key("logo")
                .large_image_text("Playing ROBLOX")
                .small_image_key("play_status")
                .small_image_text(&join_data.place_name)
                .party_id(&join_data.job_id)
                .start_time(now)
                .join_secret(&encode(join_secret))
                .build();
            log_fail!(discord.update_presence(presence));
            utils::watch(discord, rblx, now);
        }
        None => {
            let hkcr = RegKey::predef(enums::HKEY_CURRENT_USER);
            let (rblx_rp_reg, _) = log_fail!(hkcr.create_subkey(r"Software\rblx_rich_presence"));
            let rblx_reg = log_fail!(hkcr.open_subkey_with_flags(r"Software\Classes\roblox-player\shell\open\command", enums::KEY_ALL_ACCESS));
            let value: String = log_fail!(rblx_reg.get_value(""));
            
            if value.ends_with("RobloxPlayerLauncher.exe\" %1") {
                config.general.roblox = value[1..&value.len()-4].to_string();
                log_fail!(utils::set_config(&config));
            }
            log_fail!(rblx_reg.set_value("",&format!("\"{}\" \"%1\"",log_fail!(std::env::current_exe()).to_str().unwrap())));
            
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
            let join_secret: String = log_fail!(rblx_rp_reg.get_value("join_key"));
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
                .party_id(&join_data.job_id)
                .start_time(now)
                .join_secret(&join_secret)
                .build();
            log_fail!(rblx_rp_reg.set_value("join_data", &""));
            log_fail!(rblx_rp_reg.set_value("join_key", &""));

            log_fail!(discord.update_presence(presence));
            utils::watch(discord, rblx, now);
        }
    }

    println!("Closing program...");
}
