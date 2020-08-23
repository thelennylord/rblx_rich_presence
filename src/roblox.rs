use crate::utils::pause;
use crate::utils::{get_config, set_config};
use reqwest::header;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Command;
use std::path::Path;
use std::thread;
use std::time;
use url::Url;
use winreg::{enums, RegKey};
use reqwest::blocking::{Client, ClientBuilder};

pub struct Roblox {
    pub join_data: RobloxJoinData,
    pub path: String,
    pub server_hidden: bool,
    roblosecurity: String,
    skip_update: bool,
}

impl Roblox {
    pub fn new() -> Self {
        Self {
            path: "None".to_string(),
            join_data: RobloxJoinData::default(),
            server_hidden: false,
            roblosecurity: String::default(),
            skip_update: true
        }
    }

    /// Launches Roblox with the saved game data
    pub fn launch(&self) -> Result<(), std::io::Error> {
        let hkcr = RegKey::predef(enums::HKEY_CURRENT_USER);
        let rblx_reg = hkcr.open_subkey_with_flags(
            r"Software\Classes\roblox-player\shell\open\command",
            enums::KEY_SET_VALUE,
        )?;
        // prevents roblox from installing again, which we don't want
        rblx_reg.set_value("", &format!("\"{}\" %1", &self.path))?;

        // spawn Roblox Launcher
        let mut rblx_launcher = Command::new(&self.path);
        &rblx_launcher.arg(&self.join_data.as_url());
        if !rblx_launcher.status()?.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                format!("{} was interrupted", Path::new(&self.path).file_name().unwrap().to_str().unwrap()),
            ));
        }
        loop {
            let hkcr = RegKey::predef(enums::HKEY_CURRENT_USER);
            let rblx_reg = hkcr.open_subkey_with_flags(
                r"Software\Classes\roblox-player\shell\open\command",
                enums::KEY_SET_VALUE,
            )?;
            match rblx_reg.set_value(
                "",
                &format!(
                    "\"{}\" \"%1\"",
                    std::env::current_exe().unwrap().to_str().unwrap()
                ),
            ) {
                Ok(value) => value,
                Err(_) => {
                    println!("Error occurred while setting a few things up; trying again...");
                    thread::sleep(time::Duration::from_secs(1));
                    continue;
                }
            }
            break;
        }
        Ok(())
    }

    /// Launch Roblox Player directly without going through the Roblox Launcher
    //TODO: direct_launch implementation
    #[allow(dead_code)]
    pub fn direct_launch() {}

    pub fn get_join_data(&self) -> &RobloxJoinData {
        &self.join_data
    }

    #[allow(dead_code)]
    pub fn with_join_data(mut self, gd: RobloxJoinData) -> Self {
        self.join_data = gd;
        self
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.join_data = self.parse_url(url);
        self
    }

    /// Sets a custom Roblox launch path
    /// Default launch path is taken from registry
    pub fn with_path(mut self, path: String) -> Self {
        self.path = path;
        self
    }

    pub fn with_roblosecurity(mut self, roblosecurity: String) -> Self {
        self.roblosecurity = roblosecurity;
        self
    }

    /// Checks whether the .ROBLOSECURITY is valid or not
    /// Returns true if valid, else returns false.
    pub fn verify_roblosecurity(&self) -> bool {
        let client: Client = ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let res = client.get("https://www.roblox.com/mobileapi/userinfo")
            .header(header::COOKIE, format!(
                ".ROBLOSECURITY={};path=/;domain=.roblox.com;",
                self.roblosecurity
            ))
            .send()
            .unwrap();
        
        if res.status().is_success() {
            return true;
        }
        false
    }

    /// Uses Roblox Authentication Ticket to get .ROLOSECURITY
    pub fn generate_and_save_roblosecurity(&self) {
        let client = Client::new();
        let mut body = HashMap::new();
        body.insert("authenticationTicket", &self.join_data.game_info);
        let res = client.post("https://auth.roblox.com/v1/authentication-ticket/redeem")
            .header(header::USER_AGENT, "RobloxStudio/WinInet")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "application/json")
            .header("RBXAuthenticationNegotiation", "https://www.roblox.com")
            .json(&body)
            .send()
            .unwrap();
        
        if res.status().is_success() {
            let set_cookie_headers = res.headers().get(header::SET_COOKIE);
            let raw_roblosecurity: &str = set_cookie_headers.iter().next().unwrap().to_str().unwrap();
            let roblosecurity: &str = &raw_roblosecurity[15..raw_roblosecurity.find(';').unwrap()];
            let mut config = get_config().unwrap();
            
            config.general.roblosecurity = roblosecurity.to_string();
            set_config(&config).unwrap();
        }
    }

    /// Used to generate an one time authorization ticket.
    /// This ticket can be used to join games as the authorized user.
    pub fn generate_ticket(&self) -> Option<String> {
        let client = Client::new();

        let mut x_csrf_token = String::default();
        let mut tries = 0;

        loop {
            if tries < 6 {
                tries += 1;
                let ticket_res = client
                    .post("https://auth.roblox.com/v1/authentication-ticket")
                    .header(header::REFERER, "https://www.roblox.com/games")
                    .header("x-csrf-token", &x_csrf_token)
                    .header(
                        header::COOKIE,
                        format!(
                            ".ROBLOSECURITY={};path=/;domain=.roblox.com;",
                            &self.roblosecurity
                        ),
                    )
                    .header(header::CONTENT_LENGTH, 0)
                    .header(header::HOST, "auth.roblox.com")
                    .send()
                    .unwrap();
                match ticket_res.status().as_u16() {
                    200 => {
                        let header_value = ticket_res
                            .headers()
                            .get("rbx-authentication-ticket")
                            .unwrap()
                            .to_str()
                            .unwrap();
                        return Some(header_value.to_string());
                    }
                    403 => {
                        x_csrf_token = match ticket_res.headers().get("x-csrf-token") {
                            Some(value) => {
                                let val = value.to_str().unwrap();
                                val.to_string()
                            }

                            None => {
                                continue;
                            }
                        };
                        continue;
                    }
                    _ => {
                        //println!("{}", ticket_res.status().as_u16());
                        //println!("{:#?}", ticket_res.headers());
                        //crate::utils::pause();
                        return None;
                    }
                }
            }
            break;
        }

        None
    }

    fn parse_url(&self, url: String) -> RobloxJoinData {
        let mut options: HashMap<&str, String> = HashMap::new();
        for substr in url.split("+") {
            if substr == "roblox-player:" {
                continue;
            }
            let pair: Vec<String> = substr.split(":").map(|x| x.to_string()).collect();
            let mut value =
                pair.get(1).ok_or("Failed to get an argument from url").unwrap().to_string();
            match pair.get(0).unwrap().as_str() {
                "launchmode" => {
                    options.insert("launchmode", value);
                }
                "gameinfo" => {
                    options.insert("gameinfo", value);
                }
                "launchtime" => {
                    options.insert("launchtime", value);
                }
                "placelauncherurl" => {
                    value = value.replace("%26", "&");
                    value = value.replace("%3A", ":");
                    value = value.replace("%2F", "/");
                    value = value.replace("%3D", "=");
                    value = value.replace("%3F", "?");
                    options.insert("placelauncherurl", value);
                }
                "browsertrackerid" => {
                    options.insert("browsertrackerid", value);
                }
                "robloxLocale" => {
                    options.insert("robloxLocale", value);
                }
                "gameLocale" => {
                    options.insert("gameLocale", value);
                }
                _ => {
                    continue;
                }
            }
        }
        let placelauncherurl: &str = options
            .get("placelauncherurl")
            .ok_or("Failed to get place launcher url")
            .unwrap();
        let parsed_url = Url::parse(placelauncherurl).unwrap();
        let query: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
        RobloxJoinData {
            user_id: 0,
            username: String::new(),
            launch_mode: options.get("launchmode").unwrap().to_string(),
            game_info: options.get("gameinfo").unwrap().to_string(),
            request: query.get("request").unwrap().to_string(),
            launch_time: options.get("launchtime").unwrap().parse::<u64>().unwrap(),
            access_code: query
                .get("accessCode")
                .unwrap_or(&"".to_string())
                .to_string(),
            link_code: query.get("linkCode").unwrap_or(&"".to_string()).to_string(),
            place_launcher_url: options.get("placelauncherurl").unwrap().to_string(),
            is_play_together: query
                .get("isPlayTogetherGame")
                .unwrap_or(&"false".to_string())
                .to_string(),
            place_id: query
                .get("placeId")
                .unwrap_or(&"0".to_string())
                .parse::<u64>()
                .unwrap(),
            place_name: String::new(),
            job_id: query.get("gameId").unwrap_or(&String::new()).to_string(),
            friend_user_id: query
                .get("userId")
                .unwrap_or(&"0".to_string())
                .parse::<u64>()
                .unwrap(),
            browser_tracker_id: options
                .get("browsertrackerid")
                .unwrap()
                .parse::<u64>()
                .unwrap(),
            roblox_locale: options.get("robloxLocale").unwrap().to_string(),
            game_locale: options.get("gameLocale").unwrap().to_string(),
        }
    }

    pub fn with_additional_info_from_request_type(mut self) -> Self {
        match &self.join_data.request.as_str() {
            &"RequestGame" | &"RequestGameJob" | &"RequestPrivateGame" => {
                let client = Client::new();
                loop {
                    let res = match client
                        .get(&self.join_data.place_launcher_url)
                        .header(reqwest::header::USER_AGENT, "Roblox/WinInet")
                        .header(
                            reqwest::header::COOKIE,
                            format!(
                                ".ROBLOSECURITY={};path=/;domain=.roblox.com;",
                                &self.roblosecurity
                            ),
                        )
                        .send()
                    {
                        Ok(value) => value,
                        Err(_) => {
                            println!("Could not connect to Roblox, trying again...");
                            continue;
                        }
                    };
                    if res.status().is_success() {
                        let data: RequestGameReponse = res.json().unwrap();
                        let job_id = match &data.job_id {
                            Some(value) => value,
                            None => {
                                println!("Error while getting job id: {:#?}", data);
                                pause();
                                std::process::exit(1);
                            }
                        };
                        if job_id.starts_with("Join") {
                            println!("Waiting for server...");
                            let duration = time::Duration::from_secs(3);
                            thread::sleep(duration);
                            continue;
                        }

                        let join_url = data.join_script_url.unwrap().to_string();
                        let parsed_url = Url::parse(&join_url).unwrap();
                        let query: HashMap<String, String> =
                            parsed_url.query_pairs().into_owned().collect();

                        let json_data: Value =
                            serde_json::from_str(&query.get("ticket").unwrap()).unwrap();
                        self.join_data.username = match &json_data["UserName"] {
                            Value::String(value) => value.to_string(),
                            _ => {
                                println!("WARN: Could not get username");
                                "Player".to_string()
                            },
                        };

                        self.join_data.user_id = match &json_data["UserId"] {
                            Value::Number(value) =>  value.as_u64().unwrap(),
                            _ => {
                                println!("WARN: Could not get user ID of currently logged in user");
                                0
                            }
                        };

                        println!("Found an available server");
                        self.join_data.job_id = job_id.to_string();
                        break;
                    } else if res.status().is_server_error() {
                        println!("Server error occurred: {:#?}", res.status());
                        pause();
                        std::process::exit(1);
                    } else {
                        println!(
                            "Something happened while communitcating with the server: {:#?}",
                            res.status()
                        );
                        pause();
                        std::process::exit(1);
                    }
                }
            }
            &"RequestFollowUser" => {
                let client = Client::new();

                let resp = client
                    .get(&self.join_data.place_launcher_url)
                    .header(header::USER_AGENT, "Roblox/WinInet")
                    .header(
                        header::COOKIE,
                        format!(
                            ".ROBLOSECURITY={};path=/;domain=.roblox.com;",
                            &self.roblosecurity
                        ),
                    )
                    .send()
                    .unwrap();
                if resp.status().is_success() {
                    let data: RequestGameReponse = resp.json::<RequestGameReponse>().unwrap();
                    self.join_data.job_id = match &data.job_id {
                        Some(value) => value.to_string(),
                        None => {
                            match &data.status {
                                10 => {
                                    println!("Error while joining game: User is no longer in game",);
                                },
                                12 => {
                                    println!("Error while joining the game: You aren't authorized to join this game")
                                }
                                _ => {
                                    println!(
                                        "Error while joining game: {}",
                                        data.message
                                            .unwrap_or("Unknown error occurred".to_string())
                                            .to_string()
                                    );
                                }
                            }
                            pause();
                            std::process::exit(0);
                        }
                    };
                    let join_url =
                        data.join_script_url.ok_or("Failed to get join script url")
                            .unwrap()
                            .to_string();
                    let parsed_url = Url::parse(&join_url).unwrap();
                    let query: HashMap<String, String> =
                        parsed_url.query_pairs().into_owned().collect();

                    let json_data: Value = serde_json::from_str(&query.get("ticket").unwrap()).unwrap();
                    self.join_data.username = match &json_data["UserName"] {
                        Value::String(value) => value.to_string(),
                        _ => {
                            println!("WARN: Could not get username of currently logged in user");
                            "Player".to_string()
                        },
                    };

                    self.join_data.user_id = match &json_data["UserId"] {
                        Value::Number(value) =>  value.as_u64().unwrap(),
                        _ => {
                            println!("WARN: Could not get user ID of currently logged in user");
                            0
                        }
                    };

                    self.join_data.place_id = match &json_data["PlaceId"] {
                        Value::Number(value) => value.as_u64().unwrap(),
                        _ => 0,
                    };
                }
            }
            _ => {}
        }

        let res = reqwest::blocking::get(&format!(
            "https://api.roblox.com/Marketplace/ProductInfo?assetId={}",
            &self.join_data.place_id
        ))
        .unwrap();
        if res.status().is_success() {
            let data = res.text().unwrap();
            let json_data: Value = serde_json::from_str(&data).unwrap();
            self.join_data.place_name = match &json_data["Name"] {
                Value::String(value) => value.to_string(),
                _ => "Unknown Game".to_string(),
            }
        }

        self
    }

    pub fn get_server_info(&self) -> Option<RobloxServerData> {
        let mut next_cursor_page: String = String::default();
        let client = Client::new();

        // Get total number of servers
        let res = client
            .get(&format!(
                "https://www.roblox.com/games/getgameinstancesjson?placeId={}&startIndex=0",
                &self.join_data.place_id
            ))
            .header(
                reqwest::header::COOKIE,
                format!(
                    ".ROBLOSECURITY={};path=/;domain=.roblox.com;",
                    &self.roblosecurity
                ),
            )
            .header(
                reqwest::header::REFERER,
                format!("https://www.roblox.com/games/{}", &self.join_data.place_id),
            )
            .send()
            .unwrap();
        let data: serde_json::Value = serde_json::from_str(&res.text().unwrap()).unwrap();
        let total_servers = &data["TotalCollectionSize"].as_u64().unwrap();
        let max_tries: u64 = (total_servers / 100) + 1;

        // get list of all servers
        for _ in 0..max_tries {
            let url = format!("https://games.roblox.com/v1/games/{}/servers/Public?sortOrder=Asc&limit=100&cursor={}", &self.join_data.place_id, next_cursor_page);
            let resp = client.get(&url).send().unwrap();
            if resp.status().is_server_error() {
                panic!("Server error detected!");
            }

            let datap = serde_json::from_str::<RobloxServerListData>(&resp.text().unwrap()).unwrap();
            next_cursor_page = datap.next_page_cursor.unwrap_or(String::new());
            if datap.data.is_none() {
                continue;
            }

            for item in datap.data.unwrap() {
                if item.id == self.join_data.job_id {
                    return Some(item);
                }
            }
        }

        None
    }

    pub fn update_game_info(&mut self) -> Option<&mut Self> {
        // During the first call, roblox returns status 10 (user not in game) as our info on the api hasn't been updated yet.
        // So, skip the first call
        if self.skip_update {
            self.skip_update = false;
            return Some(self);
        }

        let client = Client::new();
        let url = format!("https://assetgame.roblox.com/game/PlaceLauncher.ashx?request=RequestFollowUser&browserTrackerId=0&userId={}", &self.join_data.user_id);
        let res = client.get(&url)
            .header(reqwest::header::USER_AGENT, "Roblox/WinInet")
            .header(
                reqwest::header::COOKIE,
                format!(
                    ".ROBLOSECURITY={};path=/;domain=.roblox.com;",
                    &self.roblosecurity
                ),
            )
            .send()
            .unwrap();
        
        if res.status().is_success() {
            let data: RequestGameReponse = res.json().unwrap();
            if let Some(job_id) = data.job_id {
                match data.status {
                    0 => {
                        // User is in a universe place which isn't the root place
                        let stripped_data: &str = &job_id[9..job_id.len()];
                        let split_data: Vec<&str> = stripped_data.split(";").collect();
                        let received_place_id: u64 = split_data[0].parse().unwrap();
                        
                        if self.join_data.place_id != received_place_id {
                            self.join_data.place_id = received_place_id;
                            let res = reqwest::blocking::get(&format!(
                                "https://api.roblox.com/Marketplace/ProductInfo?assetId={}",
                                &self.join_data.place_id
                            ))
                            .unwrap();
                            if res.status().is_success() {
                                let data = res.text().unwrap();
                                let json_data: Value = serde_json::from_str(&data).unwrap();
                                self.join_data.place_name = match &json_data["Name"] {
                                    Value::String(value) => value.to_string(),
                                    _ => "Unknown Game".to_string(),
                                }
                            }
                        }
                        
                        self.join_data.job_id = String::default();
                        self.server_hidden = true;
                        return Some(self);
                    },
                    12 => {
                        // User is not authorized to join their own game.
                        // This usually occurs when the user is playing on a VIP server
                        let stripped_data: &str = &job_id[9..job_id.len()];
                        let split_data: Vec<&str> = stripped_data.split(";").collect();
                        let received_place_id: u64 = split_data[0].parse().unwrap();

                        if self.join_data.place_id != received_place_id {
                            self.join_data.place_id = received_place_id;
                            let res = reqwest::blocking::get(&format!(
                                "https://api.roblox.com/Marketplace/ProductInfo?assetId={}",
                                &self.join_data.place_id
                            ))
                            .unwrap();
                            if res.status().is_success() {
                                let data = res.text().unwrap();
                                let json_data: Value = serde_json::from_str(&data).unwrap();
                                self.join_data.place_name = match &json_data["Name"] {
                                    Value::String(value) => value.to_string(),
                                    _ => "Unknown Game".to_string(),
                                }
                            }
                        }

                        if self.join_data.job_id != split_data[1] {
                            self.join_data.job_id = split_data[1].to_string();
                        }
                        
                        self.server_hidden = true;
                        return Some(self);
                    },
                    _ => {
                        if self.join_data.job_id != job_id {
                            self.join_data.job_id = job_id;
                        };
        
                        let join_url = data.join_script_url.ok_or("Failed to get join script url").unwrap().to_string();
                        let parsed_url = Url::parse(&join_url).unwrap();
                        let query: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
                        let json_data: Value = serde_json::from_str(&query.get("ticket").unwrap()).unwrap();
                        let place_id: u64 = match &json_data["PlaceId"] {
                            Value::Number(value) => value.as_u64().unwrap(),
                            _ => {
                                return None; 
                            }
                        };
                        
                        if self.join_data.place_id != place_id {
                            self.join_data.place_id = place_id;
                            let res = reqwest::blocking::get(&format!(
                                "https://api.roblox.com/Marketplace/ProductInfo?assetId={}",
                                &self.join_data.place_id
                            ))
                            .unwrap();
                            if res.status().is_success() {
                                let data = res.text().unwrap();
                                let json_data: Value = serde_json::from_str(&data).unwrap();
                                self.join_data.place_name = match &json_data["Name"] {
                                    Value::String(value) => value.to_string(),
                                    _ => "Unknown Game".to_string(),
                                }
                            }
                        }
                        
                        self.server_hidden = false;
                        return Some(self);
                    }
                };
            }
            return None;
        }

        None
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RobloxServerListData {
    previous_page_cursor: Option<String>,
    next_page_cursor: Option<String>,
    data: Option<Vec<RobloxServerData>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RobloxServerData {
    pub id: String,
    pub max_players: u32,
    pub playing: Option<u32>,
    pub fps: Option<f32>,
    pub ping: Option<u64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RobloxUserInfo {
    pub id: u64,
    pub username: String,
    pub avatar_uri: Option<String>,
    pub avatar_final: bool,
    pub is_online: bool
}

#[derive(Deserialize, Debug)]
pub struct RobloxJoinData {
    pub user_id: u64,
    pub username: String,
    pub launch_mode: String,
    pub game_info: String,
    pub request: String,
    pub launch_time: u64,
    pub access_code: String,
    pub link_code: String,
    pub place_launcher_url: String,
    pub is_play_together: String,
    pub place_id: u64,
    pub place_name: String,
    pub job_id: String,
    pub friend_user_id: u64,
    pub browser_tracker_id: u64,
    pub roblox_locale: String,
    pub game_locale: String,
}

impl RobloxJoinData {
    pub fn default() -> Self {
        Self {
            user_id: 0,
            username: "Player".to_string(),
            launch_mode: "play".to_string(),
            game_info: "abcd".to_string(),
            request: "https://www.roblox.com/".to_string(),
            launch_time: 0,
            link_code: String::default(),
            access_code: String::default(),
            place_launcher_url: "https://www.roblox.com".to_string(),
            is_play_together: "false".to_string(),
            place_id: 0,
            place_name: "A Roblox Game".to_string(),
            job_id: "abcdefghijklmnopqurstuvwxyz".to_string(),
            friend_user_id: 0,
            browser_tracker_id: 0,
            roblox_locale: "en_us".to_string(),
            game_locale: "en_us".to_string(),
        }
    }

    pub fn as_url(&self) -> String {
        let place_launcher_url: String;
        if self.request == "RequestPrivateGame" {
            place_launcher_url = format!(
                "https%3A%2F%2Fassetgame.roblox.com%2Fgame%2FPlaceLauncher.ashx%3Frequest%3DRequestPrivateGame%26browserTrackerId%3D{}%26placeId%3D{}%26accessCode%3D{}%26linkCode%3D{}",
                &self.browser_tracker_id, &self.place_id, &self.access_code, &self.link_code
            );
        } else {
            place_launcher_url = format!(
                "https%3A%2F%2Fassetgame.roblox.com%2Fgame%2FPlaceLauncher.ashx%3Frequest%3DRequestGameJob%26browserTrackerId%3D{}%26placeId%3D{}%26gameId%3D{}%26isPlayTogetherGame%3D{}",
                &self.browser_tracker_id, &self.place_id, &self.job_id, &self.is_play_together
            );
        }
        format!(
            "roblox-player:1+launchmode:{}+gameinfo:{}+launchtime:{}+placelauncherurl:{}+browsertrackerid:{}+robloxLocale:{}+gameLocale:{}",
            &self.launch_mode, &self.game_info, &self.launch_time, &place_launcher_url, &self.browser_tracker_id, &self.roblox_locale, &self.game_locale
        )
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestGameReponse {
    pub job_id: Option<String>,
    pub join_script_url: Option<String>,
    pub status: u8,
    pub message: Option<String>,
}
