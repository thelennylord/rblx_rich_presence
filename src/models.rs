use crate::{roblox, utils};
use base64::decode;
use rustcord::{EventHandlers, User};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::str;
use winreg::{enums, RegKey};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub general: GeneralConfig,
    pub presence: PresenceConfig,
}

impl Config {
    pub fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            presence: PresenceConfig::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeneralConfig {
    pub roblox: String,
    pub roblosecurity: String,
}

impl GeneralConfig {
    pub fn default() -> Self {
        GeneralConfig {
            roblox: String::default(),
            roblosecurity: String::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PresenceConfig {
    pub show_username: bool,
    pub show_game: bool,
    pub show_presence: bool,
    pub update_interval: u64,
}

impl PresenceConfig {
    pub fn default() -> Self {
        PresenceConfig {
            show_username: false,
            show_presence: true,
            show_game: true,
            update_interval: 30
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct DiscordJoinAccept {
    pub place_id: u64,
    pub job_id: String,
}

pub struct DiscordEventHandler;

impl EventHandlers for DiscordEventHandler {
    fn ready(user: User) {
        println!(
            "Connected to Discord as {}#{}",
            user.username, user.discriminator
        );
    }
    fn join_game(secret: &str) {
        // TODO: Handle join game from Discord
        let buff = match decode(&secret) {
            Ok(a) => a,
            Err(err) => {
                println!("Error: {:#?}", err);
                utils::pause();
                std::process::exit(1);
            }
        };
        let json_str = str::from_utf8(&buff).unwrap();
        let data: DiscordJoinAccept = serde_json::from_str(&json_str).unwrap();
        let config = match utils::get_config() {
            Ok(value) => value,
            Err(value) => {
                println!(
                    "Error occurred while reading config.toml\n\nError: {:#?}",
                    value
                );
                utils::pause();
                std::process::exit(1);
            }
        };
        let rblx = roblox::Roblox::new()
            .with_roblosecurity(config.general.roblosecurity)
            .with_path(config.general.roblox);

        if !rblx.verify_roblosecurity() {
            println!("ERROR: Invalid .ROBLOSECURITY cookie in config.toml");
            utils::pause();
            std::process::exit(0);
        }
        
        let join_data = roblox::RobloxJoinData {
            user_id: 0,
            username: String::new(),
            launch_mode: "play".to_string(),
            game_info: rblx.generate_ticket().ok_or("Could not generate auth ticket").unwrap(),
            request: "RequestGameJob".to_string(),
            launch_time:  0,
            access_code: String::default(),
            link_code: String::default(),
            place_launcher_url: format!("https://assetgame.roblox.com/game/PlaceLauncher.ashx?request=RequestGameJob&browserTrackerId=0&placeId={}&gameId={}&isPlayTogetherGame=false", &data.place_id, &data.job_id),
            is_play_together: "false".to_string(),
            place_id: data.place_id,
            place_name: String::new(),
            job_id: data.job_id.to_string(),
            friend_user_id: 0,
            browser_tracker_id: 0,
            roblox_locale: "en_us".to_string(),
            game_locale: "en_us".to_string()
        };
        // TODO: come up with a better way to pass data
        let hkcr = RegKey::predef(enums::HKEY_CURRENT_USER);
        let (rblx_rp_reg, _) = hkcr.create_subkey(r"Software\rblx_rich_presence").unwrap();
        rblx_rp_reg.set_value("join_data", &join_data.as_url()).unwrap();
        rblx_rp_reg.set_value("join_key", &secret.to_string()).unwrap();
        rblx_rp_reg.set_value("proceed", &"true").unwrap();
    }
}
