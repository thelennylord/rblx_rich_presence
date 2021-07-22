#[macro_use]
mod logs;
mod utils;
mod models;
mod roblox;
mod tray_menu;

use models::*;
use rustcord::{RichPresenceBuilder, Rustcord};
use winreg::{enums, RegKey};
use serde_json::Value;
use winapi::um::{
    winuser::SetWindowTextW, 
    wincon::{
        GetConsoleWindow, SetConsoleTextAttribute, FOREGROUND_RED, FOREGROUND_GREEN, FOREGROUND_BLUE, FOREGROUND_INTENSITY
    }, 
    processenv::GetStdHandle, 
    winbase::STD_OUTPUT_HANDLE
};
use tray_menu::wide_str;
use std::{
    env, thread, panic, process::{exit, Command}, time::SystemTime,
    path::Path, io::{stdout, Write}
};
use json;


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
        warn!("Could not fetch version; Received status code {:#?}", res.status());
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

    
    log!("Loading config.toml...");
    let mut config = utils::get_config().unwrap();
    log!("Loaded config.toml");
    
    // Replace URL Protocol command with rblx_rich_presence.exe
    let hkcr = RegKey::predef(enums::HKEY_CURRENT_USER);
    let rblx_reg = hkcr.open_subkey_with_flags(r"Software\Classes\roblox-player\shell\open\command", enums::KEY_ALL_ACCESS).unwrap();
    rblx_reg.set_value("", &format!("\"{}\" \"%1\"", env::current_exe().unwrap().to_str().unwrap())).unwrap();
    
    // Get latest Roblox WindowsPlayer version and find Roblox path
    if !config.general.is_custom_launcher {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get("https://clientsettings.roblox.com/v2/client-version/WindowsPlayer")
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .unwrap();
        
        let roblox_ver = if res.status().is_success() {
            let data = res.text().unwrap();
            let parsed = json::parse(&data).unwrap();
            parsed["clientVersionUpload"].to_string()
        } else {
            "version-0".to_string()
        };
    
        let cmd = Command::new("cmd")
            .args(&["/C", "echo %localappdata%"])
            .output()
            .unwrap();
    
        let out = String::from_utf8_lossy(&cmd.stdout);
        let roblox_dir = Path::new(&out.trim_end()).join("Roblox");
        if !roblox_dir.exists() {
            error!("Could not find Roblox installation directory. Have you installed Roblox yet?");
        }
    
        let roblox_player = roblox_dir.join(format!("Versions/{}/RobloxPlayerLauncher.exe", roblox_ver));
        if roblox_player.exists() {
            config.general.launcher = roblox_player.to_string_lossy().into_owned();
        } else {
            for entry in roblox_dir.join("Versions").read_dir().unwrap() {
                if let Ok(entry) = entry {
                    let roblox_player = entry.path().join("RobloxPlayerLauncher.exe");
                    if roblox_player.exists() {
                        config.general.launcher = roblox_player.to_string_lossy().into_owned();
                        break;
                    }
                }
            }
        }
    
        utils::set_config(&config).unwrap();
    } else {
        log!("Skipping Roblox launcher check since custom launcher has been provided")
    }

    // Setup registry values for passing information
    let (rblx_rp_reg, _) = hkcr.create_subkey(r"Software\rblx_rich_presence").unwrap();
    rblx_rp_reg.set_value("place_id", &0u64).unwrap();
    rblx_rp_reg.set_value("job_id", &"").unwrap();
    rblx_rp_reg.set_value("join_key", &"").unwrap();
    rblx_rp_reg.set_value("proceed", &"false").unwrap();
    
    let discord = Rustcord::init::<DiscordEventHandler>(
        "725360592570941490",
        true,
        None
    ).unwrap();
    
    let mut join_url = String::default();
    let mut from_discord = false;
    match env::args().nth(1) {
        Some(value) => {
            join_url = value;
        }

        None => {            
            let mut close = true;
            for _ in 0..20 {
                discord.run_callbacks();
                
                let proceed: String = rblx_rp_reg.get_value("proceed").unwrap();
                if proceed == "true" {
                    rblx_rp_reg.set_value("proceed", &"false").unwrap();
                    join_url = rblx_rp_reg.get_value("join_data").unwrap();
                    close = false;
                    from_discord = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            if close {
                log!("No pending task detected, closing...");
                utils::pause();
                exit(0);
            }
        }
    }

    log!("Connecting to Roblox...");
    let mut rblx = roblox::Roblox::new()
        .with_roblosecurity(config.general.roblosecurity)
        .with_path(config.general.launcher);
        
    if from_discord {
        let mut join_data = roblox::RobloxJoinData::default();
        join_data.request = "RequestGameJob".to_string();
        join_data.place_id = rblx_rp_reg.get_value("place_id").unwrap();
        join_data.job_id = rblx_rp_reg.get_value("job_id").unwrap();
        join_data.generate_launch_url();
        rblx.join_data = join_data;

    } else {
        rblx.with_url(join_url);
        rblx.generate_and_save_roblosecurity();
    }

    if !rblx.verify_roblosecurity() {
        error!("Invalid .ROBLOSECURITY cookie in config.toml detected. Join a game to update the saved .ROBLOSECURITY cookie.");	
    }
    
    rblx.join_data.game_info = rblx.generate_ticket().or_else(|| {
        error!("Could not generate authentication ticket. Are Roblox servers down? Please try joining the game again.");
    }).unwrap();
    
    rblx.get_additional_info_from_request_type();


    log!("Launching Roblox...");
    if let Err(error) = rblx.launch() {
        error!("Could not launch Roblox; {}", error);
    };
    log!("Launched Roblox");
    log!("Loading rich presence...");

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

    log!("Closing rich presence...");
}