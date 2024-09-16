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
use bevy_rapier2d::dynamics::ReadMassProperties;
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

pub mod solve_spaceship;
use solve_spaceship::*;

pub mod pathfollower;
use pathfollower::*;


const UPDATE_RATE : u64 = 30; // 30hz by default
const FRAME_TIME : std::time::Duration = std::time::Duration::from_millis(1000 / UPDATE_RATE); // milliseconds per frame

const MAX_FRAME_SIZE : usize = 1024; // maximum size of an incoming websocket frame

const VERSION : u8 = 0; // bump this up every time a major change is made

const BASIC_FIGHTER_TYPE_NUM : u16 = 0;
const CASTLE_TYPE_NUM : u16 = 1;
const BULLET_TYPE_NUM : u16 = 2;


#[derive(Debug, ProtocolRoot, PartialEq)]
enum ClientMessage {
    Test(String, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, u8),
    Connect(String, String), // connect with your nickname and the password respectively. doesn't let you place your castle yet.
    // passwords are, like in MMOSG, used for various things: they can grant entry into a server, they can assign your team, etc. In io games they are usually ignored.
    PlacePiece(f32, f32, u16), // x, y, type
    // attempt to place an object
    // before the client can place anything else, it must place a castle (type 1). this is the only time in the game that a client can place an object in neutral territory.
    // obviously it's not possible to place a castle in enemy territory
    StrategyPointAdd(u32, u16, f32, f32), // (id, index, x, y) insert a point to a strategy path at an index
    StrategyPointUpdate(u32, u16, f32, f32), // (id, index, x, y) move a point on a strategy path
    StrategyRemove(u32, u16), // (id, index) remove a node from a strategy
    StrategyClear(u32), // (id) clear a strategy
    // if any Strategy commands are sent referencing nonexistent nodes on a strategy, or StrategyPointUpdate is sent referencing a non-point strategy node (such
    // as a teleportal entrance), the server will simply ignore them. This may be a problem in the future.
    StrategySetEndcapRotation(u32, f32), // (id, r) set the strategy endcap for an object to a rotation
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
    Metadata(u64, f32, f32, u8), // send whatever data (id, board width x height, slot) the client needs to begin rendering the gameboard
    // this also tells the client that it was accepted (e.g. got the right password); getting the password _wrong_ would abort the connection
    // slot tells the client what position it's occupying. 0 = spectating, 1 = free agent, 2-255 = teams.
    ObjectCreate(f32, f32, f32, u64, u32, u16), // x, y, a, owner, id, type: inform the client of an object.
    ObjectMove(u32, f32, f32, f32), // id, x, y, a
    ObjectTrajectoryUpdate(u32, f32, f32, f32, f32, f32, f32), // id, x, y, a, xv, yv, av
    DeleteObject(u32), // id
    StrategyCompletion(u32, u16), // (id, remaining) a node in a strategy has been fulfilled, and this is the number of strategy nodes remaining!
    // this serves as a sort of checksum; if the number of strategy nodes remaining here is different from the number of strategy nodes remaining in the client
    // the client knows there's something wrong and can send StrategyClear to attempt to recover.
    PlayerData(u64, String, u8), // (id, banner, slot): data on another player
    YouLose, // you LOST!
    Winner(u64), // id of the winner. if 0, it was a tie.
    Territory(u32, f32), // set the territory radius for a piece
    Fabber(u32, f32), // set the fabber radius for a piece
    Disconnect
}


// upon connecting, the server immediately sends the client Test("EXOSPHERE", 128, 4096, 115600, 123456789012345, -64, -4096, -115600, -123456789012345, -4096.512, -8192.756, VERSION)
// if the client can successfully decode the message, and VERSION matches the client version, the game may proceed. Otherwise, the client will immediately drop the connection.
// to signify that the test passed, the client will send a Test() with the same values and the client version. If the server can successfully decode them, and the version the
// client sends matches VERSION, the game may proceed. Otherwise, the server will abort the connection.
// this exchange prevents old, underprepared, or incompatible clients from connecting to a game.
// If a client attempts to do anything before protocol verification, it will be kicked off the server.


struct Client {
    id : u64,
    banner : String,
    slot : u8,
    channel : std::sync::Mutex<tokio::sync::mpsc::Sender<ServerMessage>>,
    has_placed_castle : bool,
    alive : bool,
    money : u32 // if I make it a u16 richard will crash the server by somehow farming up >66k money
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
    currently_attached_players : u16, // the number of players CONNECTED
    currently_playing : u16 // the number of players with territory
}


impl GameState {
    fn get_state_byte(&self) -> u8 { // todo: use bit shifting
        self.io as u8 * 128 + self.playing as u8 * 64 + self.strategy as u8 * 32
    }
}


impl Client {
    fn send(&self, msg : ServerMessage) {
        if let Ok(lock) = self.channel.lock() {
            if let Err(_) = lock.try_send(msg) {
                println!("failed to send message on channel");
            }
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
    last_update_pos : Vec2,
    last_update_ang : f32,
    health : f32
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
            radius : 500.0,
            l_missiles : 4,
            l_ships : 2,
            l_econ : 2,
            l_defense : 3,
            l_buildings : 2
        }
    }

    fn is_available(&self, tp : PlaceType) -> bool { // determine if this fabber can produce an object given its numerical identifier
        match tp {
            PlaceType::BasicFighter => self.l_ships >= 1,
            PlaceType::Castle => false, // fabbers can never place castles
        }
    }
}


#[derive(Component)]
struct Ship {
    speed : f32
}

impl Ship {
    fn normal() -> Self {
        return Self {
            speed : 16.0
        }
    }
}


impl GamePiece {
    fn new(type_indicator : u16, owner : u64, slot : u8, health : f32) -> Self {
        Self {
            type_indicator,
            owner,
            slot,
            last_update_pos : Vec2 {
                x : 0.0,
                y : 0.0
            },
            last_update_ang : 0.0,
            health : health
        }
    }
}


enum Bullets {
    MinorBullet(u16) // simple bullet with range
}

#[derive(Component)]
struct TimeToLive {
    lifetime : u16
}


#[derive(Component)]
struct Bullet {} // bullet collision semantics
// normal collisions between entities are only destructive if greater than a threshold
// bullet collisions are always destructive


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


impl Gun {
    fn mediocre() -> Self {
        Self {
            enabled : true,
            cd : 20,
            bullets : Bullets::MinorBullet(40),
            repeats : 0,
            repeat_cd : 0,
            r_point : 0,
            tick : 0
        }
    }
}


fn shoot(mut commands : Commands, mut pieces : Query<(&Transform, &Velocity, &mut Gun)>, broadcast : ResMut<Sender>) {
    for (position, velocity, mut gun) in pieces.iter_mut() {
        if gun.enabled {
            gun.tick += 1;
            if gun.tick > gun.cd {
                gun.tick = 0;
                match gun.bullets {
                    Bullets::MinorBullet(range) => {
                        let mut transform = position.clone();
                        let ang = transform.rotation.to_euler(EulerRot::ZYX).0;
                        transform.translation += (Vec2::from_angle(ang) * 40.0).extend(0.0);
                        let piece = commands.spawn((GamePiece::new(BULLET_TYPE_NUM, 0, 0, 1.0), RigidBody::Dynamic, Collider::cuboid(2.5, 2.5), Velocity::linear(velocity.linvel + Vec2::from_angle(ang) * 400.0), TransformBundle::from(transform), Damping {
                            linear_damping : 0.0,
                            angular_damping : 0.0
                        }, TimeToLive { lifetime : range }, Bullet {}, ActiveEvents::COLLISION_EVENTS));
                        let _ = broadcast.send(ServerMessage::ObjectCreate(transform.translation.x, transform.translation.y, ang, 0, piece.id().index(), BULLET_TYPE_NUM));
                    }
                }
            }
        }
    }
}


fn ttl(mut commands : Commands, mut expirees : Query<(Entity, &mut TimeToLive)>, broadcast : ResMut<Sender>) {
    for (entity, mut ttl) in expirees.iter_mut() {
        if ttl.lifetime == 0 {
            commands.entity(entity).despawn();
            if let Err(_) = broadcast.send(ServerMessage::DeleteObject(entity.index())) {
                println!("game engine lost connection to webserver. this is probably not critical.");
            }

        }
        else {
            ttl.lifetime -= 1;
        }
    }
}


fn handle_collisions(mut commands : Commands, mut collision_events: EventReader<CollisionEvent>, mut pieces : Query<(Entity, &mut GamePiece, Option<&Bullet>)>, broadcast : ResMut<Sender>, mut client_killed : EventWriter<ClientKilledEvent>) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(one, two, _) = event {
            let mut one_dmg : f32 = 0.0; // damage to apply to entity 1
            let mut two_dmg : f32 = 0.0; // damage to apply to entity 2
            if let Ok((_, piece_one, bullet_one)) = pieces.get(*one) {
                if let Ok((_, piece_two, bullet_two)) = pieces.get(*two) {
                    if bullet_one.is_some() { // if either one of these is a bullet, the collision is *fully destructive* - at least the bullet will be completely destroyed.
                        two_dmg = piece_one.health;
                        one_dmg = piece_one.health;
                    }
                    if bullet_two.is_some() {
                        one_dmg += piece_two.health;
                        two_dmg += piece_one.health;
                    }
                }
            }
            if one_dmg != 0.0 {
                if let Ok((entity_one, mut piece_one, _)) = pieces.get_mut(*one) {
                    piece_one.health -= one_dmg;
                    if piece_one.health <= 0.0 {
                        if piece_one.type_indicator == CASTLE_TYPE_NUM {
                            client_killed.send(ClientKilledEvent { client : piece_one.owner });
                            println!("possible client kill");
                        }
                        commands.entity(entity_one).despawn();
                        if let Err(_) = broadcast.send(ServerMessage::DeleteObject(entity_one.index())) {
                            println!("game engine lost connection to webserver. this is probably not critical.");
                        }
                    }
                }
            }
            if two_dmg != 0.0 {
                if let Ok((entity_two, mut piece_two, _)) = pieces.get_mut(*two) {
                    piece_two.health -= two_dmg;
                    if piece_two.health <= 0.0 {
                        if piece_two.type_indicator == CASTLE_TYPE_NUM {
                            client_killed.send(ClientKilledEvent { client : piece_two.owner });
                            println!("possible client kill");
                        }
                        commands.entity(entity_two).despawn();
                        if let Err(_) = broadcast.send(ServerMessage::DeleteObject(entity_two.index())) {
                            println!("game engine lost connection to webserver. this is probably not critical.");
                        }
                    }
                }
            }
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

fn nanor(thing : f32, or : f32) -> f32 {
    if thing.is_nan() {
        return or;
    }
    return thing;
}


fn move_ships(mut ships : Query<(&mut ExternalForce, &mut ExternalImpulse, &Velocity, &Transform, &Ship, &mut PathFollower, &GamePiece, &Collider, Entity)>, mut clients : ResMut<ClientMap>) {
    for (mut force, mut impulse, velocity, transform, ship, mut follower, piece, collider, entity) in ships.iter_mut() {
        if let Some(next) = follower.get_next() {
            let gpos = match next {
                PathNode::StraightTo(x, y) => Vec2 { x, y },
                PathNode::Teleportal(_, _) => Vec2::ZERO // TODO: IMPLEMENT
            };
            let cpos = transform.translation.truncate();
            let inv_mass = collider.raw.mass_properties(1.0).inv_mass;
            let cangle = transform.rotation.to_euler(EulerRot::ZYX).0;
            if (gpos - cpos).length() > 15.0 {
                impulse.impulse = if loopify(cangle, (gpos - cpos).to_angle()).abs() < PI / 6.0 {
                    Vec2::from_angle(cangle) / inv_mass * linear_maneuvre(cpos, gpos, velocity.linvel, ship.speed * 10.0, ship.speed * 3.3)
                }
                else {
                    Vec2::ZERO
                };
                impulse.impulse -= velocity.linvel.project_onto((gpos - cpos).perp()) / inv_mass * 0.2; // linear deviation correction thrusters
                impulse.torque_impulse = (-loopify((gpos - cpos).to_angle(), cangle) * 10.0 - velocity.angvel * 2.0) / inv_mass * 40.0;
                //force.force = Vec2::from_angle(cangle) * (linear_maneuvre(cpos, gpos, velocity.linvel, ship.speed * 50.0, 250.0) / inv_mass);
                // this can produce odd effects at close approach, hence the normalizer code
                //println!("cangle: {}, gangle: {}, angvel: {}", cangle, (cpos - gpos).to_angle(), velocity.angvel);
            }
            else {
                force.force = Vec2::ZERO;
                force.torque = 0.0;
                impulse.impulse = velocity.linvel / inv_mass * -0.1;
                impulse.torque_impulse = 0.0;
                if follower.bump() {
                    if let Some(client) = clients.get_mut(&piece.owner) {
                        client.send(ServerMessage::StrategyCompletion(entity.index(), follower.len()));
                    }
                }
                else {
                    match follower.get_endcap() {
                        EndNode::Rotation(r) => {
                            impulse.torque_impulse = (-loopify(r, cangle) * 10.0 - velocity.angvel * 2.0) / inv_mass * 40.0;
                        }
                        EndNode::None => {}
                    }
                }
            }
        }
    }
}


fn client_tick(mut commands : Commands, mut pieces : Query<(Entity, &GamePiece, Option<&mut PathFollower>, Option<&Transform>, Option<&Territory>)>, mut ev_newclient : EventWriter<NewClientEvent>, mut place : EventWriter<PlaceEvent>, mut state : ResMut<GameState>, config : Res<GameConfig>, mut clients : ResMut<ClientMap>, mut receiver : ResMut<Receiver>, broadcast : ResMut<Sender>, mut client_killed : EventWriter<ClientKilledEvent>) {
    // manage events from network-connected clients
    loop { // loops receiver.try_recv(), until it returns empty
        match receiver.try_recv() {
            Ok(message) => {
                match message {
                    Comms::ClientConnect(cli) => {
                        clients.insert(cli.id, cli);
                    },
                    Comms::ClientDisconnect(id) => {
                        println!("client disconnected. cleaning up!");
                        for (entity, piece, _, _, _) in pieces.iter() {
                            if piece.owner == id {
                                commands.entity(entity).despawn();
                                if let Err(_) = broadcast.send(ServerMessage::DeleteObject(entity.index())) {
                                    println!("game engine lost connection to webserver. this is probably not critical.");
                                }
                            }
                        }
                        state.currently_attached_players -= 1;
                        clients.remove(&id);
                        client_killed.send(ClientKilledEvent { client : id });
                    },
                    Comms::MessageFrom(id, msg) => {
                        let mut kill = false;
                        if clients.contains_key(&id) {
                            match msg {
                                ClientMessage::Connect(banner, password) => {
                                    let slot : u8 = if state.currently_attached_players < config.max_player_slots { 1 } else { 0 }; // todo: implement teams
                                    for k in clients.keys() {
                                        if *k != id {
                                            let message = ServerMessage::PlayerData(*k, clients[k].banner.clone(), clients[k].slot);
                                            clients[&id].send(message);
                                        }
                                    }
                                    state.currently_playing += 1;
                                    clients.get_mut(&id).unwrap().send(ServerMessage::Metadata(id, config.width, config.height, slot));
                                    if let Err(_) = broadcast.send(ServerMessage::PlayerData(id, banner.clone(), slot)) {
                                        println!("couldn't broadcast player data");
                                    }
                                    state.currently_attached_players += 1;
                                    clients.get_mut(&id).unwrap().slot = slot;
                                    clients.get_mut(&id).unwrap().banner = banner;
                                    ev_newclient.send(NewClientEvent {id});
                                },
                                ClientMessage::PlacePiece(x, y, t) => {
                                    if t == CASTLE_TYPE_NUM {
                                        if (!state.playing || state.io) {
                                            if clients[&id].has_placed_castle {
                                                println!("client attempted to place an extra castle. dropping.");
                                                kill = true;
                                            }
                                            else {
                                                let mut is_okay = true;
                                                for (_, _, _, transform, territory) in pieces.iter() {
                                                    if let Some(transform) = transform {
                                                        if let Some(territory) = territory {
                                                            let dx = transform.translation.x - x;
                                                            let dy = transform.translation.y - y;
                                                            let d = (dx * dx + dy * dy).sqrt();
                                                            if d < territory.radius + 600.0 { // if the territories would intersect
                                                                is_okay = false;
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                                if is_okay {
                                                    clients.get_mut(&id).unwrap().has_placed_castle = true;
                                                    clients.get_mut(&id).unwrap().alive = true;
                                                    let slot = clients[&id].slot;
                                                    place.castle(x, y, id, slot);
                                                    place.basic_fighter_free(x - 200.0, y, PI, id, slot);
                                                    place.basic_fighter_free(x + 200.0, y, 0.0, id, slot);
                                                    place.basic_fighter_free(x, y - 200.0, 0.0, id, slot);
                                                    place.basic_fighter_free(x, y + 200.0, 0.0, id, slot);
                                                }
                                            }
                                        }
                                    }
                                    else if state.playing && state.strategy {
                                        match t {
                                            _ => {
                                                println!("client attempted to place unknown type {}. dropping.", t);
                                                kill = true;
                                            }
                                        }
                                    }
                                },
                                ClientMessage::StrategyPointAdd(piece_id, index, x, y) => {
                                    if state.playing && state.strategy {
                                        for (entity, piece, pathfollower, _, _) in pieces.iter_mut() {
                                            if entity.index() == piece_id { // TODO: FIX THIS
                                                // THIS IS REALLY BAD
                                                // REALLY REALLY REALLY BAD
                                                // WE'RE DOING LINEAR TIME LOOKUPS WHERE A CONSTANT TIME LOOKUP WOULD SUFFICE AND WELL
                                                // FIIIIIIIIIIIIIIIIIIX THISSSSSSSSSS
                                                if let Some(mut pathfollower) = pathfollower {
                                                    if piece.owner == id {
                                                        pathfollower.insert_point(index, x, y);
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
                                    }
                                },
                                ClientMessage::StrategyClear(piece_id) => {
                                    if state.playing && state.strategy {
                                        for (entity, piece, pathfollower, _, _) in pieces.iter_mut() {
                                            if entity.index() == piece_id { // see above
                                                if let Some(mut pathfollower) = pathfollower {
                                                    if piece.owner == id {
                                                        pathfollower.clear();
                                                    }
                                                }
                                                else {
                                                    println!("whoops");
                                                }
                                            }
                                        }
                                    }
                                },
                                ClientMessage::StrategyPointUpdate(piece_id, index, x, y) => {
                                    if state.playing && state.strategy {
                                        for (entity, piece, pathfollower, _, _) in pieces.iter_mut() {
                                            if entity.index() == piece_id { // see above
                                                if let Some(mut pathfollower) = pathfollower {
                                                    if piece.owner == id {
                                                        pathfollower.update_point(index, x, y);
                                                    }
                                                }
                                                else {
                                                    println!("whoops");
                                                }
                                            }
                                        }
                                    }
                                },
                                ClientMessage::StrategyRemove(piece_id, index) => {
                                    if state.playing && state.strategy {
                                        for (entity, piece, pathfollower, _, _) in pieces.iter_mut() {
                                            if entity.index() == piece_id { // see above
                                                if let Some(mut pathfollower) = pathfollower {
                                                    if piece.owner == id {
                                                        pathfollower.remove_node(index);
                                                    }
                                                }
                                                else {
                                                    println!("whoops");
                                                }
                                            }
                                        }
                                    }
                                },
                                ClientMessage::StrategySetEndcapRotation(piece_id, r) => {
                                    if state.playing && state.strategy {
                                        for (entity, piece, pathfollower, _, _) in pieces.iter_mut() {
                                            if entity.index() == piece_id { // see above
                                                if let Some(mut pathfollower) = pathfollower {
                                                    if piece.owner == id {
                                                        pathfollower.set_endcap_rotation(r);
                                                    }
                                                }
                                                else {
                                                    println!("whoops");
                                                }
                                            }
                                        }
                                    }
                                }
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

fn send_objects(mut events : EventReader<NewClientEvent>, mut clients : ResMut<ClientMap>, objects : Query<(Entity, &GamePiece, &Transform, Option<&Territory>, Option<&Fabber>)>) {
    for ev in events.read() {
        if let Some(client) = clients.get_mut(&ev.id) {
            for (entity, piece, transform, territory, fabber) in objects.iter() {
                client.send(ServerMessage::ObjectCreate(transform.translation.x, transform.translation.y, transform.rotation.to_euler(EulerRot::ZYX).0, piece.owner, entity.index(), piece.type_indicator));
                if let Some(territory) = territory {
                    client.send(ServerMessage::Territory(entity.index(), territory.radius));
                }
                if let Some(fabber) = fabber {
                    client.send(ServerMessage::Fabber(entity.index(), fabber.radius));
                }
            }
        }
    }
}

fn position_updates(broadcast : ResMut<Sender>, mut objects : Query<(Entity, &mut GamePiece, &Transform)>) {
    for (entity, mut piece, transform) in objects.iter_mut() {
        // todo: only send position updates if it's moving
        let pos = transform.translation.truncate();
        let ang = transform.rotation.to_euler(EulerRot::ZYX).0;
        // updates on position
        if (pos - piece.last_update_pos).length() > 1.0 || loopify(ang, piece.last_update_ang).abs() > 0.01 {
            // are basically straight lines.
            let _ = broadcast.send(ServerMessage::ObjectMove( // ignore the errors
                entity.index(),
                pos.x,
                pos.y,
                transform.rotation.to_euler(EulerRot::ZYX).0
            ));
            piece.last_update_pos = pos;
            piece.last_update_ang = ang;
        }
    }
}

fn frame_broadcast(broadcast : ResMut<Sender>, mut state : ResMut<GameState>, config : Res<GameConfig>) {
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
        if state.currently_playing >= config.min_player_slots {
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

#[derive(Copy, Clone, Debug)]
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

fn make_thing(mut commands : Commands, broadcast : ResMut<Sender>, mut things : EventReader<PlaceEvent>, territories : Query<(&GamePiece, &Transform, Option<&Fabber>, Option<&Territory>)>) {
    'evloop: for ev in things.read() {
        let mut transform = Transform::from_xyz(ev.x, ev.y, 0.0);
        transform.rotate_z(ev.a);
        let mut piece = commands.spawn((RigidBody::Dynamic, Velocity::zero(), TransformBundle::from(transform), ExternalForce::default(), ExternalImpulse::default(), Damping {
            linear_damping : 0.0,// todo: clear out unnecessary components (move them to the match statement so we don't have, say, ExternalImpulse on a static body)
            angular_damping : 0.0
        }));
        // fabber check
        let mut isfab = false;
        println!("placing of type {:?}", ev.tp);
        if ev.free {
            isfab = true;
        }
        else {
            for (territory_holder, position, fabber, territory) in territories.iter() {
                let d_x = position.translation.x - ev.x;
                let d_y = position.translation.y - ev.y;
                let dist = d_x * d_x + d_y * d_y;
                if let Some(fabber) = fabber {
                    println!("distance from territory (cl {}, slot {}) is {}", territory_holder.owner, territory_holder.slot, dist);
                    if dist < fabber.radius * fabber.radius && fabber.is_available(ev.tp) {
                        if territory_holder.owner == ev.owner {
                            isfab = true;
                        }
                        if territory_holder.slot > 1 && ev.slot == territory_holder.slot {
                            isfab = true;
                        }
                    }
                }
                if let Some(territory) = territory {
                    if let PlaceType::Castle = ev.tp {
                        if dist.sqrt() < territory.radius + 600.0 {
                            if territory_holder.owner != ev.owner && (territory_holder.slot == 1 || territory_holder.slot != ev.slot) {
                                println!("too close!");
                                piece.despawn();
                                continue 'evloop;
                            }
                        }
                    }
                }
            }
        }
        if !isfab {
            piece.despawn();
            continue;
        }
        let health : f32;
        let t_num : u16 = match ev.tp {
            PlaceType::BasicFighter => {
                piece.insert((Collider::cuboid(20.5, 20.5), PathFollower::start(ev.x, ev.y), Ship::normal(), ReadMassProperties::default(), Gun::mediocre()));
                health = 3.0;
                BASIC_FIGHTER_TYPE_NUM
            },
            PlaceType::Castle => {
                let terr = Territory::castle();
                let fab = Fabber::castle();
                let _ = broadcast.send(ServerMessage::Territory(piece.id().index(), terr.radius));
                let _ = broadcast.send(ServerMessage::Fabber(piece.id().index(), fab.radius));
                piece.insert((Collider::cuboid(30.0, 30.0), terr, fab));
                health = 6.0;
                CASTLE_TYPE_NUM
            }
        };
        piece.insert(GamePiece::new(t_num, ev.owner, ev.slot, health));
        let _ = broadcast.send(ServerMessage::ObjectCreate(ev.x, ev.y, ev.a, ev.owner, piece.id().index(), t_num));
    }
}

fn setup(mut state : ResMut<GameState>, config : Res<GameConfig>) {
    // todo: construct board (walls, starting rubble, etc)
    state.tick = 0;
    state.time_in_stage = config.wait_period;
}


#[derive(Event)]
struct ClientKilledEvent { // something happened that could have killed a client
    // we'll healthcheck to see if the client actually died and update game state accordingly
    client : u64
}


fn client_health_check(mut commands : Commands, mut events : EventReader<ClientKilledEvent>, mut clients : ResMut<ClientMap>, pieces : Query<(Option<&Territory>, &GamePiece, Entity)>, mut state : ResMut<GameState>, config : Res<GameConfig>, mut broadcast : ResMut<Sender>) {
    // checks:
    // * if the client is still present (if the client disconnected, it's dead by default!), exit early
    // * if the client has any remaining Territory, it's not dead, false alarm
    // if we determined that the client is in fact dead, send a Lose message and update the state accordingly.
    // At the end, if there is 1 or 0 players left, send a Win broadcast as appropriate and reset the state for the next game.
    if !state.playing {
        return; // client kill can't happen during wait mode
    }
    let mut did_something = false;
    for ev in events.read() {
        if !clients.contains_key(&ev.client) { // if the client's already disconnected, we can't exactly tell them they lost
            state.currently_playing -= 1;
        }
        else {
            let mut has_territory = false;
            for (territory, piece, _) in pieces.iter() {
                if territory.is_some() && piece.owner == ev.client {
                    has_territory = true; 
                }
            }
            if !has_territory {
                state.currently_playing -= 1;
                clients[&ev.client].send(ServerMessage::YouLose);
                clients.get_mut(&ev.client).unwrap().alive = false;
            }
        }
        did_something = true;
    }
    if did_something { // only if we made a change does it make sense to update the state here
        if state.currently_playing < 2 {
            if state.currently_playing == 1 {
                let mut winid : u64 = 0;
                for (id, client) in clients.iter() {
                    if client.alive {
                        winid = *id;
                        break;
                    }
                }
                for (id, client) in clients.iter() {
                    client.send(ServerMessage::Winner(winid));
                    client.send(ServerMessage::Disconnect);
                }
            }
            state.playing = false;
            state.strategy = false;
            state.tick = 0;
            state.time_in_stage = config.wait_period;
            state.currently_attached_players = 0;
            state.currently_playing = 0;
            for (_, _, entity) in pieces.iter() {
                // todo: don't delete things that are supposed to stick around (like walls)
                commands.entity(entity).despawn();
            }
        }
    }
}


#[derive(Resource, Deref, DerefMut)]
struct ClientMap(HashMap<u64, Client>);

#[derive(Resource, Deref, DerefMut)] // todo: better names (or generic type arguments)
struct Receiver(mpsc::Receiver<Comms>);

#[derive(Resource, Deref, DerefMut)]
struct Sender(broadcast::Sender<ServerMessage>);


#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlaySchedule;


fn run_play_schedule(world : &mut World) {
    let state = world.get_resource::<GameState>().expect("gamestate resource not loaded!");
    if state.playing && !state.strategy {
        world.run_schedule(PlaySchedule);
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
                    banner : "None".to_string(),
                    slot : 0,
                    channel : std::sync::Mutex::new(from_bevy_tx),
                    alive : false
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
                                    if let ServerMessage::Disconnect = msg {
                                        let _ = client_tx.close().await;
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
            PlaySchedule,
            (
                move_ships, shoot, ttl,
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend)
                    .in_set(PhysicsSet::SyncBackend),
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::StepSimulation)
                    .in_set(PhysicsSet::StepSimulation),
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback)
                    .in_set(PhysicsSet::Writeback),
            ),
        )
        .init_schedule(PlaySchedule)
        .edit_schedule(PlaySchedule, |schedule| {
            schedule.configure_sets((
                PhysicsSet::SyncBackend,
                PhysicsSet::StepSimulation,
                PhysicsSet::Writeback
            ).chain());
        })
        .add_event::<NewClientEvent>()
        .add_event::<ClientKilledEvent>()
        .add_event::<PlaceEvent>()
        .insert_resource(config)
        .insert_resource(ClientMap(HashMap::new()))
        .add_plugins(bevy_time::TimePlugin)
        .insert_resource(Receiver(to_bevy_rx))
        .insert_resource(Sender(from_bevy_broadcast_tx))
        .insert_resource(GameConfig {
            width: 5000.0,
            height: 5000.0,
            wait_period: 20 * UPDATE_RATE as u16,
            play_period: 10 * UPDATE_RATE as u16,
            strategy_period: 10 * UPDATE_RATE as u16,
            max_player_slots: 1000,
            min_player_slots: 1
        })
        .insert_resource(GameState {
            playing : false,
            io : false,
            strategy : false,
            tick : 0,
            time_in_stage : 0,
            currently_attached_players : 0,
            currently_playing : 0
        })
        .add_systems(PreUpdate, run_play_schedule)
        .add_systems(Update, (client_tick,
            send_objects,
            position_updates,
            frame_broadcast.before(position_updates),
            make_thing,
            handle_collisions,
            client_health_check.before(handle_collisions))) // health checking should be BEFORE handle_collisions so there's a frame gap in which the entities are actually despawned
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
