use base64::decode;
use rustcord::{EventHandlers, User};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, str};
use winreg::{enums, RegKey};
use std::io::{stdout, Write};

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
    pub launcher: String,
    pub is_custom_launcher: bool,
    pub roblosecurity: String,
}

impl GeneralConfig {
    pub fn default() -> Self {
        GeneralConfig {
            launcher: String::default(),
            is_custom_launcher: false,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct PartialConfig {
    pub general: Option<PartialGeneralConfig>,
    pub presence: Option<PartialPresenceConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PartialGeneralConfig {
    pub launcher: Option<String>,
    pub is_custom_launcher: Option<bool>,
    pub roblosecurity: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PartialPresenceConfig {
    pub show_username: Option<bool>,
    pub show_game: Option<bool>,
    pub show_presence: Option<bool>,
    pub update_interval: Option<u64>,
}

impl PartialConfig {
    /// Transforms a PartialConfig into a Config
    /// Consumes self to return Config
    pub fn to_complete(self) -> Config {
        let mut config: Config = Config::default();

        // General Config
        if self.general.is_some() {
            let general = self.general.unwrap();
            if general.launcher.is_some() {
                config.general.launcher = general.launcher.unwrap();
            }

            if general.is_custom_launcher.is_some() {
                config.general.is_custom_launcher = general.is_custom_launcher.unwrap();
            }

            if general.roblosecurity.is_some() {
                config.general.roblosecurity = general.roblosecurity.unwrap();
            }
        }
        
        // Presence Config
        if self.presence.is_some() {
            let presence = self.presence.unwrap();
            if presence.show_username.is_some() {
                config.presence.show_username = presence.show_username.unwrap();
            }

            if presence.show_game.is_some() {
                config.presence.show_game = presence.show_game.unwrap();
            }

            if presence.show_presence.is_some() {
                config.presence.show_presence = presence.show_presence.unwrap();
            }

            if presence.update_interval.is_some() {
                config.presence.update_interval = presence.update_interval.unwrap();
            }
        }

        config
    }

    /// Returns boolean whether all the properties of a PartialConfig is Some
    pub fn has_all_some(&self) -> bool {
        // General Config
        if self.general.is_some() {
            let general = self.general.as_ref().unwrap();
            if general.launcher.is_none() {
                return false;
            }

            if general.is_custom_launcher.is_none() {
                return false;
            }

            if general.roblosecurity.is_none() {
                return false;
            }
        } else {
            return false;
        }
        
        // Presence Config
        if self.presence.is_some() {
            let presence = self.presence.as_ref().unwrap();
            if presence.show_username.is_none() {
                return false;
            }

            if presence.show_game.is_none() {
                return false;
            }

            if presence.show_presence.is_none() {
                return false;
            }

            if presence.update_interval.is_none() {
                return false;
            }
        } else {
            return false;
        }

        true
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
        log!(
            "Connected to Discord as {}#{}",
            user.username, user.discriminator
        );
    }
    fn join_game(secret: &str) {
        // TODO: Handle join game from Discord
        let buff = decode(&secret).unwrap();
        let json_str = str::from_utf8(&buff).unwrap();
        let data: DiscordJoinAccept = serde_json::from_str(&json_str).unwrap();

        // TODO: come up with a better way to pass data
        let hkcr = RegKey::predef(enums::HKEY_CURRENT_USER);
        let (rblx_rp_reg, _) = hkcr.create_subkey(r"Software\rblx_rich_presence").unwrap();
        rblx_rp_reg.set_value("place_id", &data.place_id).unwrap();
        rblx_rp_reg.set_value("job_id", &data.job_id).unwrap();
        rblx_rp_reg.set_value("join_key", &secret.to_string()).unwrap();
        rblx_rp_reg.set_value("proceed", &"true").unwrap();
    }
}