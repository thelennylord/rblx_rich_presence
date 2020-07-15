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
use sysinfo;
use sysinfo::{ProcessExt, Signal, SystemExt};

pub fn pause() {
    let mut stdout = stdout();
    stdout.write(b"\nPress Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
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
        if let None = updated {
            println!("WARN: Could not find the game you're, so Discord join invites are disabled until we find it");
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
        let srv_info = &rblx.get_server_info();

        if srv_info.id == "0" {
            println!("WARN: Could not find the server you are in, so Discord join invites are disabled until we find it");
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
            .party_size(srv_info.playing.unwrap_or(1))
            .party_max(srv_info.max_players)
            .join_secret(&encode(join_secret))
            .build();
        crate::log_fail!(&discord.update_presence(presence));
    } else {
        &discord.clear_presence();
    }
}

pub fn watch(disc: rustcord::Rustcord, rblx: Roblox, now: SystemTime) {
    let mut tries: u8 = 0;
    let disc = Arc::new(disc);
    let rblx = Arc::new(Mutex::new(rblx));

    // tray menu thread
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || unsafe {
        tray_menu::start(tx);
    });
    let tray_tx = rx.recv().unwrap();

    // discord rp thread
    let thread_disc = disc.clone();
    let thread_rblx = rblx.clone();
    thread::spawn(move || loop {
        let config = crate::log_fail!(get_config());
        update_presence(&config, &thread_disc, &thread_rblx, now);
        thread::sleep(time::Duration::from_secs(config.presence.update_interval));
    });
    // config thread
    let config_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("config.toml");
    let prev_time = Arc::new(Mutex::new(0 as u64));
    let thread2_disc = disc.clone();
    let thread2_rblx = rblx.clone();
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
    loop {
        let system = sysinfo::System::new_all();
        let mut rblx_found = false;
        let duration = time::Duration::from_millis(500);
        thread::sleep(duration);
        disc.run_callbacks();
        for _ in system.get_process_by_name("RobloxPlayerBeta.exe") {
            rblx_found = true;
            tries = 16;
        }
        if !rblx_found {
            if tries < 15 {
                // handle when roblox is updating
                loop {
                    let mut found_launcher = false;
                    for _ in system.get_process_by_name("RobloxPlayerLauncher.exe") {
                        found_launcher = true;
                    }
                    if !found_launcher {
                        continue;
                    }
                    // presume roblox launcher has finished updating
                    // we'll have to kill RobloxPlayerBeta.exe as original values will be passed
                    // we'll have to also set the url protocol again as roblox resets it after every update
                    let mut found_rblx = false;
                    loop {
                        for process in system.get_process_by_name("RobloxPlayerBeta.exe") {
                            found_rblx = true;
                            process.kill(Signal::Kill);
                        }
                        if found_rblx {
                            break;
                        };
                    }
                    break;
                }
                thread::sleep(duration);
                tries += 1;
                println!("Could not find Roblox, trying again...");
                continue;
            } else if tries != 16 {
                println!("Could not find Roblox, shutting down...");
                std::process::exit(0);
            }
            break;
        }
    }

    disc.clear_presence();

    crate::log_fail!(tray_tx.send(true));
    crate::log_fail!(rx.recv());
    println!("Roblox shut down");
}

pub fn get_config() -> Result<Config, std::io::Error> {
    let mut file = crate::log_fail!(
        File::open(
            std::env::current_exe()?
                .parent()
                .unwrap()
                .join("config.toml")
        ),
        "Could not find config.toml in exe directory"
    );
    //.expect("Could not find config.toml in exe directory");

    let mut buffer: String = String::new();

    crate::log_fail!(
        file.read_to_string(&mut buffer),
        "Could not read config.toml"
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
        "Could not write to config.toml"
    );
    file.write(config_toml.as_bytes())?;
    Ok(())
}

//#[allow(unused_macros)]
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
