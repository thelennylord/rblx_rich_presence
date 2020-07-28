use crate::models::Config;
use crate::roblox::Roblox;
use crate::tray_menu;
use base64::encode;
use rustcord::RichPresenceBuilder;
use rustcord::Rustcord;
use std::borrow::Cow;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use std::time::SystemTime;
use winreg::{enums, RegKey};
use sysinfo::{System, SystemExt};

pub fn pause() {
    let mut stdout = stdout();
    stdout.write_all(b"\nPress Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read_exact(&mut [0]).unwrap();
}

fn update_presence(config: &Config, discord: &Rustcord, rblx: &Mutex<Roblox>, now: SystemTime) {
    let mut rblx = rblx.lock().unwrap();
    let large_image_text = if config.presence.show_username {
        Cow::Owned(format!("Playing ROBLOX as {}", rblx.join_data.username))
    } else {
        Cow::Borrowed("Playing ROBLOX")
    };

    if config.presence.show_presence {
        // check the game the user is in
        let updated = rblx.update_game_info();
        if updated.is_none() {
            println!("WARN: Couldn't find the game you're in, so Discord join invites are disabled until it's found");
            let presence = RichPresenceBuilder::new()
                .state("In a game")
                .details(&rblx.join_data.place_name)
                .large_image_key("logo")
                .large_image_text(&large_image_text)
                .small_image_key("play_status")
                .small_image_text(&rblx.join_data.place_name)
                .start_time(now)
                .build();
            crate::log_fail!(discord.update_presence(presence));
            return;
        }

        if rblx.server_hidden {
            let presence = RichPresenceBuilder::new()
                .state("In a game")
                .details(&rblx.join_data.place_name)
                .large_image_key("logo")
                .large_image_text(&large_image_text)
                .small_image_key("play_status")
                .small_image_text(&rblx.join_data.place_name)
                .start_time(now)
                .build();
            crate::log_fail!(&discord.update_presence(presence));
            return;
        }

        // usual checks
        let server_info = rblx.get_server_info();

        if server_info.is_none() {
            println!("WARN: Couldn't find the server you're in, so Discord join invites are disabled until it's found");
            let presence = RichPresenceBuilder::new()
                .state("In a game")
                .details(&rblx.join_data.place_name)
                .large_image_key("logo")
                .large_image_text(&large_image_text)
                .small_image_key("play_status")
                .small_image_text(&rblx.join_data.place_name)
                .start_time(now)
                .build();
            crate::log_fail!(&discord.update_presence(presence));
        } else {
            let server_info = server_info.unwrap();
            let join_secret = format!(
                "{{\"place_id\": {}, \"job_id\": \"{}\"}}",
                &rblx.join_data.place_id, &rblx.join_data.job_id
            );
            let presence = RichPresenceBuilder::new()
                .state("In a game")
                .details(&rblx.join_data.place_name)
                .large_image_key("logo")
                .large_image_text(&large_image_text)
                .small_image_key("play_status")
                .small_image_text(&rblx.join_data.place_name)
                .party_id(&rblx.join_data.job_id)
                .start_time(now)
                .party_size(server_info.playing.unwrap_or(1))
                .party_max(server_info.max_players)
                .join_secret(&encode(join_secret))
                .build();
            crate::log_fail!(&discord.update_presence(presence));
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
        let config = crate::log_fail!(get_config());
        update_presence(&config, &thread_disc, &thread_rblx, now);
        thread::sleep(time::Duration::from_secs(config.presence.update_interval));
    });

    // Config watcher thread
    let config_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("config.toml");
    let prev_time = Arc::new(Mutex::new(0 as u64));
    let thread2_disc = disc.clone();
    let thread2_rblx = rblx;
    let thread_prev_time = Arc::clone(&prev_time);
    thread::spawn(move || loop {
        let metadata = crate::log_fail!(std::fs::metadata(&config_path));
        if let Ok(time) = metadata.modified() {
            let since_epoch = time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let mut prev_time = thread_prev_time.lock().unwrap();
            if *prev_time == 0 {
                *prev_time = since_epoch;
            } else if since_epoch != *prev_time {
                let config = crate::log_fail!(get_config());
                update_presence(&config, &thread2_disc, &thread2_rblx, now);
                *prev_time = since_epoch;
                println!("config.toml updated; updating rich presence");
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
                loop {
                    system.refresh_all();

                    if system.get_process_by_name("RobloxPlayerLauncher.exe").is_empty() {
                        // Roblox is not updating, break out of the loop
                        break;
                    }

                    if !update_msg_shown {
                        // We want the message to be displayed only once
                        update_msg_shown = true;
                        println!("Found Roblox launcher open; Roblox could be updating...\nWaiting for Roblox to finish updating...");
                    }
                    updated = true;
                    thread::sleep(duration);
                }

                if updated {
                    println!("Roblox has finished updating");
                    
                    // Registry values have been reset, so revert them back
                    let hkcr = RegKey::predef(enums::HKEY_CURRENT_USER);
                    let rblx_reg = crate::log_fail!(hkcr.open_subkey_with_flags(
                        r"Software\Classes\roblox-player\shell\open\command",
                        enums::KEY_SET_VALUE,
                    ));

                    crate::log_fail!(rblx_reg.set_value("", &format!("\"{}\" \"%1\"", std::env::current_exe().unwrap().to_str().unwrap())));
                }

                tries += 1;
                println!("Could not find Roblox, trying again...");
            } else {
                println!("Could not find Roblox, shutting down...");
                break;
                //std::process::exit(0);
            }
            continue;
        }
        tries = 16;
    }

    disc.clear_presence();

    crate::log_fail!(tray_tx.send(true));
    crate::log_fail!(rx.recv());
    println!("Roblox has shut down");
}

pub fn get_config() -> Result<Config, std::io::Error> {
    let dir = std::env::current_exe()?;
    let config_path = dir.parent().unwrap().join("config.toml");
    if !config_path.exists() {
        println!("WARN: Could not find config.toml; creating...");
        crate::log_fail!(set_config(&Config::default()));
    };

    let err_msg = format!("ERROR: Could not find config.toml at {:#?}", &dir.parent());
    let mut file = crate::log_fail!(
        File::open(config_path),
        err_msg
    );

    let mut buffer: String = String::new();

    crate::log_fail!(
        file.read_to_string(&mut buffer),
        "ERROR: Could not read config.toml"
    );
    let config: Config = toml::from_str(&buffer)?;

    Ok(config)
}

pub fn set_config(config: &Config) -> Result<(), std::io::Error> {
    let config_toml = crate::log_fail!(toml::to_string_pretty(&config));
    let mut file = crate::log_fail!(
        File::create(
            std::env::current_exe()?
                .parent()
                .unwrap()
                .join("config.toml")
        ),
        "ERROR: Could not write to config.toml"
    );
    file.write_all(config_toml.as_bytes())?;
    Ok(())
}

#[macro_export]
macro_rules! log_fail {
    ($res:expr) => {
        match $res {
            Ok(value) => value,
            Err(value) => {
                println!("An error ocurred:\n {:#?}", value);
                crate::utils::pause();
                std::process::exit(0);
            }
        }
    };
    ($res:expr, $custom:tt) => {
        match $res {
            Ok(value) => value,
            Err(_) => {
                println!("An error ocurred:\n {}", $custom);
                crate::utils::pause();
                std::process::exit(0);
            }
        }
    };
}
