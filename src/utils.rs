use crate::models::{PartialConfig, Config};
use crate::roblox::Roblox;
use crate::tray_menu;
use base64::encode;
use rustcord::{Rustcord, RichPresenceBuilder};
use winreg::{enums, RegKey};
use sysinfo::{System, SystemExt};
use std::{
    fs::File, io::{stdin, stdout, Read, Write}, sync::{mpsc, Arc, Mutex},
    thread, path::{Path, PathBuf}, time, env, time::SystemTime
};
use winapi::um::{
    wincon::{
        SetConsoleTextAttribute, FOREGROUND_RED, FOREGROUND_GREEN, FOREGROUND_BLUE,
    },
    processenv::GetStdHandle, winbase::STD_OUTPUT_HANDLE 
};

pub fn pause() {
    let mut stdout = stdout();
    stdout.write_all(b"\nPress any key to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read_exact(&mut [0]).unwrap();
}

fn update_presence(config: &Config, discord: &Rustcord, rblx: &Mutex<Roblox>, now: SystemTime) {
    let mut rblx = rblx.lock().unwrap();
    let updated = rblx.update_game_info();
    
    let large_image_text = if config.presence.show_username {
        format!("Playing ROBLOX as {}", rblx.join_data.username)
    } else {
        "Playing ROBLOX".to_string()
    };

    let place_name: &str = if config.presence.show_game {
        if rblx.join_data.place_name == "Unknown Game" {
            ""
        } else {
            rblx.join_data.place_name.as_str()
        }
    } else {
        ""
    };

    if config.presence.show_presence {
        // check the game the user is in
        if !updated {
            warn!("Couldn't find the game you're in, so Discord join invites are disabled until it's found");
            let presence = RichPresenceBuilder::new()
                .state("In a game")
                .details(place_name)
                .large_image_key("logo")
                .large_image_text(&large_image_text)
                .small_image_key("play_status")
                .small_image_text(place_name)
                .start_time(now)
                .build();
            discord.update_presence(presence).unwrap();
            return;
        }

        if rblx.server_hidden {
            let presence = RichPresenceBuilder::new()
                .state("In a game")
                .details(place_name)
                .large_image_key("logo")
                .large_image_text(&large_image_text)
                .small_image_key("play_status")
                .small_image_text(place_name)
                .start_time(now)
                .build();
            discord.update_presence(presence).unwrap();
            return;
        }

        // usual checks
        let server_info = rblx.get_server_info();

        if server_info.is_none() {
            warn!("Couldn't find the server you're in, so Discord join invites are disabled until it's found");
            let presence = RichPresenceBuilder::new()
                .state("In a game")
                .details(place_name)
                .large_image_key("logo")
                .large_image_text(&large_image_text)
                .small_image_key("play_status")
                .small_image_text(place_name)
                .start_time(now)
                .build();
            discord.update_presence(presence).unwrap();
        } else {
            let server_info = server_info.unwrap();
            let join_secret = format!(
                "{{\"place_id\": {}, \"job_id\": \"{}\"}}",
                &rblx.join_data.place_id, &rblx.join_data.job_id
            );
            let presence = RichPresenceBuilder::new()
                .state("In a game")
                .details(place_name)
                .large_image_key("logo")
                .large_image_text(&large_image_text)
                .small_image_key("play_status")
                .small_image_text(place_name)
                .party_id(&rblx.join_data.job_id)
                .start_time(now)
                .party_size(server_info.playing.unwrap_or(1))
                .party_max(server_info.max_players)
                .join_secret(&encode(join_secret))
                .build();
            discord.update_presence(presence).unwrap();
        }

    } else {
        discord.clear_presence();
    }
}

pub fn watch(disc: rustcord::Rustcord, rblx: Roblox, now: SystemTime) {
    let disc = Arc::new(disc);
    let rblx = Arc::new(Mutex::new(rblx));

    // Tray menu thread
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || unsafe {
        tray_menu::start(tx);
    });
    let tray_tx = rx.recv().unwrap();

    // Discord Rich Presence thread
    let thread_disc = disc.clone();
    let thread_rblx = rblx.clone();
    thread::spawn(move || loop {
        let config = get_config().unwrap();
        update_presence(&config, &thread_disc, &thread_rblx, now);
        thread::sleep(time::Duration::from_secs(config.presence.update_interval));
    });

    // Config watcher thread
    let config_path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("config.toml");
    let prev_time = Arc::new(Mutex::new(0 as u64));
    let thread2_disc = disc.clone();
    let thread2_rblx = rblx;
    let thread_prev_time = Arc::clone(&prev_time);
    thread::spawn(move || loop {
        let metadata = std::fs::metadata(&config_path).unwrap();
        if let Ok(time) = metadata.modified() {
            let since_epoch = time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let mut prev_time = thread_prev_time.lock().unwrap();
            if *prev_time == 0 {
                *prev_time = since_epoch;
            } else if since_epoch != *prev_time {
                let config = get_config().unwrap();
                update_presence(&config, &thread2_disc, &thread2_rblx, now);
                *prev_time = since_epoch;
                log!("config.toml updated; updating rich presence");
            }
        }
        thread::sleep(time::Duration::from_secs(1));
    });

    // Main loop
    let mut tries: u8 = 0;
    let mut system = System::new_all();
    loop {
        system.refresh_all();
        let duration = time::Duration::from_millis(500);
        let rblx_not_found: bool = system.get_process_by_name("RobloxPlayerBeta.exe").is_empty();
        
        thread::sleep(duration);
        disc.run_callbacks();
        
        if rblx_not_found {
            if tries < 15 {
                // Check whether Roblox is updating by starting a loop which will check whether the launcher is opened
                let mut updated: bool = false;
                let mut update_msg_shown: bool = false;
                let mut config = get_config().unwrap();
                loop {
                    system.refresh_all();
                    
                    let launcher_name: &str = Path::new(&config.general.launcher).file_name().unwrap().to_str().unwrap();
                    if system.get_process_by_name(launcher_name).is_empty() {
                        // Roblox is not updating, break out of the loop
                        break;
                    }

                    if !update_msg_shown {
                        // We want the message to be displayed only once
                        update_msg_shown = true;
                        log!("Found Roblox launcher open; Roblox could be updating..."); 
                        log!("Waiting for Roblox to finish updating...");
                    }
                    updated = true;
                    thread::sleep(duration);
                }

                if updated {
                    log!("Roblox has finished updating");
                    
                    
                    // Registry values have been reset, so revert them back
                    let hkcr = RegKey::predef(enums::HKEY_CURRENT_USER);
                    let rblx_reg = hkcr.open_subkey_with_flags(
                        r"Software\Classes\roblox-player\shell\open\command",
                        enums::KEY_ALL_ACCESS,
                    ).unwrap();
                    let value: String = rblx_reg.get_value("").unwrap();
                    let exe_dir: PathBuf = env::current_exe().unwrap();
                    let exe_name: &str = exe_dir.file_name().unwrap().to_str().unwrap();
                    if !value.ends_with(&format!("{}\" \"%1\"", exe_name)) {
                        config.general.launcher = value[1..&value.len()-4].to_string();
                        set_config(&config).unwrap();
                    }

                    rblx_reg.set_value("", &format!("\"{}\" \"%1\"", env::current_exe().unwrap().to_str().unwrap())).unwrap();
                }

                tries += 1;
                warn!("Could not find Roblox, trying again...");
            } else {
                warn!("Could not find Roblox, shutting down...");
                break;
            }
            continue;
        }
        tries = 16;
    }

    disc.clear_presence();

    tray_tx.send(true).unwrap();
    rx.recv().unwrap();
    log!("Roblox has shut down");
}

pub fn get_config() -> Result<Config, std::io::Error> {
    let dir = env::current_exe()?;
    let config_path = dir.parent().unwrap().join("config.toml");
    if !config_path.exists() {
        warn!("Could not find config.toml; creating...");
        set_config(&Config::default()).unwrap();
    };

    let mut file = File::open(config_path).unwrap();
    let mut buffer: String = String::new();

    file.read_to_string(&mut buffer).unwrap();

    // Autofill the config with missing fields along with their default values
    let partial_config: PartialConfig = toml::from_str(&buffer)?;
    let has_all_some: bool = partial_config.has_all_some();
    let config: Config = partial_config.to_complete();
    if !has_all_some {
        set_config(&config)?;
    }

    Ok(config)
}

pub fn set_config(config: &Config) -> Result<(), std::io::Error> {
    let config_toml = toml::to_string_pretty(&config).unwrap();
    let mut file = File::create(
        env::current_exe()?
            .parent()
            .unwrap()
            .join("config.toml")
    ).unwrap();
    file.write_all(config_toml.as_bytes())?;
    Ok(())
}