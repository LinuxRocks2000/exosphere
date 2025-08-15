use bevy_ecs_macros::Resource;
use common::PlayerId;
// config file parser
use serde_derive::{Deserialize, Serialize};

use crate::placer::Placer;

#[derive(Deserialize, Serialize)]
pub struct TimesConfig {
    pub wait_period: u16,
    pub strategy_period: u16,
    pub play_period: u16,
}

#[derive(Deserialize, Serialize)]
pub struct PlayerCountConfig {
    pub min_players: u16,
    pub max_players: u16,
}

#[derive(Deserialize, Serialize)]
pub struct InitItemDescriptor {
    pub tp: String,
    pub x: f32,
    pub y: f32,
    pub a: Option<f32>,
}

impl InitItemDescriptor {
    pub(crate) fn place(
        &self,
        placer: &mut Placer,
        root_x: f32,
        root_y: f32,
        root_a: f32,
        client: PlayerId,
        slot: u8,
    ) {
        let x = self.x + root_x;
        let y = self.y + root_y;
        let a = self.a.unwrap_or_default() * std::f32::consts::PI / 180.0 + root_a;
        match self.tp.as_ref() {
            "basic_fighter" => {
                placer.basic_fighter_free(x, y, a, client, slot);
            }
            "castle" => {
                placer.castle(x, y, client, slot);
            }
            "sniper" => {
                placer.sniper_free(x, y, a, client, slot);
            }
            "lasernode_small" => {
                placer.small_lasernode_free(x, y, client, slot);
            }
            _ => {
                panic!("unknown placer type code {}", self.tp);
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BoardConfig {
    pub width: f32,
    pub height: f32,
    pub things: Vec<InitItemDescriptor>,
}

#[derive(Deserialize, Serialize)]
pub struct ClientSetupConfig {
    pub money: u32,
    pub area: Vec<InitItemDescriptor>,
}

#[derive(Deserialize, Serialize, Resource)]
pub struct Config {
    pub game_address: String,
    pub times: TimesConfig,
    pub counts: PlayerCountConfig,
    pub board: BoardConfig,
    pub client_setup: ClientSetupConfig,
    pub game_type: String, // "io" or "normal"
}

pub fn read_config() -> Option<Config> {
    let args = std::env::args().collect::<Vec<String>>();
    let file_name = if let Some(f) = args.get(1) {
        f
    } else {
        "config.json"
    };
    let file = std::fs::File::open(file_name).ok()?;
    serde_json::from_reader(file).ok()
}

pub fn read_config_or_default() -> Config {
    if let Some(config) = read_config() {
        config
    } else {
        println!(
            "WARNING: no config file specified or config file couldn't be opened; using defaults!"
        );
        Config::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        // a typical 2-player small game with no map obstructions. sensible default.
        // DON'T CHANGE THIS! write json overrides instead
        Self {
            game_address: "0.0.0.0:3000".to_string(),
            game_type: "normal".to_string(),
            times: TimesConfig {
                wait_period: 300,
                play_period: 600,
                strategy_period: 600,
            },
            counts: PlayerCountConfig {
                min_players: 2,
                max_players: 10,
            },
            board: BoardConfig {
                width: 3000.0,
                height: 3000.0,
                things: vec![],
            },
            client_setup: ClientSetupConfig {
                money: 100,
                area: vec![
                    InitItemDescriptor {
                        tp: "basic_fighter".to_string(),
                        x: 200.0,
                        y: 0.0,
                        a: Some(0.0),
                    },
                    InitItemDescriptor {
                        tp: "basic_fighter".to_string(),
                        x: -200.0,
                        y: 0.0,
                        a: Some(180.0),
                    },
                    InitItemDescriptor {
                        tp: "basic_fighter".to_string(),
                        x: 0.0,
                        y: -200.0,
                        a: Some(270.0),
                    },
                    InitItemDescriptor {
                        tp: "basic_fighter".to_string(),
                        x: 0.0,
                        y: 200.0,
                        a: Some(90.0),
                    },
                    InitItemDescriptor {
                        tp: "castle".to_string(),
                        x: 0.0,
                        y: 0.0,
                        a: Some(0.0),
                    },
                ],
            },
        }
    }
}
