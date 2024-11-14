/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

use wasm_bindgen::prelude::*;
use common::comms::*;
use common::protocol::Protocol;
use common::types::PieceType;
use common::VERSION;
use common::types::Asset;
use num_traits::cast::FromPrimitive;
use std::collections::HashMap;
use common::pathfollower::PathFollower;


#[derive(Debug)]
#[wasm_bindgen]
pub enum KeyAcrossEvent {
    KeyUp,
    KeyDown
}

#[wasm_bindgen]
pub enum MouseAcrossEvent {
    Move,
    Up,
    Down
}


#[wasm_bindgen(module="/core.js")]
extern "C" {
    fn alert(s : &str);
    fn send_ws(data : Vec<u8>);
    fn get_input_value(name : &str) -> String;
    fn set_board_size(w : f32, h : f32);
    fn set_offset(x : f32, y : f32);
    fn set_time(tick : u16, stage : u16, phase_name : &str);
    fn ctx_stroke(width : f32, color : &str);
    fn ctx_outline_circle(x : f32, y : f32, rad : f32);
    fn set_money(amount : u32);
    fn render_background(fabbers_buf : &mut [f32], fabbers_count : usize, territories_buf : &mut [f32], territory_count : usize);
    fn ctx_draw_sprite(resource : &str, x : f32, y : f32, a : f32, w : f32, h : f32); // the difference between draw_sprite and draw_image is that draw_sprite includes an angle of rotation
    // and also maybe draw_image doesn't exist...
}


fn send(message : ClientMessage) {
    let mut buf = vec![0u8; message.size()];
    message.encode_into(&mut buf);
    send_ws(buf);
}


struct PlayerData {
    id : u64,
    slot : u8,
    name : String,
    money : u32
}


struct InputState {
    mouse_x : f32,
    mouse_y : f32,
    keys_down : HashMap<String, bool>,
    mouse_down : bool
}


impl InputState {
    fn key(&self, key : &str) -> bool {
        match self.keys_down.get(key) {
            Some(r) => *r,
            _ => false
        }
    }
}


struct TerritoryData {
    radius : f32
}


struct FabberData {
    radius : f32
}


struct ObjectData {
    id : u32,
    tp : PieceType,
    x : f32,
    y : f32,
    a : f32,
    owner : u64,
    health : f32
}


#[wasm_bindgen]
struct State {
    gameboard_width : f32,
    gameboard_height : f32,
    id : u64,
    slot : u8,
    is_io : bool,
    playing : bool,
    strategy : bool,
    tick : u16,
    time_in_stage : u16,
    global_tick : u32,
    player_data : HashMap<u64, PlayerData>,
    object_data : HashMap<u32, ObjectData>,
    territory_data : HashMap<u32, TerritoryData>,
    fabber_data : HashMap<u32, FabberData>,
    inputs : InputState,
    off_x : f32,
    off_y : f32,
    x_scroll_ramp : f32,
    y_scroll_ramp : f32,
    has_placed : bool,
    money : u32,
    has_tested : bool,
    territory_buf : Vec<f32>,
    fabber_buf : Vec<f32>,
    path : PathFollower
}


const SCROLL_ACC : f32 = 0.3;
const SCROLL_BASE : f32 = 5.0;
const SCROLL_MAX : f32 = 30.0;


impl State {
    fn place(&self, tp : PieceType) {
        send(ClientMessage::PlacePiece(self.inputs.mouse_x, self.inputs.mouse_y, tp as u16));
    }

    fn overlay(&mut self) {
        self.fabber_buf.clear();
        self.territory_buf.clear();
        for (piece, fabber) in &self.fabber_data {
            if let Some(obj) = self.object_data.get(&piece) {
                self.fabber_buf.reserve(3);
                self.fabber_buf.push(obj.x);
                self.fabber_buf.push(obj.y);
                self.fabber_buf.push(fabber.radius * if self.is_friendly(obj.owner) { 1.0 } else { -1.0 });
            }
        }
        for (piece, fabber) in &self.territory_data {
            if let Some(obj) = self.object_data.get(&piece) {
                self.territory_buf.reserve(3);
                self.territory_buf.push(obj.x);
                self.territory_buf.push(obj.y);
                self.territory_buf.push(fabber.radius * if self.is_friendly(obj.owner) { 1.0 } else { -1.0 });
            }
        }
        render_background(&mut self.fabber_buf, self.fabber_data.len(), &mut self.territory_buf, self.territory_data.len());
    }

    fn is_friendly(&self, other : u64) -> bool {
        if other == 0 {
            return false;
        }
        if other == self.id {
            return true;
        }
        if let Some(player) = self.player_data.get(&other) {
            if player.slot == self.slot && self.slot > 1 { // slot 0 is spectator, 1 is free-agent, and 2-255 are teams. if we're on the same team, they're our friend!
                true
            }
            else {
                false
            }
        }
        else {
            false
        }
    }
}

#[wasm_bindgen]
impl State {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            gameboard_height : 0.0,
            gameboard_width : 0.0,
            id : 0,
            slot : 0,
            is_io: false,
            playing: false,
            strategy: false,
            tick : 0,
            time_in_stage : 0,
            global_tick : 0,
            player_data : HashMap::new(),
            territory_data : HashMap::new(),
            fabber_data : HashMap::new(),
            object_data : HashMap::new(),
            inputs : InputState {
                mouse_x : 0.0,
                mouse_y : 0.0,
                keys_down : HashMap::new(),
                mouse_down : false
            },
            off_x : 0.0,
            off_y : 0.0,
            x_scroll_ramp : 0.0,
            y_scroll_ramp : 0.0,
            has_placed : false,
            money : 0,
            has_tested : false,
            territory_buf : vec![],
            fabber_buf : vec![]
        }
    }

    pub fn tick(&mut self) {
        let ysc = (SCROLL_BASE + SCROLL_ACC * self.y_scroll_ramp).min(SCROLL_MAX);
        let xsc = (SCROLL_BASE + SCROLL_ACC * self.x_scroll_ramp).min(SCROLL_MAX);
        if self.inputs.key("ArrowUp") {
            self.off_y -= ysc;
            self.y_scroll_ramp += 1.0;
        }
        else if self.inputs.key("ArrowDown") {
            self.off_y += ysc;
            self.y_scroll_ramp += 1.0;
        }
        else {
            self.y_scroll_ramp = 0.0;
        }
        if self.inputs.key("ArrowLeft") {
            self.off_x -= xsc;
            self.x_scroll_ramp += 1.0;
        }
        else if self.inputs.key("ArrowRight") {
            self.off_x += xsc;
            self.x_scroll_ramp += 1.0;
        }
        else {
            self.x_scroll_ramp = 0.0;
        }
        self.overlay();
        set_offset(self.off_x, self.off_y);
        ctx_stroke(2.0, "white");
        ctx_outline_circle(self.inputs.mouse_x, self.inputs.mouse_y, 5.0);
        for obj in self.object_data.values() {
            let resource = match obj.tp.asset() {
                Asset::Simple(uri) => uri,
                Asset::Partisan(friendly, enemy) => {
                    if self.is_friendly(obj.owner) {
                        friendly
                    }
                    else {
                        enemy
                    }
                },
                Asset::Unimpl => "notfound.svg"
            };
            let wh = obj.tp.shape().to_bbox();
            ctx_draw_sprite(resource, obj.x, obj.y, obj.a, wh.0, wh.1);
        }
    }

    pub fn set_mouse_pos(&mut self, x : f32, y : f32) {
        self.inputs.mouse_x = x;
        self.inputs.mouse_y = y;
    }

    pub fn mouse_down(&mut self) {
        self.inputs.mouse_down = true;
        if self.has_placed {
            
        }
        else { // we don't have a castle yet! let's place that now, if possible
            // TODO: check territory stuff
            self.place(PieceType::Castle);
            self.has_placed = true;
        }
    }

    pub fn mouse_up(&mut self) {
        self.inputs.mouse_down = true;
    }

    pub fn key_down(&mut self, key : String) {
        self.inputs.keys_down.insert(key, true);
    }

    pub fn key_up(&mut self, key : String) {
        self.inputs.keys_down.insert(key, false);
    }

    pub fn on_message(&mut self, message : Vec<u8>) {
        let message = ServerMessage::decode(&message);
        if let Ok(ref msg) = message {
            if self.has_tested {
                match msg {
                    ServerMessage::Metadata(id, gb_w, gb_h, slot) => {
                        self.gameboard_width = *gb_w;
                        self.gameboard_height = *gb_h;
                        self.id = *id;
                        self.slot = *slot;
                        set_board_size(*gb_w, *gb_h);
                    },
                    ServerMessage::GameState(byte, tick, stage_time) => {
                        self.tick = *tick;
                        self.time_in_stage = *stage_time;
                        self.is_io = byte & 0b10000000 != 0;
                        self.playing = byte & 0b01000000 != 0;
                        self.strategy = byte & 0b00100000 != 0;
                        self.global_tick += 1;
                        set_time(*tick, *stage_time, if self.playing {
                            if self.strategy {
                                "MOVE SHIPS"
                            }
                            else {
                                "PLAYING"
                            }
                        } else { "WAITING" });
                    },
                    ServerMessage::PlayerData(id, banner, slot) => {
                        self.player_data.insert(*id, PlayerData {
                            id : *id,
                            name : banner.clone(),
                            slot : *slot,
                            money : 0
                        });
                    },
                    ServerMessage::Money(id, amount) => {
                        if let Some(player) = self.player_data.get_mut(id) {
                            player.money = *amount;
                        }
                        if self.id == *id {
                            self.money = *amount;
                            set_money(*amount);
                        }
                    },
                    ServerMessage::Territory(id, radius) => {
                        self.territory_data.insert(*id, TerritoryData {
                            radius : *radius
                        });
                    },
                    ServerMessage::Fabber(id, radius) => {
                        self.fabber_data.insert(*id, FabberData {
                            radius : *radius
                        });
                    },
                    ServerMessage::ObjectCreate(x, y, a, owner, id, tp) => {
                        self.object_data.insert(*id, ObjectData {
                            x : *x,
                            y : *y,
                            a : *a,
                            owner : *owner,
                            id : *id,
                            tp : PieceType::from_u16(*tp).unwrap(),
                            health : 1.0
                        });
                    },
                    ServerMessage::ObjectMove(id, x, y, a) => {
                        if let Some(obj) = self.object_data.get_mut(&id) {
                            obj.x = *x;
                            obj.y = *y;
                            obj.a = *a;
                        }
                    },
                    ServerMessage::DeleteObject(id) => {
                        self.object_data.remove(&id);
                    },
                    ServerMessage::Health(id, health) => {
                        if let Some(obj) = self.object_data.get_mut(&id) {
                            obj.health = *health;
                        }
                    }
                    _ => {
                        alert(&format!("bad protocol frame {:?}", msg));
                    }
                }
            }
            else {
                if let ServerMessage::Test(exostring, 128, 4096, 115600, 123456789012345, -64, -4096, -115600, -123456789012345, -4096.512, -8192.756, VERSION) = msg {
                    if exostring == "EXOSPHERE" {
                        send(ClientMessage::Test("EXOSPHERE".to_string(), 128, 4096, 115600, 123456789012345, -64, -4096, -115600, -123456789012345, -4096.512, -8192.756, VERSION));
                        send(ClientMessage::Connect(get_input_value("nickname"), "".to_string()));
                        self.has_tested = true;
                        return;
                    }
                }
                alert(&format!("server failed verification: {:?}", message));
            }
        }
        else {
            alert("error");
        }
    }
}