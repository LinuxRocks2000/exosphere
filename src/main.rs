/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// biiiiiiiiiiiiiiiiiiiiiiiiiiiiig TODO: split this up into a bunch of different files because JEEZ this is unreadable garbage

// note:
/*
    user ids are u64s.
    user slots are u8s.
    the id is guaranteed to be unique, but slots often collide. the 0 slot means spectator - clients in slot 0 can't do anything. the 1 slot means free agent;
    they are not allied with anybody. slots 2-255 are the team slots. if two players have the same slot, they're allies. yay!

    user id 0 is the system, which does not ever have to obey territory or fabber boundaries.
*/

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::na::Vector;
use warp::Filter;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::{Mutex, mpsc, broadcast};
use std::sync::Arc;
use std::collections::HashMap;
use bevy::ecs::schedule::ScheduleLabel;
use std::f32::consts::PI;

pub mod protocol;
use protocol::Protocol;
use protocol::ProtocolRoot;
use crate::protocol::DecodeError;


const UPDATE_RATE : u64 = 30; // 30hz by default
const FRAME_TIME : std::time::Duration = std::time::Duration::from_millis(1000 / UPDATE_RATE); // milliseconds per frame

const MAX_FRAME_SIZE : usize = 1024; // maximum size of an incoming websocket frame

const VERSION : u8 = 0; // bump this up every time a major change is made

const BASIC_FIGHTER_TYPE_NUM : u16 = 0;
const CASTLE_TYPE_NUM : u16 = 1;

#[derive(Debug, ProtocolRoot, PartialEq)]
enum ClientMessage {
    Test(String, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, u8),
    Connect(String, String), // connect with your nickname and the password respectively. doesn't let you place your castle yet.
    // passwords are, like in MMOSG, used for various things: they can grant entry into a server, they can assign your team, etc. In io games they are usually ignored.
    PlacePiece(f32, f32, f32, u16), // x, y, a, type
    // attempt to place an object
    // before the client can place anything else, it must place a castle (type 1). this is the only time in the game that a client can place an object in neutral territory.
    // obviously it's not possible to place a castle in enemy territory
    StrategyPointAdd(u32, u16, f32, f32), // (id, index, x, y) insert a point to a strategy path at an index
    StrategyPointUpdate(u32, u16, f32, f32), // (id, index, x, y) move a point on a strategy path
    StrategyRemove(u32, u16), // (id, index) remove a node from a strategy
    StrategyClear(u32) // (id) clear a strategy
    // if any Strategy commands are sent referencing nonexistent nodes on a strategy, or StrategyPointUpdate is sent referencing a non-point strategy node (such
    // as a teleportal entrance), the server will simply ignore them. This may be a problem in the future.
}


#[derive(Debug, ProtocolRoot, Clone)]
enum ServerMessage {
    Test(String, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, u8),
    GameState(u8, u16, u16), // game state, stage tick, stage total time
    // this is just 5 bytes - quite acceptable to send to every client every frame, much lower overhead than even ObjectMove.
    // the first byte is bitbanged. bit 1 is io mode enable - in io mode, anyone can place a castle at any time. bit 2 is waiting/playing (1 = playing, 0 = waiting): in wait mode,
    // castles can be placed, and the gameserver begins the game when >=2 castles have been waiting for a certain duration.
    // wait mode does not exist in io mode; if bits 1 and 2 are set, something's wrong. Bit 3 controls if the mode is "move ships" (1) or "play" (0) - in
    // "move ships" mode, people can set and modify the paths their ships will follow, and in play mode the ships will move along those paths.
    // In play mode, castles that wish to do so may also "possess" a ship, controlling its motion in real time; this is the replacement for MMOSG's RTFs.
    // Bits 4-8 are reserved.
    Metadata(u64, f32, f32, u8), // send whatever data (id, board width x height, state) the client needs to begin rendering the gameboard
    // this also tells the client that it was accepted (e.g. got the right password); getting the password _wrong_ would abort the connection
    // state tells the client what position it's occupying. 0 = spectating, 1 = free agent, 2-255 = teams.
    ObjectCreate(f32, f32, f32, u64, u32, u16), // x, y, a, owner, id, type: inform the client of an object.
    ObjectMove(u32, f32, f32, f32), // id, x, y, a
    ObjectTrajectoryUpdate(u32, f32, f32, f32, f32, f32, f32), // id, x, y, a, xv, yv, av
    DeleteObject(u32), // id
    StrategyCompletion(u32, u16) // (id, remaining) a node in a strategy has been fulfilled, and this is the number of strategy nodes remaining!
    // this serves as a sort of checksum; if the number of strategy nodes remaining here is different from the number of strategy nodes remaining in the client
    // the client knows there's something wrong and can send StrategyClear to attempt to recover.
}


// upon connecting, the server immediately sends the client Test("EXOSPHERE", 128, 4096, 115600, 123456789012345, -64, -4096, -115600, -123456789012345, -4096.512, -8192.756, VERSION)
// if the client can successfully decode the message, and VERSION matches the client version, the game may proceed. Otherwise, the client will immediately drop the connection.
// to signify that the test passed, the client will send a Test() with the same values and the client version. If the server can successfully decode them, and the version the
// client sends matches VERSION, the game may proceed. Otherwise, the server will abort the connection.
// this exchange prevents old, underprepared, or incompatible clients from connecting to a game.
// If a client attempts to do anything before protocol verification, it will be kicked off the server.


struct Client {
    id : u64,
    slot : u8,
    channel : tokio::sync::mpsc::Sender<ServerMessage>,
    has_placed_castle : bool
}


// todo: break up GameConfig and GameState into smaller structs for better parallelism
#[derive(Resource)]
struct GameConfig {
    width : f32,
    height : f32,
    wait_period : u16, // time it waits before the game starts
    play_period : u16, // length of a play period
    strategy_period : u16, // length of a strategy period
    max_player_slots : u16,
    min_player_slots : u16
}


#[derive(Resource)]
struct GameState {
    playing : bool,
    io : bool,
    strategy : bool,
    tick : u16,
    time_in_stage : u16,
    currently_attached_players : u16
}


impl GameState {
    fn get_state_byte(&self) -> u8 { // todo: use bit shifting
        self.io as u8 * 128 + self.playing as u8 * 64 + self.strategy as u8 * 32
    }
}


impl Client {
    fn send(&mut self, msg : ServerMessage) {
        if let Err(_) = self.channel.try_send(msg) {
            println!("failed to send message on channel");
        }
    }
}


enum Comms {
    ClientConnect(Client),
    ClientDisconnect(u64),
    MessageFrom(u64, ClientMessage)
}


#[derive(Component)]
struct GamePiece {
    type_indicator : u16, // the type indicator sent to the client
    // assigned by the gamepiece builder functions
    // todo: do this a better way
    owner : u64, // entry in the Clients hashmap
    slot : u8, // identity slot of the owner
    // in the future, we may want to eliminate this and instead do lookups in the HashMap (which is swisstable, so it's pretty fast)
    // but for now it's convenient
    last_update_pos : Vec2
}


#[derive(Component)]
struct Territory { // a territory control radius produced by a castle or fort.
    radius : f32
}

impl Territory {
    fn castle() -> Self { // TODO: make this meaningful
        Self {
            radius : 600.0
        }
    }
}


#[derive(Component)]
struct Fabber { // a fabber bay with a radius
    radius : f32,
    l_missiles : u8,
    l_ships : u8,
    l_econ : u8,
    l_defense : u8,
    l_buildings : u8
}


impl Fabber {
    fn castle() -> Self {
        Self { // Large-M4S2E2D3B2
            radius : 300.0,
            l_missiles : 4,
            l_ships : 2,
            l_econ : 2,
            l_defense : 3,
            l_buildings : 2
        }
    }
}


impl GamePiece {
    fn new(type_indicator : u16, owner : u64, slot : u8) -> Self {
        Self {
            type_indicator,
            owner,
            slot,
            last_update_pos : Vec2 {
                x : 0.0,
                y : 0.0
            }
        }
    }
}


enum Bullets {
    MinorBullet(f32) // simple bullet with range
}

#[derive(Component)]
struct Gun {
    enabled : bool,
    cd : u16, // cooldown ticks between shots
    bullets : Bullets,
    repeats : u16, // number of repeater shots
    repeat_cd : u16, // time between repeater shots
    // state fields (don't touch):
    r_point : u16, // current repeater position
    tick : u16 // current tick
}


enum PathNode {
    StraightTo(f32, f32)
    // todo: teleportal
}


#[derive(Component)]
struct PathFollower { // follow a path.
    nodes : Vec<PathNode>
}


impl PathFollower {
    fn get_next_pos(&self) -> Vec2 {
        match self.nodes[0] {
            PathNode::StraightTo(x, y) => {
                Vec2 { x, y }
            }
        }
    }

    fn start(x : f32, y : f32) -> Self {
        Self {
            nodes : vec![PathNode::StraightTo(x, y)]
        }
    }
}


#[derive(Event)]
struct NewClientEvent {
    id : u64
}


trait Placer { // trait that we can implement on EventWriter
    fn castle(&mut self, x : f32, y : f32, client : u64, slot : u8);

    fn basic_fighter(&mut self, x : f32, y : f32, a : f32, client : u64, slot : u8);

    fn basic_fighter_free(&mut self, x : f32, y : f32, a : f32, client : u64, slot : u8);
}

impl Placer for EventWriter<'_, PlaceEvent> {
    fn castle(&mut self, x : f32, y : f32, client : u64, slot : u8) {
        self.send(PlaceEvent {
            x,
            y,
            a : 0.0,
            owner : client,
            slot,
            tp : PlaceType::Castle,
            free : true
        });
    }

    fn basic_fighter(&mut self, x : f32, y : f32, a : f32, client : u64, slot : u8) {
        self.send(PlaceEvent {
            x, y, a,
            owner : client,
            slot,
            tp : PlaceType::BasicFighter,
            free : false
        });
    }

    fn basic_fighter_free(&mut self, x : f32, y : f32, a : f32, client : u64, slot : u8) {
        self.send(PlaceEvent {
            x, y, a,
            owner : client,
            slot,
            tp : PlaceType::BasicFighter,
            free : true
        });
    }
}


fn client_tick(mut commands : Commands, mut pieces : Query<(Entity, &GamePiece, Option<&mut PathFollower>)>, mut ev_newclient : EventWriter<NewClientEvent>, mut place : EventWriter<PlaceEvent>, mut state : ResMut<GameState>, config : Res<GameConfig>, mut clients : ResMut<ClientMap>, mut receiver : ResMut<Receiver>, broadcast : ResMut<Sender>) {
    // manage events from network-connected clients
    loop { // loops receiver.try_recv(), until it returns empty
        match receiver.try_recv() {
            Ok(message) => {
                match message {
                    Comms::ClientConnect(mut cli) => {
                        clients.insert(cli.id, cli);
                    },
                    Comms::ClientDisconnect(id) => {
                        println!("client disconnected. cleaning up!");
                        for (entity, piece, _) in pieces.iter() {
                            if piece.owner == id {
                                commands.entity(entity).despawn();
                                if let Err(_) = broadcast.send(ServerMessage::DeleteObject(entity.index())) {
                                    println!("game engine lost connection to webserver. this is probably not critical.");
                                }
                            }
                        }
                        state.currently_attached_players -= 1;
                        clients.remove(&id);
                    },
                    Comms::MessageFrom(id, msg) => {
                        let mut kill = false;
                        if let Some(client) = clients.get_mut(&id) {
                            match msg {
                                ClientMessage::Connect(banner, password) => {
                                    let slot : u8 = if state.currently_attached_players < config.max_player_slots { 1 } else { 0 };
                                    client.send(ServerMessage::Metadata(id, config.width, config.height, slot));
                                    state.currently_attached_players += 1;
                                    client.slot = slot;
                                    ev_newclient.send(NewClientEvent {id});
                                },
                                ClientMessage::PlacePiece(x, y, a, t) => {
                                    match t {
                                        CASTLE_TYPE_NUM => {
                                            if client.has_placed_castle {
                                                println!("client attempted to place an extra castle. dropping.");
                                                kill = true;
                                            }
                                            client.has_placed_castle = true;
                                            place.castle(x, y, client.id, client.slot);
                                            place.basic_fighter_free(x - 200.0, y, PI, client.id, client.slot);
                                            place.basic_fighter_free(x + 200.0, y, 0.0, client.id, client.slot);
                                            place.basic_fighter_free(x, y - 200.0, 0.0, client.id, client.slot);
                                            place.basic_fighter_free(x, y + 200.0, 0.0, client.id, client.slot);
                                        },
                                        _ => {
                                            println!("client attempted to place unknown type {}. dropping.", t);
                                            kill = true;
                                        }
                                    }
                                },
                                ClientMessage::StrategyPointAdd(id, index, x, y) => {
                                    for (entity, piece, pathfollower) in pieces.iter_mut() {
                                        if entity.index() == id { // TODO: FIX THIS
                                            // THIS IS REALLY BAD
                                            // REALLY REALLY REALLY BAD
                                            // WE'RE DOING LINEAR TIME LOOKUPS WHERE A CONSTANT TIME LOOKUP WOULD SUFFICE AND WELL
                                            // FIIIIIIIIIIIIIIIIIIX THISSSSSSSSSS
                                            if let Some(mut pathfollower) = pathfollower {
                                                if piece.owner == client.id {
                                                    pathfollower.nodes.insert(index as usize, PathNode::StraightTo(x, y));
                                                }
                                                else {
                                                    println!("client attempted to move thing it doesn't own [how rude]");
                                                }
                                            }
                                            else {
                                                println!("attempt to move immovable object");
                                            }
                                        }
                                    }
                                },
                                _ => {
                                    println!("error: client sent unimplemented frame! dropping client.");
                                    kill = true;
                                }
                            }
                        }
                        else {
                            println!("error: received message from client {}, which does not exist", id);
                        }
                        if kill {
                            clients.remove(&id);
                        }
                    }
                }
            },
            Err(mpsc::error::TryRecvError::Empty) => {
                break;
            }
            _ => {
                println!("ERROR OCCURRED! TAMERE!");
            }
        }
    }
}

fn send_objects(mut events : EventReader<NewClientEvent>, mut clients : ResMut<ClientMap>, objects : Query<(Entity, &GamePiece, &Transform)>) {
    for ev in events.read() {
        if let Some(client) = clients.get_mut(&ev.id) {
            for (entity, piece, transform) in objects.iter() {
                client.send(ServerMessage::ObjectCreate(transform.translation.x, transform.translation.y, transform.rotation.to_axis_angle().1, piece.owner, entity.index(), piece.type_indicator));
            }
        }
    }
}

fn position_updates(broadcast : ResMut<Sender>, mut objects : Query<(Entity, &Velocity, &mut GamePiece, &Transform)>) {
    for (entity, velocity, mut piece, transform) in objects.iter_mut() {
        // todo: only send position updates if it's moving
        let pos = transform.translation.truncate();
        // updates on position
        if (pos - piece.last_update_pos).length() > 1.0 {
            // are basically straight lines.
            let _ = broadcast.send(ServerMessage::ObjectMove( // ignore the errors
                entity.index(),
                pos.x,
                pos.y,
                transform.rotation.to_axis_angle().1
            ));
            piece.last_update_pos = pos;
        }/*
        if (velocity.linvel - piece.last_update_vel).length() > 0.1 {
            let _ = broadcast.send(ServerMessage::ObjectTrajectoryUpdate( // ignore the errors
                entity.index(),
                pos.x,
                pos.y,
                transform.rotation.to_axis_angle().1,
                velocity.linvel.x,
                velocity.linvel.y,
                velocity.angvel
            ));
            piece.last_update_vel = velocity.linvel;
        }*/
    }
}

fn frame_broadcast(broadcast : ResMut<Sender>, mut state : ResMut<GameState>, config : Res<GameConfig>, clients : Res<ClientMap>) {
    if state.playing {
        state.tick += 1;
        if state.tick > state.time_in_stage {
            state.strategy = !state.strategy;
            if state.strategy {
                state.time_in_stage = config.strategy_period;
            }
            else {
                state.time_in_stage = config.play_period;
            }
            state.tick = 0;
        }
    }
    else {
        if state.currently_attached_players >= config.min_player_slots {
            state.tick += 1;
        }
        else {
            state.tick = 0;
        }
        if state.tick > state.time_in_stage {
            state.playing = true;
        }
    }
    let _ = broadcast.send(ServerMessage::GameState (state.get_state_byte(), state.tick, state.time_in_stage));
}

#[derive(Debug)]
enum PlaceType {
    BasicFighter,
    Castle
}

#[derive(Event)]
struct PlaceEvent {
    x : f32,
    y : f32,
    a : f32,
    owner : u64,
    slot : u8,
    tp : PlaceType,
    free : bool // do we need to fabber check this one? if free is set to true, fabber and territory checks are skipped
}

fn make_thing(mut commands : Commands, broadcast : ResMut<Sender>, mut things : EventReader<PlaceEvent>, territories : Query<(&GamePiece, &Transform, &Fabber)>) {
    for ev in things.read() {
        let mut transform = Transform::from_xyz(ev.x, ev.y, 0.0);
        transform.rotate_z(ev.a);
        let mut piece = commands.spawn((RigidBody::Dynamic, Velocity::zero(), TransformBundle::from(transform), ExternalForce::default(), Damping {
            linear_damping : 0.0,
            angular_damping : 0.0
        }));
        // fabber check
        let mut isfab = false;
        println!("placing of type {:?}", ev.tp);
        if ev.free {
            isfab = true;
        }
        else {
            for (territory_holder, position, fabber) in territories.iter() {
                let d_x = position.translation.x - ev.x;
                let d_y = position.translation.y - ev.y;
                let dist = d_x * d_x + d_y * d_y;
                println!("distance from territory (cl {}, slot {}) is {}", territory_holder.owner, territory_holder.slot, dist);
                if dist < fabber.radius * fabber.radius {
                    if territory_holder.owner == ev.owner {
                        isfab = true;
                    }
                    if territory_holder.slot > 1 && ev.slot == territory_holder.slot {
                        isfab = true;
                    }
                }
            }
        }
        if !isfab {
            continue;
        }
        let t_num : u16 = match ev.tp {
            PlaceType::BasicFighter => {
                piece.insert(Collider::cuboid(20.5, 20.5));
                piece.insert(PathFollower::start(ev.x, ev.y));
                BASIC_FIGHTER_TYPE_NUM
            },
            PlaceType::Castle => {
                piece.insert((Collider::cuboid(60.0, 60.0), Territory::castle(), Fabber::castle()));
                CASTLE_TYPE_NUM
            }
        };
        piece.insert(GamePiece::new(t_num, ev.owner, ev.slot));
        let _ = broadcast.send(ServerMessage::ObjectCreate(ev.x, ev.y, ev.a, ev.owner, piece.id().index(), t_num));
    }
}

fn setup(mut commands : Commands, mut state : ResMut<GameState>, config : Res<GameConfig>) {
    state.tick = 0;
    state.time_in_stage = config.wait_period;
}

#[derive(Resource, Deref, DerefMut)]
struct ClientMap(HashMap<u64, Client>);

#[derive(Resource, Deref, DerefMut)] // todo: better names (or generic type arguments)
struct Receiver(mpsc::Receiver<Comms>);

#[derive(Resource, Deref, DerefMut)]
struct Sender(broadcast::Sender<ServerMessage>);


#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PhysicsSchedule;


use std::time::Duration;

fn run_physics_schedule(world : &mut World) {
    let state = world.get_resource::<GameState>().expect("gamestate resource not loaded!");
    if state.playing && !state.strategy {
        world.run_schedule(PhysicsSchedule);
    }
}


#[tokio::main]
async fn main() {
    let top_id = Arc::new(Mutex::new(1_u64)); // POSSIBLE BUG: if the client id goes beyond 18,446,744,073,709,551,615, it may overflow and assign duplicate IDs
    // this is not likely to be a real problem
    let (to_bevy_tx, to_bevy_rx) = mpsc::channel::<Comms>(128);
    let (from_bevy_broadcast_tx, _) = broadcast::channel::<ServerMessage>(128);
    let bevy_broadcast_tx_cloner = from_bevy_broadcast_tx.clone();
    let websocket = warp::path("game")
        .and(warp::ws())
        .and(warp::any().map(move || {
            top_id.clone()
        }))
        .and(warp::any().map(move || {
            to_bevy_tx.clone()
        }))
        .and(warp::any().map(move || {
            bevy_broadcast_tx_cloner.subscribe()
        }))
        .map(|ws : warp::ws::Ws, top_id : Arc<Mutex<u64>>, to_bevy : mpsc::Sender<Comms>, mut from_bevy_broadcast : broadcast::Receiver<ServerMessage>| {
            ws.max_frame_size(MAX_FRAME_SIZE).on_upgrade(|client| async move {
                let mut topid = top_id.lock().await;
                let my_id : u64 = *topid;
                *topid += 1;
                drop(topid);
                let (mut client_tx, mut client_rx) = client.split();
                let (from_bevy_tx, mut from_bevy_rx) = tokio::sync::mpsc::channel(10);
                let mut me_verified = false;
                let mut cl = Some(Client {
                    has_placed_castle : false,
                    id : my_id,
                    slot : 0,
                    channel : from_bevy_tx
                });
                if let Err(_) = client_tx.send(warp::ws::Message::binary(ServerMessage::Test("EXOSPHERE".to_string(), 128, 4096, 115600, 123456789012345, -64, -4096, -115600, -123456789012345, -4096.512, -8192.756, VERSION).encode())).await {
                    println!("client disconnected before handshake");
                    return;
                }
                'cli_loop: loop {
                    tokio::select!{
                        message = client_rx.next() => {
                            match message {
                                Some(msg) => {
                                    if let Ok(msg) = msg {
                                        if msg.is_binary() {
                                            if let Ok(frame) = ClientMessage::decode_from(&msg.as_bytes()) {
                                                if me_verified {
                                                    if let Err(_) = to_bevy.send(Comms::MessageFrom(my_id, frame)).await {
                                                        println!("channel failure 1: lost connection to game engine");
                                                        break 'cli_loop;
                                                    }
                                                }
                                                else {
                                                    if frame == ClientMessage::Test("EXOSPHERE".to_string(), 128, 4096, 115600, 123456789012345, -64, -4096, -115600, -123456789012345, -4096.512, -8192.756, VERSION) {
                                                        let cl_out = cl.take().expect("fatal: apparent reuse of client (this code path should NEVER be called!)");
                                                        if let Err(_) = to_bevy.send(Comms::ClientConnect(cl_out)).await {
                                                            println!("channel failure 1.125: lost connection to game engine");
                                                            break 'cli_loop;
                                                        }
                                                        me_verified = true;
                                                    }
                                                    else {
                                                        println!("client failed verification");
                                                        break 'cli_loop;
                                                    }
                                                }
                                            }
                                            else {
                                                println!("channel failure 1.25: malformed frame");
                                                break 'cli_loop;
                                            }
                                        }
                                    }
                                    else {
                                        println!("channel failure 2");
                                        break 'cli_loop;
                                    }
                                }
                                None => {
                                    if me_verified {
                                        if let Err(_) = to_bevy.send(Comms::ClientDisconnect(my_id)).await {
                                            println!("channel failure 3: lost connection to game engine");
                                        }
                                    }
                                    else {
                                        println!("client disconnect before completion of handshake");
                                        break 'cli_loop;
                                    }
                                    break 'cli_loop;
                                }
                            }
                        },
                        msg = from_bevy_rx.recv() => {
                            match msg {
                                Some(msg) => {
                                    if let Err(_) = client_tx.send(warp::ws::Message::binary(msg.encode())).await {
                                        println!("channel failure 4");
                                        break 'cli_loop;
                                    }
                                }
                                None => {
                                    println!("channel failure 5: connection to game engine broken");
                                    break 'cli_loop;
                                }
                            }
                        },
                        msg = from_bevy_broadcast.recv() => {
                            match msg {
                                Ok(msg) => {
                                    if let Err(_) = client_tx.send(warp::ws::Message::binary(msg.encode())).await {
                                        println!("channel failure 6");
                                        break 'cli_loop;
                                    }
                                }
                                Err(_) => {
                                    println!("broadcast channel failure. This is likely fatal.");
                                    break 'cli_loop;
                                }
                            }
                        }
                    }
                }
            })
        });
    tokio::task::spawn(warp::serve(websocket).run(([0,0,0,0], 3000)));
    let mut config = RapierConfiguration::new(100.0);
    config.gravity = Vec2 { x : 0.0, y : 0.0 };
    App::new()
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().with_default_system_setup(false))
        .add_systems(
            PhysicsSchedule,
            (
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend)
                    .in_set(PhysicsSet::SyncBackend),
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::StepSimulation)
                    .in_set(PhysicsSet::StepSimulation),
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback)
                    .in_set(PhysicsSet::Writeback),
            ),
        )
        .init_schedule(PhysicsSchedule)
        .edit_schedule(PhysicsSchedule, |schedule| {
            schedule.configure_sets((
                PhysicsSet::SyncBackend,
                PhysicsSet::StepSimulation,
                PhysicsSet::Writeback
            ).chain());
        })
        .add_event::<NewClientEvent>()
        .add_event::<PlaceEvent>()
        .insert_resource(config)
        .insert_resource(ClientMap(HashMap::new()))
        .add_plugins(bevy_time::TimePlugin)
        .insert_resource(Receiver(to_bevy_rx))
        .insert_resource(Sender(from_bevy_broadcast_tx))
        .insert_resource(GameConfig {
            width: 1000.0,
            height: 1000.0,
            wait_period: 10 * UPDATE_RATE as u16,
            play_period: 10 * UPDATE_RATE as u16,
            strategy_period: 10 * UPDATE_RATE as u16,
            max_player_slots: 1000,
            min_player_slots: 2
        })
        .insert_resource(GameState {
            playing : false,
            io : false,
            strategy : true,
            tick : 0,
            time_in_stage : 0,
            currently_attached_players : 0
        })
        .add_systems(PreUpdate, run_physics_schedule)
        .add_systems(Update, (client_tick, send_objects, position_updates, frame_broadcast.before(position_updates), make_thing))
        .add_systems(Startup, setup)
        .set_runner(|mut app| {
            loop {
                let start = std::time::Instant::now();
                app.update();
                let time_elapsed = start.elapsed();
                if time_elapsed < FRAME_TIME {
                    let time_remaining = FRAME_TIME - time_elapsed;
                    std::thread::sleep(time_remaining);
                }
            }
        })
        .run();
}
