/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

use common::comms::*;
use common::pathfollower::{PathFollower, PathIter, PathNode};
use common::steal_mut;
use common::types::PieceType;
use common::VERSION;
use common::{PieceId, PlayerId};
use num_traits::cast::FromPrimitive;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
// TODO: refactor this whole thing to use bevy
// pretty important
// right now this code is unstable, hard to understand, and calls common::steal_mut *multiple times*.
// it is essentially impossible to maintain in any coherent way. a move to ECS is necessary.

const PLACE_MENU: [&'static [PieceType]; 4] = [
    &[
        PieceType::BallisticMissile,
        PieceType::SeekingMissile,
        PieceType::HypersonicMissile,
        PieceType::TrackingMissile,
        PieceType::CruiseMissile,
    ], // missiles
    &[
        PieceType::BasicFighter,
        PieceType::TieFighter,
        PieceType::Sniper,
        PieceType::FleetDefenseShip,
        PieceType::DemolitionCruiser,
        PieceType::Battleship,
    ], // ships
    &[PieceType::Seed, PieceType::Farmhouse, PieceType::ScrapShip], // economic
    &[
        PieceType::LaserNode,
        PieceType::BasicTurret,
        PieceType::LaserNodeLR,
        PieceType::SmartTurret,
        PieceType::BlastTurret,
        PieceType::LaserTurret,
        PieceType::EmpZone,
    ], // defense
];

#[derive(Debug)]
#[wasm_bindgen]
pub enum KeyAcrossEvent {
    KeyUp,
    KeyDown,
}

#[wasm_bindgen]
pub enum MouseAcrossEvent {
    Move,
    Up,
    Down,
}

#[wasm_bindgen]
struct TeamDescriptor {
    name: String,
    id: u8,
}

#[wasm_bindgen]
impl TeamDescriptor {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_id(&self) -> u8 {
        self.id
    }
}

#[wasm_bindgen(module = "/core.js")]
extern "C" {
    fn alert(s: &str);
    fn send_ws(data: Vec<u8>);
    fn get_input_value(name: &str) -> String;
    fn set_board_size(w: f32, h: f32);
    fn set_offset(x: f32, y: f32);
    fn set_time(tick: u16, stage: u16, phase_name: &str);
    fn ctx_stroke(width: f32, color: &str);
    fn ctx_fill(color: &str);
    fn ctx_outline_circle(x: f32, y: f32, rad: f32);
    fn ctx_fill_circle(x: f32, y: f32, rad: f32);
    fn set_money(amount: u32);
    fn render_background(
        fabbers_buf: &mut [f32],
        fabbers_count: usize,
        territories_buf: &mut [f32],
        territory_count: usize,
    );
    fn ctx_draw_image(resource: &str, x: f32, y: f32, a: f32, w: f32, h: f32); // draw an image at a given position and angle
    fn ctx_line_between(x1: f32, y1: f32, x2: f32, y2: f32);
    fn setup_placemenu_row(index: usize);
    fn add_placemenu_item(row: usize, item: u16, img: &str, name: &str, desc: &str, cost: u32);
    fn clear_piecepicker();
    fn ctx_alpha(alpha: f32);
    fn ctx_fill_rect(x: f32, y: f32, w: f32, h: f32);
    fn reload();
    fn draw_text_box(x: f32, y: f32, lines: Vec<String>);
    fn screen(scr: &str);
    fn set_teams_select(teams: Vec<TeamDescriptor>);
}

fn send(message: ClientMessage) {
    send_ws(bitcode::encode(&message));
}

struct PlayerData {
    id: PlayerId,
    slot: u8,
    name: String,
    money: u32,
}

struct InputState {
    mouse_x: f32,
    mouse_y: f32,
    keys_down: HashMap<String, bool>,
    mouse_down: bool,
}

impl InputState {
    fn key(&self, key: &str) -> bool {
        match self.keys_down.get(key) {
            Some(r) => *r,
            _ => false,
        }
    }
}

struct TerritoryData {
    radius: f32,
}

struct FabberData {
    radius: f32,
}

struct ObjectData {
    id: PieceId,
    tp: PieceType,
    x: f32,
    y: f32,
    a: f32,
    owner: PlayerId,
    health: f32,
    path: PathFollower,
}

impl ObjectData {
    // TODO: make the path functions not spam the server with path updates (some kind of update-cache, mayhaps?)
    // [2025-8-14] this probably isn't really a problem
    fn path_iter<'a>(&'a self) -> PathIter<'a> {
        self.path.iter()
    }

    fn extend_path(&mut self, node: PathNode) {
        let endex = self.path.endex().unwrap();
        self.path.insert_node(endex, node);
        send(ClientMessage::Strategy {
            evt: StrategyPathModification::Insert(self.id, endex, node),
        });
    }

    fn path_insert(&mut self, before: u16, node: PathNode) {
        self.path.insert_node(before, node);
        send(ClientMessage::Strategy {
            evt: StrategyPathModification::Insert(self.id, before, node),
        });
    }

    fn update_strategy(&mut self, index: u16, node: PathNode) {
        self.path.update_node(index, node);
        send(ClientMessage::Strategy {
            evt: StrategyPathModification::Set(self.id, index, node),
        });
    }

    fn delete_strategy(&mut self, index: u16) {
        self.path.remove_node(index);
        send(ClientMessage::Strategy {
            evt: StrategyPathModification::Delete(self.id, index),
        });
    }
}

#[wasm_bindgen]
struct State {
    gameboard_width: f32,
    gameboard_height: f32,
    id: PlayerId,
    slot: u8,
    tick: u16,
    stage_duration: u16,
    stage: Stage,
    global_tick: u32,
    player_data: HashMap<PlayerId, PlayerData>,
    object_data: HashMap<PieceId, ObjectData>,
    territory_data: HashMap<PieceId, TerritoryData>,
    fabber_data: HashMap<PieceId, FabberData>,
    inputs: InputState,
    off_x: f32,
    off_y: f32,
    x_scroll_ramp: f32,
    y_scroll_ramp: f32,
    has_placed: bool,
    money: u32,
    has_tested: bool,
    territory_buf: Vec<f32>,
    fabber_buf: Vec<f32>,
    active_piece: Option<PieceId>,
    hovered: Option<PieceId>,
    hovered_anything: Option<PieceId>,
    updating_node: Option<(PieceId, u16)>,
    piecepicker: Option<PieceType>,
    lasers: Vec<Laser>,
    explosions: Vec<Explosion>,
    gun_states: HashMap<PieceId, bool>,
    is_placeable: bool,
}

const SCROLL_ACC: f32 = 0.3;
const SCROLL_BASE: f32 = 5.0;
const SCROLL_MAX: f32 = 30.0;

impl State {
    fn place(&self, tp: PieceType) {
        send(ClientMessage::PlacePiece {
            x: self.inputs.mouse_x,
            y: self.inputs.mouse_y,
            tp,
        });
    }

    fn overlay(&mut self) {
        self.fabber_buf.clear();
        self.territory_buf.clear();
        let mut territories_count = 0;
        let mut fabbers_count = 0;
        for (piece, fabber) in &self.fabber_data {
            if let Some(obj) = self.object_data.get(&piece) {
                self.fabber_buf.reserve(3);
                self.fabber_buf.push(obj.x);
                self.fabber_buf.push(obj.y);
                self.fabber_buf.push(
                    fabber.radius
                        * if self.is_friendly(obj.owner) {
                            1.0
                        } else {
                            -1.0
                        },
                );
                fabbers_count += 1;
            }
        }
        for (piece, fabber) in &self.territory_data {
            if let Some(obj) = self.object_data.get(&piece) {
                self.territory_buf.reserve(3);
                self.territory_buf.push(obj.x);
                self.territory_buf.push(obj.y);
                self.territory_buf.push(
                    fabber.radius
                        * if self.is_friendly(obj.owner) {
                            1.0
                        } else {
                            -1.0
                        },
                );
                territories_count += 1;
            }
        }
        render_background(
            &mut self.fabber_buf,
            fabbers_count,
            &mut self.territory_buf,
            territories_count,
        );
    }

    fn is_friendly(&self, other: PlayerId) -> bool {
        if other == PlayerId::SYSTEM {
            return false;
        }
        if other == self.id {
            return true;
        }
        if let Some(player) = self.player_data.get(&other) {
            if player.slot == self.slot && self.slot > 1 {
                // slot 0 is spectator, 1 is free-agent, and 2-255 are teams. if we're on the same team, they're our friend!
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

struct Laser {
    age: u8,
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
}

struct Explosion {
    x: f32,
    y: f32,
    rad: f32,
    age: u8,
}

#[wasm_bindgen]
impl State {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // state creation is the first thing that happens and will never happen again, so it's a good point to do entry routines (like setting the panic hook)
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        for row in 0..PLACE_MENU.len() {
            setup_placemenu_row(row);
            for item in PLACE_MENU[row] {
                add_placemenu_item(
                    row,
                    *item as u16,
                    item.asset().to_friendly(),
                    item.name(),
                    item.description(),
                    item.price(),
                );
            }
        }
        Self {
            gameboard_height: 0.0,
            gameboard_width: 0.0,
            id: PlayerId::SYSTEM,
            slot: 0,
            stage: Stage::Waiting,
            tick: 0,
            stage_duration: 0,
            global_tick: 0,
            player_data: HashMap::new(),
            territory_data: HashMap::new(),
            fabber_data: HashMap::new(),
            object_data: HashMap::new(),
            inputs: InputState {
                mouse_x: 0.0,
                mouse_y: 0.0,
                keys_down: HashMap::new(),
                mouse_down: false,
            },
            off_x: 0.0,
            off_y: 0.0,
            x_scroll_ramp: 0.0,
            y_scroll_ramp: 0.0,
            has_placed: false,
            money: 0,
            has_tested: false,
            territory_buf: vec![],
            fabber_buf: vec![],
            active_piece: None,
            hovered: None,
            hovered_anything: None,
            updating_node: None,
            piecepicker: Some(PieceType::Castle),
            lasers: vec![],
            explosions: vec![],
            gun_states: HashMap::new(),
            is_placeable: false,
        }
    }

    pub fn tick(&mut self) {
        let ysc = (SCROLL_BASE + SCROLL_ACC * self.y_scroll_ramp).min(SCROLL_MAX);
        let xsc = (SCROLL_BASE + SCROLL_ACC * self.x_scroll_ramp).min(SCROLL_MAX);
        if self.inputs.key("ArrowUp") || self.inputs.key("w") {
            self.off_y -= ysc;
            self.y_scroll_ramp += 1.0;
        } else if self.inputs.key("ArrowDown") || self.inputs.key("s") {
            self.off_y += ysc;
            self.y_scroll_ramp += 1.0;
        } else {
            self.y_scroll_ramp = 0.0;
        }
        if self.inputs.key("ArrowLeft") || self.inputs.key("a") {
            self.off_x -= xsc;
            self.x_scroll_ramp += 1.0;
        } else if self.inputs.key("ArrowRight") || self.inputs.key("d") {
            self.off_x += xsc;
            self.x_scroll_ramp += 1.0;
        } else {
            self.x_scroll_ramp = 0.0;
        }
        self.overlay();
        self.hovered = None;
        self.hovered_anything = None;
        for obj in self.object_data.values() {
            if let Some(radius) = obj.tp.sensor() {
                ctx_stroke(0.5, "#AAAAAA");
                ctx_outline_circle(obj.x, obj.y, radius);
            }
            if obj.health < 1.0 {
                ctx_fill("#AAAAAA");
                ctx_fill_rect(
                    obj.x - 37.5,
                    obj.y - obj.tp.shape().to_bbox().1 - 20.0,
                    75.0,
                    5.0,
                );
                ctx_fill("red");
                ctx_fill_rect(
                    obj.x - 37.5,
                    obj.y - obj.tp.shape().to_bbox().1 - 20.0,
                    75.0 * obj.health,
                    5.0,
                );
            }
        }
        for obj in self.object_data.values() {
            let asset = obj.tp.asset();
            let resource = if self.is_friendly(obj.owner) {
                asset.to_friendly()
            } else {
                asset.to_enemy()
            };
            let wh = obj.tp.shape().to_bbox();
            ctx_draw_image(resource, obj.x, obj.y, obj.a, wh.0, wh.1);
            if obj.tp.user_movable() && obj.owner == self.id {
                let mut running_x = obj.x;
                let mut running_y = obj.y;
                ctx_stroke(1.0, "white");
                for node in obj.path_iter() {
                    let (x, y) = match node {
                        PathNode::StraightTo(x, y) => (x, y),
                        PathNode::Target(piece) => {
                            if let Some(piece) = self.object_data.get(&piece) {
                                (piece.x, piece.y)
                            } else {
                                (running_x, running_y)
                            }
                        }
                        PathNode::Rotation(a, _) => {
                            ctx_draw_image(
                                "rotation_arrow.svg",
                                running_x,
                                running_y,
                                a,
                                30.0,
                                30.0,
                            );
                            (running_x, running_y)
                        }
                    };
                    ctx_fill("white");
                    ctx_fill_circle(x, y, 1.0);
                    if running_x != x || running_y != y {
                        ctx_line_between(running_x, running_y, x, y);
                    }
                    running_x = x;
                    running_y = y;
                }

                let mut dx = self.inputs.mouse_x - obj.x;
                let mut dy = self.inputs.mouse_y - obj.y;

                if dx * dx + dy * dy < 6.0 * 6.0 && obj.tp.user_movable() {
                    if obj.owner == self.id {
                        self.hovered = Some(obj.id);
                    }
                    self.hovered_anything = Some(obj.id);
                }
                if self.active_piece == Some(obj.id) {
                    ctx_stroke(2.0, "white");
                    ctx_outline_circle(running_x, running_y, 5.0);
                    dx = self.inputs.mouse_x - running_x;
                    dy = self.inputs.mouse_y - running_y;
                }
                if self.inputs.key("r") {
                    if let None = self.updating_node {
                        if self.active_piece == Some(obj.id) {
                            let dura = (((dx * dx + dy * dy).sqrt() * 0.16) as u16).min(100);
                            let node = PathNode::Rotation(dy.atan2(dx), dura);
                            if let Some(PathNode::Rotation(r, dur)) = obj.path.get_last() {
                                let ind = obj.path.len().unwrap() - 1;
                                steal_mut(&obj.path).update_node(ind, node);
                                send(ClientMessage::Strategy {
                                    evt: StrategyPathModification::Set(obj.id, ind, node),
                                });
                            } else {
                                steal_mut(obj).extend_path(node);
                            }
                        }
                    }
                }
                if obj.tp.show_field() {
                    if let Some(field) = obj.tp.field() {
                        ctx_stroke(1.0, "white");
                        ctx_outline_circle(obj.x, obj.y, field);
                    }
                }
            }
            let dx = self.inputs.mouse_x - obj.x;
            let dy = self.inputs.mouse_y - obj.y;

            if dx * dx + dy * dy < 6.0 * 6.0 && obj.owner != self.id {
                if let Some(player) = self.player_data.get(&obj.owner) {
                    draw_text_box(
                        self.inputs.mouse_x,
                        self.inputs.mouse_y,
                        vec![
                            if self.is_friendly(obj.owner) {
                                "FRIENDLY"
                            } else {
                                "ENEMY"
                            }
                            .to_string(),
                            player.name.clone(),
                        ],
                    );
                }
            }
        }
        set_offset(self.off_x, self.off_y);
        if self.stage == Stage::MoveShips {
            if let Some((id, index)) = self.updating_node {
                if self.inputs.key("r") {
                    if let Some(piece) = self.object_data.get(&id) {
                        let (x, y) = match piece.path.get(index as usize).unwrap() {
                            PathNode::StraightTo(x, y) => (x, y),
                            PathNode::Target(piece) => {
                                if let Some(piece) = self.object_data.get(&piece) {
                                    (piece.x, piece.y)
                                } else {
                                    return;
                                }
                            }
                            PathNode::Rotation(_, _) => unreachable!(),
                        };
                        let dx = self.inputs.mouse_x - x;
                        let dy = self.inputs.mouse_y - y;
                        let dura = (((dx * dx + dy * dy).sqrt() * 0.16) as u16).min(100);
                        let node = PathNode::Rotation(dy.atan2(dx), dura);
                        if let Some(PathNode::Rotation(_, _)) = piece.path.get(index as usize + 1) {
                            // if there's already a rotation in this path
                            steal_mut(piece)
                                .update_strategy(index + 1, PathNode::Rotation(dy.atan2(dx), dura));
                        } else {
                            steal_mut(piece).path_insert(index + 1, node);
                        }
                    }
                } else if self.inputs.key("Backspace") {
                    if let Some(mut piece) = self.object_data.get_mut(&id) {
                        if let Some(PathNode::Rotation(_, _)) = piece.path.get(index as usize + 1) {
                            piece.delete_strategy(index + 1);
                        } else {
                            piece.delete_strategy(index);
                        }
                        self.updating_node = None;
                    }
                } else {
                    if let Some(mut piece) = self.object_data.get_mut(&id) {
                        piece.update_strategy(
                            index,
                            PathNode::StraightTo(self.inputs.mouse_x, self.inputs.mouse_y),
                        );
                    }
                }
            }
        }
        ctx_stroke(2.0, "red");
        self.lasers.retain_mut(|laser: &mut Laser| {
            laser.age -= 1;
            ctx_line_between(laser.from_x, laser.from_y, laser.to_x, laser.to_y);
            laser.age > 0
        });
        ctx_fill("orange");
        self.explosions.retain_mut(|explosion: &mut Explosion| {
            explosion.age -= 1;
            ctx_fill_circle(explosion.x, explosion.y, explosion.rad);
            explosion.age > 0
        });
        if let Some(tp) = self.piecepicker {
            self.is_placeable = !self.has_placed;
            for (piece, fabber) in &self.fabber_data {
                let obj = &self.object_data[piece];
                if obj.owner == self.id {
                    let dx = self.inputs.mouse_x - obj.x;
                    let dy = self.inputs.mouse_y - obj.y;
                    if dx * dx + dy * dy < fabber.radius * fabber.radius {
                        self.is_placeable = true;
                        break;
                    }
                }
            }
            if self.is_placeable {
                ctx_alpha(0.5);
                let wh = tp.shape().to_bbox();
                ctx_draw_image(
                    tp.asset().to_friendly(),
                    self.inputs.mouse_x,
                    self.inputs.mouse_y,
                    0.0,
                    wh.0,
                    wh.1,
                );
                ctx_alpha(1.0);
                if let Some(field) = tp.field() {
                    ctx_stroke(1.0, "white");
                    ctx_outline_circle(self.inputs.mouse_x, self.inputs.mouse_y, field);
                }
            }
        } else {
            ctx_stroke(2.0, "white");
            ctx_outline_circle(self.inputs.mouse_x, self.inputs.mouse_y, 5.0);
        }
    }

    pub fn set_mouse_pos(&mut self, x: f32, y: f32) {
        self.inputs.mouse_x = x;
        self.inputs.mouse_y = y;
    }

    pub fn mouse_down(&mut self) {
        self.inputs.mouse_down = true;
        if self.has_placed {
            if let Stage::MoveShips = self.stage {
                if let Some(piece) = self.piecepicker {
                    if self.money >= piece.price() {
                        if self.is_placeable {
                            self.place(piece);
                            if !self.inputs.key("Shift") {
                                self.piecepicker = None;
                                clear_piecepicker();
                            }
                        }
                    }
                } else if self.hovered.is_none()
                    && self.hovered_anything.is_some()
                    && self.active_piece.is_some()
                {
                    if let Some(mut piece) = self.object_data.get_mut(&self.active_piece.unwrap()) {
                        if piece.tp.supports_target_control() {
                            piece.extend_path(PathNode::Target(self.hovered_anything.unwrap()));
                        }
                    }
                } else if let None = self.hovered {
                    // if we aren't hovering anything new, create a path node
                    // first up: check if this click might extend a pre-existing path
                    let mx = self.inputs.mouse_x;
                    let my = self.inputs.mouse_y;
                    let mut nearest_node = None;
                    let mut nearest_obj = PieceId::ZERO;
                    let mut nearest_node_dist = 0.0;
                    let mut nearest_pair: (f32, f32) = (0.0, 0.0);
                    let mut is_move = false; // are we going to place a new path node at a projection on the segment? or are we going to move (or otherwise update) a node?
                    'outer: for obj in self.object_data.values() {
                        let mut running_x = obj.x;
                        let mut running_y = obj.y;
                        for i in 0..obj.path.len().unwrap() {
                            let node = obj.path.get(i as usize).unwrap();
                            let (x, y) = match node {
                                PathNode::StraightTo(x, y) => (x, y),
                                PathNode::Target(piece) => {
                                    if let Some(piece) = self.object_data.get(&piece) {
                                        (piece.x, piece.y)
                                    } else {
                                        (running_x, running_y)
                                    }
                                }
                                PathNode::Rotation(a, _) => {
                                    ctx_draw_image(
                                        "rotation_arrow.svg",
                                        running_x,
                                        running_y,
                                        a,
                                        30.0,
                                        30.0,
                                    );
                                    (running_x, running_y)
                                }
                            };
                            let d_x = x - mx;
                            let d_y = y - my;
                            if d_x * d_x + d_y * d_y < 5.0 * 5.0 {
                                is_move = true;
                                nearest_node = Some(i);
                                nearest_obj = obj.id;
                                break 'outer;
                            }
                            if running_x != x || running_y != y {
                                let v_x = x - running_x;
                                let v_y = y - running_y;
                                let u_x = running_x - mx;
                                let u_y = running_y - my;
                                let t = (-1.0 * (v_x * u_x + v_y * u_y) / (v_x * v_x + v_y * v_y))
                                    .min(1.0)
                                    .max(0.0);
                                let p_x = running_x * (1.0 - t) + x * t;
                                let p_y = running_y * (1.0 - t) + y * t;
                                let dx = p_x - mx;
                                let dy = p_y - my;
                                let d = dx * dx + dy * dy;
                                if let None = nearest_node {
                                    nearest_pair = (p_x, p_y);
                                    nearest_node_dist = d;
                                    nearest_obj = obj.id;
                                    nearest_node = Some(i);
                                } else if d < nearest_node_dist {
                                    nearest_node = Some(i);
                                    nearest_pair = (p_x, p_y);
                                    nearest_obj = obj.id;
                                    nearest_node_dist = d;
                                }
                            }
                            running_x = x;
                            running_y = y;
                        }
                    }
                    if is_move {
                        self.updating_node = Some((nearest_obj, nearest_node.unwrap()));
                    } else if let Some(piece) = self.active_piece {
                        if let Some(mut piece) = self.object_data.get_mut(&piece) {
                            piece.extend_path(PathNode::StraightTo(
                                self.inputs.mouse_x,
                                self.inputs.mouse_y,
                            ));
                        }
                    } else if let Some(nearest_node) = nearest_node {
                        if nearest_node_dist < 5.0 * 5.0 {
                            if let Some(mut piece) = self.object_data.get_mut(&nearest_obj) {
                                piece.path_insert(
                                    nearest_node,
                                    PathNode::StraightTo(nearest_pair.0, nearest_pair.1),
                                );
                            }
                        }
                    }
                } else {
                    self.active_piece = self.hovered;
                }
            }
        } else {
            // we don't have a castle yet! let's place that now, if possible
            // TODO: check territory stuff
            self.place(PieceType::Castle);
            self.has_placed = true;
            self.piecepicker = None;
        }
    }

    pub fn mouse_up(&mut self) {
        self.inputs.mouse_down = true;
        self.updating_node = None;
    }

    pub fn key_down(&mut self, key: String) {
        self.inputs.keys_down.insert(key, true);
    }

    pub fn key_up(&mut self, key: String) {
        if key == "Escape" {
            self.active_piece = None;
            clear_piecepicker();
            self.piecepicker = None;
        }
        if let Some(piece) = self.active_piece {
            if key == "g" {
                if let Some(mut state) = self.gun_states.get_mut(&piece) {
                    *state = !(*state);
                    send(ClientMessage::Special {
                        id: piece,
                        evt: ObjectSpecialPropertySet::GunState(*state),
                    });
                } else {
                    self.gun_states.insert(piece, true);
                    send(ClientMessage::Special {
                        id: piece,
                        evt: ObjectSpecialPropertySet::GunState(true),
                    });
                }
            }
        }
        self.inputs.keys_down.insert(key, false);
    }

    pub fn piece_picked(&mut self, pick: u16) {
        self.piecepicker = PieceType::from_u16(pick);
    }

    pub fn piece_unpicked(&mut self) {
        self.piecepicker = None;
    }

    pub fn on_message(&mut self, message: Vec<u8>) {
        let message: ServerMessage = bitcode::decode(&message).unwrap();
        if self.has_tested {
            match message {
                ServerMessage::Metadata {
                    id,
                    board_width,
                    board_height,
                    slot,
                } => {
                    self.gameboard_width = board_width;
                    self.gameboard_height = board_height;
                    self.id = id;
                    self.slot = slot;
                    set_board_size(board_width, board_height);
                    screen("gameui");
                }
                ServerMessage::PasswordChallenge => {
                    screen("password-challenge");
                }
                ServerMessage::TeamChallenge { available } => {
                    set_teams_select(
                        available
                            .into_iter()
                            .map(|(name, id)| TeamDescriptor { name, id })
                            .collect(),
                    );
                    screen("team-challenge");
                }
                ServerMessage::GameState {
                    stage,
                    stage_duration,
                    tick_in_stage,
                } => {
                    self.tick = tick_in_stage;
                    self.stage_duration = stage_duration;
                    self.stage = stage;
                    self.global_tick += 1;
                    set_time(tick_in_stage, stage_duration, stage.get_str());
                }
                ServerMessage::PlayerData { id, nickname, slot } => {
                    self.player_data.insert(
                        id,
                        PlayerData {
                            id: id,
                            name: nickname.clone(),
                            slot: slot,
                            money: 0,
                        },
                    );
                }
                ServerMessage::Money { id, amount } => {
                    if let Some(player) = self.player_data.get_mut(&id) {
                        player.money = amount;
                    }
                    if self.id == id {
                        self.money = amount;
                        set_money(amount);
                    }
                }
                ServerMessage::Territory { id, radius } => {
                    self.territory_data
                        .insert(id, TerritoryData { radius: radius });
                }
                ServerMessage::Fabber { id, radius } => {
                    self.fabber_data.insert(id, FabberData { radius: radius });
                }
                ServerMessage::ObjectCreate {
                    x,
                    y,
                    a,
                    owner,
                    id,
                    tp,
                } => {
                    self.object_data.insert(
                        id,
                        ObjectData {
                            x: x,
                            y: y,
                            a: a,
                            owner: owner,
                            id: id,
                            tp: tp,
                            health: 1.0,
                            path: PathFollower::start(x, y),
                        },
                    );
                }
                ServerMessage::ObjectMove { id, x, y, a } => {
                    if let Some(obj) = self.object_data.get_mut(&id) {
                        obj.x = x;
                        obj.y = y;
                        obj.a = a;
                    }
                }
                ServerMessage::DeleteObject { id } => {
                    self.object_data.remove(&id);
                }
                ServerMessage::Health { id, health } => {
                    if let Some(obj) = self.object_data.get_mut(&id) {
                        obj.health = health;
                    }
                }
                ServerMessage::StrategyCompletion { id, remaining } => {
                    if let Some(obj) = self.object_data.get_mut(&id) {
                        obj.path.bump().unwrap();
                        if obj.path.len().unwrap() != remaining {
                            alert(&format!("error! mismatched strategy paths {} (local) vs {} (server)! attempting recovery", obj.path.len().unwrap(), remaining));
                            obj.path.clear();
                            send(ClientMessage::Strategy {
                                evt: StrategyPathModification::Clear(id),
                            });
                        }
                    }
                }
                ServerMessage::LaserCast {
                    caster: _,
                    from_x,
                    from_y,
                    to_x,
                    to_y,
                } => {
                    self.lasers.push(Laser {
                        from_x: from_x,
                        from_y: from_y,
                        to_x: to_x,
                        to_y: to_y,
                        age: 2,
                    });
                }
                ServerMessage::Explosion {
                    x,
                    y,
                    radius,
                    damage: _,
                } => {
                    self.explosions.push(Explosion {
                        x: x,
                        y: y,
                        rad: radius,
                        age: 2,
                    });
                }
                ServerMessage::Disconnect => {
                    // the server is signalling that we will be disconnected. we don't get a choice in the matter
                    // eventually this might do something on the client side; for now it's a no-op
                }
                ServerMessage::Winner { id } => {
                    // TODO: win screen
                    if self.id == id {
                        alert("you won!");
                    } else {
                        alert(&format!("{:?} won!", id));
                    }
                    reload();
                }
                ServerMessage::YouLose => {
                    // todo: loss screen
                    alert("you lost");
                    reload();
                }
                ServerMessage::Reject => {
                    alert("invalid password!");
                    reload();
                }
                _ => {
                    alert(&format!("bad protocol frame {:?}", message));
                }
            }
        } else {
            if let ServerMessage::Test(
                ref exostring,
                128,
                4096,
                115600,
                123456789012345,
                -64,
                -4096,
                -115600,
                -123456789012345,
                -4096.512,
                -8192.756,
                VERSION,
            ) = message
            {
                if exostring == "EXOSPHERE" {
                    send(ClientMessage::Test(
                        "EXOSPHERE".to_string(),
                        128,
                        4096,
                        115600,
                        123456789012345,
                        -64,
                        -4096,
                        -115600,
                        -123456789012345,
                        -4096.512,
                        -8192.756,
                        VERSION,
                    ));
                    send(ClientMessage::Connect {
                        nickname: get_input_value("nickname"),
                    });
                    self.has_tested = true;
                    return;
                }
            }
            alert(&format!("server failed verification: {:?}", message));
        }
    }

    pub fn on_password_submit(&self, password: String) {
        send(ClientMessage::TryPassword { password });
    }

    pub fn on_team_submit(&self, password: String, team_number: u8) {
        send(ClientMessage::TryTeam {
            team_number,
            password,
        });
    }
}
