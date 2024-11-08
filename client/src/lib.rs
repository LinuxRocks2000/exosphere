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
use common::VERSION;


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
    fn register_listeners(
        tick : &Closure<dyn FnMut()>,
        key : &Closure<dyn FnMut(String, KeyAcrossEvent)>,
        mouse : &Closure<dyn FnMut(f32, f32, MouseAcrossEvent)>,
        websocket : &Closure<dyn FnMut(Vec<u8>)>);
    fn send_ws(data : Vec<u8>);
    fn get_input_value(name : &str) -> String;
    fn set_board_size(w : f32, h : f32);
    fn set_offset(x : f32, y : f32);
    fn set_time(tick : u16, stage : u16, phase_name : &str);
}


fn send(message : ClientMessage) {
    let mut buf = vec![0u8; message.size()];
    message.encode_into(&mut buf);
    send_ws(buf);
}

use std::collections::HashMap;


struct PlayerData {
    id : u64,
    slot : u8,
    name : String
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
    inputs : InputState,
    off_x : f32,
    off_y : f32,
    x_scroll_ramp : f32,
    y_scroll_ramp : f32
}


const SCROLL_ACC : f32 = 0.3;
const SCROLL_BASE : f32 = 5.0;
const SCROLL_MAX : f32 = 30.0;


impl State {
    fn tick(&mut self) {
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
        set_offset(self.off_x, self.off_y);
    }
}


use std::sync::RwLock;
use std::sync::Arc;


#[wasm_bindgen]
pub fn entrypoint() {
    let mut has_tested = false;
    let state = Arc::new(RwLock::new(State {
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
        inputs : InputState {
            mouse_x : 0.0,
            mouse_y : 0.0,
            keys_down : HashMap::new(),
            mouse_down : false
        },
        off_x : 0.0,
        off_y : 0.0,
        x_scroll_ramp : 0.0,
        y_scroll_ramp : 0.0
    }));
    let state_1 = state.clone();
    let state_2 = state.clone();
    let state_3 = state.clone();
    let tick = Box::new(Closure::new(move || {
        let mut state = state.write().unwrap();
        state.tick();
    }));
    let key = Box::new(Closure::new(move |key, evt| {
        let mut state = state_1.write().unwrap();
        match evt {
            KeyAcrossEvent::KeyUp => {
                state.inputs.keys_down.insert(key, false);
            },
            KeyAcrossEvent::KeyDown => {
                state.inputs.keys_down.insert(key, true);
            }
        }
    }));
    let mouse = Box::new(Closure::new(move |mx, my, evt| {
        let mut state = state_2.write().unwrap();
        state.inputs.mouse_x = mx;
        state.inputs.mouse_y = my;
        match evt {
            MouseAcrossEvent::Move => {

            },
            MouseAcrossEvent::Up => {
                state.inputs.mouse_down = false;
            },
            MouseAcrossEvent::Down => {
                state.inputs.mouse_down = true;
            }
        }
    }));
    let websocket = Box::new(Closure::new(move |d : Vec<u8>| {
        let mut state = state_3.write().unwrap();
        let message = ServerMessage::decode_from(&d);
        if let Ok(ref msg) = message {
            if has_tested {
                match msg {
                    ServerMessage::Metadata(id, gb_w, gb_h, slot) => {
                        state.gameboard_width = *gb_w;
                        state.gameboard_height = *gb_h;
                        state.id = *id;
                        state.slot = *slot;
                        set_board_size(*gb_w, *gb_h);
                    },
                    ServerMessage::GameState(byte, tick, stage_time) => {
                        state.tick = *tick;
                        state.time_in_stage = *stage_time;
                        state.is_io = byte & 0b10000000 != 0;
                        state.playing = byte & 0b01000000 != 0;
                        state.strategy = byte & 0b00100000 != 0;
                        state.global_tick += 1;
                        set_time(*tick, *stage_time, if state.playing {
                            if state.strategy {
                                "MOVE SHIPS"
                            }
                            else {
                                "PLAYING"
                            }
                        } else { "WAITING" });
                    },
                    ServerMessage::PlayerData(id, banner, slot) => {
                        state.player_data.insert(*id, PlayerData {
                            id : *id,
                            name : banner.clone(),
                            slot : *slot
                        });
                    },
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
                        has_tested = true;
                        return;
                    }
                }
                alert(&format!("server failed verification: {:?}", message));
            }
        }
        else {
            alert("error");
        }
    }));
    register_listeners( // register_listeners is glued into javascript, presumably around memory protections. dropping the closures doesn't do anything on the WASM side but
        // the javascript side breaks (because it's trying to call closures that don't exist)
        // if you know a better way to intentionally leak memory over the wasm boundary, PLEASE pr it in.
        Box::leak(tick),
        Box::leak(key),
        Box::leak(mouse),
        Box::leak(websocket)
    );
}