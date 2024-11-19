/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// biiiiiiiiiiiiiiiiiiiiiiiiiiiiig TODO: split this up into a bunch of different files because JEEZ this is unreadable garbage
// TODO TODO TODO: fix the sensor leak: when a sensored thing dies, its sensor persists. the sensor doesn't do anything because of attachment checks, but it's still *there*
// wasting memory and cpu cycles.

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
use warp::Filter;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::{Mutex, mpsc, broadcast};
use std::sync::Arc;
use bevy::ecs::schedule::ScheduleLabel;
use rand::Rng;
use std::collections::HashMap;

use common::protocol::*;
use common::types::*;
use common::VERSION;
use common::PlayerId;
pub use common::comms;
use comms::{ ClientMessage, ServerMessage };


pub enum Comms { // webserver -> game engine
    ClientConnect(Client), // (client) a client connected
    ClientDisconnect(PlayerId), // (id) a client disconnected
    MessageFrom(PlayerId, ClientMessage) // (id, message) a client sent a message that was successfully decoded and filtered
}


pub mod solve_spaceship;
use solve_spaceship::*;

pub mod events;
use events::*;

pub mod consts;
use consts::*;

pub mod components;
use components::*;

mod systems;
use systems::*;

pub mod resources;
use resources::*;


pub struct Client {
    id : PlayerId,
    nickname : String,
    slot : u8,
    channel : std::sync::Mutex<tokio::sync::mpsc::Sender<ServerMessage>>,
    has_placed_castle : bool,
    alive : bool,
    money : u32, // if I make it a u16 richard will crash the server by somehow farming up >66k money
    connected : bool
}


impl Client {
    fn send(&self, msg : ServerMessage) {
        if let Ok(lock) = self.channel.lock() {
            if let Err(_) = lock.try_send(msg) {
                println!("failed to send message on channel");
            }
        }
    }

    fn collect(&mut self, amount : u32) {
        self.money += amount;
        self.send(ServerMessage::Money { id : self.id, amount : self.money });
    }

    fn charge(&mut self, amount : u32) -> bool { // returns if we actually successfully made the charge or not
        if self.money >= amount {
            self.money -= amount;
            self.send(ServerMessage::Money { id : self.id, amount : self.money });
            return true;
        }
        return false;
    }
}


#[derive(Copy, Clone)]
enum Bullets {
    MinorBullet(u16), // simple bullet with range
    Bomb(ExplosionProperties, u16) // properties of the explosion we're boutta detonate, range of the bullet
}


fn discharge_barrel(commands : &mut Commands, owner : PlayerId, barrel : u16, gun : &Gun, position : &Transform, velocity : &Velocity, broadcast : &ResMut<Sender>) {
    let ang = position.rotation.to_euler(EulerRot::ZYX).0;
    let vel = Velocity::linear(velocity.linvel + Vec2::from_angle(ang) * 450.0);
    let mut transform = position.clone();
    transform.translation += (Vec2::from_angle(ang) * gun.center_offset).extend(0.0);
    transform.translation += (Vec2::from_angle(ang).perp() * gun.barrel_spacing * (barrel as f32 - gun.barrels as f32 / 2.0 + 0.5)).extend(0.0);
    match gun.bullets {
        Bullets::MinorBullet(range) => {
            let piece = commands.spawn((GamePiece::new(PieceType::Bullet, owner, 0, 0.5), RigidBody::Dynamic, Collider::cuboid(2.5, 2.5), vel, TransformBundle::from(transform), Damping {
                linear_damping : 0.0,
                angular_damping : 0.0
            }, TimeToLive { lifetime : range }, Bullet { tp : gun.bullets }, ActiveEvents::COLLISION_EVENTS));
            let _ = broadcast.send(ServerMessage::ObjectCreate { x : transform.translation.x, y : transform.translation.y, a : ang, owner : PlayerId::SYSTEM, id : piece.id().into(), tp : PieceType::Bullet });
        },
        Bullets::Bomb(_, range) => {
            let piece = commands.spawn((GamePiece::new(PieceType::SmallBomb, owner, 0, 0.5), RigidBody::Dynamic, Collider::cuboid(5.0, 5.0), vel, TransformBundle::from(transform), Damping {
                linear_damping : 0.0,
                angular_damping : 0.0
            }, TimeToLive { lifetime : range }, Bullet { tp : gun.bullets }, ActiveEvents::COLLISION_EVENTS));
            let _ = broadcast.send(ServerMessage::ObjectCreate { x : transform.translation.x, y : transform.translation.y, a : ang, owner : PlayerId::SYSTEM, id : piece.id().into(), tp : PieceType::SmallBomb });
        }
    }
}


fn shoot(mut commands : Commands, mut pieces : Query<(&Transform, &Velocity, &GamePiece, &mut Gun)>, broadcast : ResMut<Sender>) {
    for (position, velocity, piece, mut gun) in pieces.iter_mut() {
        if gun.enabled {
            if gun.tick == 0 {
                gun.r_point += 1;
                if gun.r_point >= gun.repeats {
                    gun.tick = gun.cd;
                    gun.r_point = 0;
                }
                else {
                    gun.tick = gun.repeat_cd;
                }
                if gun.scatter_barrels {
                    discharge_barrel(&mut commands, piece.owner, rand::thread_rng().gen_range(0..gun.barrels), &gun, position, velocity, &broadcast);
                }
                else {
                    for barrel in 0..gun.barrels {
                        discharge_barrel(&mut commands, piece.owner, barrel, &gun, position, velocity, &broadcast);
                    }
                }
            }
            gun.tick -= 1;
        }
    }
}


fn ttl(mut expirees : Query<(Entity, &mut TimeToLive)>, mut kill_event : EventWriter<PieceDestroyedEvent>) {
    for (entity, mut ttl) in expirees.iter_mut() {
        if ttl.lifetime == 0 {
            kill_event.send(PieceDestroyedEvent { piece : entity, responsible : PlayerId::SYSTEM });
        }
        else {
            ttl.lifetime -= 1;
        }
    }
}


fn on_piece_dead(mut commands : Commands, broadcast : ResMut<Sender>, pieces : Query<&GamePiece>, bullets : Query<(&Bullet, &Transform)>, chests : Query<&Chest>, mut events : EventReader<PieceDestroyedEvent>, mut explosions : EventWriter<ExplosionEvent>, mut client_kill : EventWriter<ClientKilledEvent>, mut clients : ResMut<ClientMap>) {
    for evt in events.read() {
        if let Ok(piece) = pieces.get(evt.piece) {
            if let Ok((bullet, pos)) = bullets.get(evt.piece) {
                if let Bullets::Bomb(explosion, _) = bullet.tp {
                    explosions.send(ExplosionEvent {
                        x : pos.translation.x,
                        y : pos.translation.y,
                        props : explosion
                    });
                }
            }
            if let Ok(_) = chests.get(evt.piece) {
                if let Some(cl) = clients.get_mut(&evt.responsible) {
                    cl.collect(20); // kill the chest, collect some dough, that's life, yo!
                }
            }
            if piece.tp == PieceType::Castle {
                client_kill.send(ClientKilledEvent { client : piece.owner });
            }
            commands.entity(evt.piece).despawn();
            if let Err(_) = broadcast.send(ServerMessage::DeleteObject { id : evt.piece.into() }) {
                println!("game engine lost connection to webserver. this is probably not critical.");
            }
        }
    }
}


fn seed_mature(mut commands : Commands, mut seeds : Query<(Entity, &Transform, &mut Seed)>, place : EventWriter<PlaceEvent>) {
    let mut place = Placer(place);
    for (entity, transform, mut seed) in seeds.iter_mut() {
        if seed.growing {
            seed.time_to_grow -= 1;
        }
        if seed.time_to_grow == 0 {
            commands.entity(entity).despawn();
            place.chest_free(transform.translation.x, transform.translation.y);
        }
    }
}


fn handle_collisions(mut collision_events: EventReader<CollisionEvent>,
        mut pieces : Query<(Entity, &mut GamePiece, Option<&Bullet>, Option<&mut Seed>)>,
        mut spaceshipoids : Query<&mut Spaceshipoid>,
        farms : Query<&Farmhouse>,
        explode_on_collision : Query<(Entity, &CollisionExplosion, &Transform)>,
        explosions : Query<&ExplosionProperties>,
        velocities : Query<&Velocity, Without<StaticWall>>, // we use this to calculate the relative VELOCITY (NOT collision energy - it's physically inaccurate, but idc) and then the damage they incur.
        // this means at low speeds you can safely puuuuush, but at high speeds you get destroyed
        mut piece_destroy : EventWriter<PieceDestroyedEvent>,
        mut explosion_event : EventWriter<ExplosionEvent>,
        sensors : Query<&FieldSensor>,
        clients : Res<ClientMap>) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(one, two, _) = event {
            if let Ok((entity, explode, pos)) = explode_on_collision.get(*one) {
                piece_destroy.send(PieceDestroyedEvent { piece : entity, responsible : PlayerId::SYSTEM });
                explosion_event.send(ExplosionEvent {
                    x : pos.translation.x,
                    y : pos.translation.y,
                    props : explode.explosion
                });
            }
            if let Ok((entity, explode, pos)) = explode_on_collision.get(*two) {
                piece_destroy.send(PieceDestroyedEvent { piece : entity, responsible : PlayerId::SYSTEM });
                explosion_event.send(ExplosionEvent {
                    x : pos.translation.x,
                    y : pos.translation.y,
                    props : explode.explosion
                });
            }
            let mut one_dmg : f32 = 0.0; // damage to apply to entity 1
            let mut two_dmg : f32 = 0.0; // damage to apply to entity 2
            let mut one_killer = PlayerId::SYSTEM; // the id of the player that owned the piece that damaged the piece
            let mut two_killer = PlayerId::SYSTEM; // that is one HELL of a sentence
            // [tyler, several months later] that it is, laddie. that it is.
            // todo: defense and damage modifiers
            if let Ok(v1) = velocities.get(*one) {
                if let Ok(v2) = velocities.get(*two) {
                    let r_vel = (v1.linvel - v2.linvel).length();
                    if r_vel >= 400.0 { // anything slower is nondestructive.
                        // because bullets are usually moving at 400.0 and change, bullets will usually do a little over 1 damage. missiles will do quite a bit more.
                        let d = (r_vel - 350.0).sqrt() / 10.0;
                        one_dmg = d;
                        two_dmg = d;
                        if let Ok((_, piece_one, _, _)) = pieces.get(*one) {
                            two_killer = piece_one.owner;
                        }
                        if let Ok((_, piece_two, _, _)) = pieces.get(*two) {
                            one_killer = piece_two.owner;
                        }
                    }
                }
            }
            if let Ok(explosion) = explosions.get(*one) {
                if let Ok((_, _, _, _)) = pieces.get(*two) {
                    two_dmg += explosion.damage;
                }
            }
            if let Ok(explosion) = explosions.get(*two) {
                if let Ok((_, _, _, _)) = pieces.get(*one) {
                    one_dmg += explosion.damage;
                }
            }
            if one_dmg != 0.0 {
                if let Ok((entity_one, mut piece_one, _, _)) = pieces.get_mut(*one) {
                    piece_one.health -= one_dmg;
                    if let Some(client) = clients.get(&piece_one.owner) {
                        client.send(ServerMessage::Health { id : entity_one.into(), health : piece_one.health / piece_one.start_health });
                    }
                    if piece_one.health <= 0.0 {
                        piece_destroy.send(PieceDestroyedEvent { piece : entity_one, responsible : one_killer });
                    }
                }
            }
            if two_dmg != 0.0 {
                if let Ok((entity_two, mut piece_two, _, _)) = pieces.get_mut(*two) {
                    piece_two.health -= two_dmg;
                    if let Some(client) = clients.get(&piece_two.owner) {
                        client.send(ServerMessage::Health { id : entity_two.into(), health : piece_two.health / piece_two.start_health });
                    }
                    if piece_two.health <= 0.0 {
                        piece_destroy.send(PieceDestroyedEvent { piece : entity_two, responsible : two_killer });
                    }
                }
            }
        }
        let (one, two) = match event { CollisionEvent::Started(one, two, _) => (one, two), CollisionEvent::Stopped(one, two, _) => (one, two) };
        let mut sensor = sensors.get(*one);
        let mut sensor_is_one = true;
        if let Err(_) = sensor {
            sensor_is_one = false;
            sensor = sensors.get(*two);
        }
        if let Ok(sensor) = sensor {
            let mut sensor_owner = PlayerId::SYSTEM;
            let mut sensor_slot : u8 = 0;
            if let Ok((_, sensored_piece, _, _)) = pieces.get(sensor.attached_to) {
                sensor_owner = sensored_piece.owner;
                sensor_slot = sensored_piece.slot;
            }
            let piece = if sensor_is_one {
                pieces.get_mut(*two)
            } else {
                pieces.get_mut(*one)
            };
            if let Ok((entity, gamepiece, _, seed)) = piece {
                if let Some(mut seed) = seed {
                    if let Ok(_) = farms.get(sensor.attached_to) {
                        if let CollisionEvent::Started(_, _, _) = event {
                            seed.growing = true;
                        }
                        else {
                            seed.growing = false;
                        }
                    }
                }
                if gamepiece.owner != sensor_owner && (gamepiece.slot != sensor_slot || gamepiece.slot == 1) { // check if the piece is enemy or not
                    if sensor.attached_to != entity { // missiles can't attempt to attack themselves
                        if let Ok(mut shipoid) = spaceshipoids.get_mut(sensor.attached_to) {       
                            shipoid.sensor_tripped(entity.into());
                        }
                    }
                }
            }
        }
    }
}


struct Placer<'a> (EventWriter<'a, PlaceEvent>);

impl Placer<'_> {
    fn p_simple(&mut self, x : f32, y : f32, client : PlayerId, slot : u8, tp : PieceType) {
        self.0.send(PlaceEvent {
            x,
            y,
            a : 0.0,
            owner : client,
            slot,
            tp,
            free : true
        });
    }

    fn basic_fighter_free(&mut self, x : f32, y : f32, a : f32, client : PlayerId, slot : u8) {
        self.0.send(PlaceEvent {
            x, y, a,
            owner : client,
            slot,
            tp : PieceType::BasicFighter,
            free : true
        });
    }

    fn chest_free(&mut self, x : f32, y : f32) {
        self.0.send(PlaceEvent {
            x, y, a : 0.0,
            owner : PlayerId::SYSTEM,
            slot : 0,
            tp : PieceType::Chest,
            free : true
        });
    }

    fn castle(&mut self, x : f32, y : f32, client : PlayerId, slot : u8) {
        self.0.send(PlaceEvent {
            x, y, a : 0.0,
            owner : client,
            slot,
            tp : PieceType::Castle,
            free : true
        });
    }
}


fn boom(mut commands : Commands, mut explosions : EventReader<ExplosionEvent>, sender : ResMut<Sender>) { // manage explosions
    // explosions are really just sensored colliders with an explosionproperties
    for explosion in explosions.read() {
        let _ = sender.send(ServerMessage::Explosion { x : explosion.x, y : explosion.y, radius : explosion.props.radius, damage : explosion.props.damage });
        commands.spawn((RigidBody::Dynamic, explosion.props, Collider::cuboid(explosion.props.radius, explosion.props.radius), TransformBundle::from(Transform::from_xyz(explosion.x, explosion.y, 0.0)), ActiveEvents::COLLISION_EVENTS));
    }
}


fn explosion_clear(mut commands : Commands, explosions : Query<(Entity, &ExplosionProperties)>) { // must come BEFORE boom (so it's always on the tick afterwards)
    for (entity, _) in explosions.iter() {
        commands.entity(entity).despawn();
    }
}

/*
fn move_missiles(mut missiles : Query<(Entity, &mut ExternalImpulse, &Velocity, &Transform, &mut Missile, &mut PathFollower, &Collider, &GamePiece)>, targetables : Query<&Transform>, targets : Query<&Transform>, mut clients : ResMut<ClientMap>) {
    for (entity, mut impulse, velocity, transform, mut missile, mut follower, collider, piece) in missiles.iter_mut() {
        if let Some(next) = follower.get_next() {
            let mut gpos = match next {
                PathNode::StraightTo(x, y) => Vec2 { x, y },
                PathNode::Target(ent) => {
                    if let Ok(tg) = targetables.get(ent) {
                        tg.translation.truncate()
                    }
                    else {
                        follower.bump();
                        continue;
                    }
                }
            };
            if let Some(lock) = missile.target_lock {
                if let Ok(target) = targets.get(lock) {
                    gpos = target.translation.truncate();
                }
                else {
                    println!("target eliminated");
                    missile.target_lock = None;
                }
            }
            let cpos = transform.translation.truncate();
            let inv_mass = collider.raw.mass_properties(1.0).inv_mass;
            let cangle = transform.rotation.to_euler(EulerRot::ZYX).0;
            let delta = (gpos - cpos).length();
            if delta > 30.0 {
                if delta < missile.intercept_burn {
                    impulse.impulse = Vec2::from_angle(cangle) / inv_mass * missile.intercept_burn_power;
                    impulse.impulse -= velocity.linvel.project_onto((gpos - cpos).perp()) / inv_mass * 0.2; // linear deviation correction thrusters
                    impulse.torque_impulse = -loopify((gpos - cpos).to_angle(), velocity.linvel.to_angle()) * 200.0 / inv_mass - velocity.angvel / inv_mass * 200.0;
                }
                else {
                    impulse.impulse = Vec2::from_angle(cangle) / inv_mass * missile.acc_profile - velocity.linvel / inv_mass * missile.decelerator;
                    impulse.impulse -= velocity.linvel.project_onto((gpos - cpos).perp()) / inv_mass * 0.05; // linear deviation correction thrusters
                    impulse.torque_impulse = -loopify((gpos - cpos).to_angle(), velocity.linvel.to_angle()) * 50.0 / inv_mass - velocity.angvel / inv_mass * 30.0;
                }
                //force.force = Vec2::from_angle(cangle) * (linear_maneuvre(cpos, gpos, velocity.linvel, ship.speed * 50.0, 250.0) / inv_mass);
                // this can produce odd effects at close approach, hence the normalizer code
                //println!("cangle: {}, gangle: {}, angvel: {}", cangle, (cpos - gpos).to_angle(), velocity.angvel);
            }
            else {
                if follower.bump() {
                    if let Some(client) = clients.get_mut(&piece.owner) {
                        client.send(ServerMessage::StrategyCompletion { id : entity.index(), remaining : follower.len() });
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
}*/


fn send_objects(mut events : EventReader<NewClientEvent>, mut clients : ResMut<ClientMap>, objects : Query<(Entity, &GamePiece, &Transform, Option<&Territory>, Option<&Fabber>)>) {
    for ev in events.read() {
        if let Some(client) = clients.get_mut(&ev.id) {
            for (entity, piece, transform, territory, fabber) in objects.iter() {
                client.send(ServerMessage::ObjectCreate {
                    x : transform.translation.x,
                    y : transform.translation.y,
                    a : transform.rotation.to_euler(EulerRot::ZYX).0,
                    owner : piece.owner,
                    id : entity.into(),
                    tp : piece.tp
                });
                if let Some(territory) = territory {
                    client.send(ServerMessage::Territory { id : entity.into(), radius : territory.radius });
                }
                if let Some(fabber) = fabber {
                    client.send(ServerMessage::Fabber { id : entity.into(), radius : fabber.radius });
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
            let _ = broadcast.send(ServerMessage::ObjectMove { // ignore the errors
                id : entity.into(),
                x : pos.x,
                y : pos.y,
                a : transform.rotation.to_euler(EulerRot::ZYX).0
            });
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
    let _ = broadcast.send(ServerMessage::GameState { stage : state.get_state_enum(), tick_in_stage : state.tick, stage_duration : state.time_in_stage });
}


fn update_field_sensors(mut sensors : Query<(&FieldSensor, &mut Transform)>, pieces : Query<&Transform, Without<FieldSensor>>) {
    for (sensor, mut pos) in sensors.iter_mut() {
        if let Ok(piece_pos) = pieces.get(sensor.attached_to) {
            pos.translation = piece_pos.translation;
        } 
    }
}


fn setup(mut commands : Commands, mut state : ResMut<GameState>, config : Res<GameConfig>) {
    // todo: construct board (walls, starting rubble, etc)
    state.tick = 0;
    state.time_in_stage = config.wait_period;
    let system = commands.register_one_shot_system(setup_board);
    commands.spawn(BoardSetup(system));
}


struct EmptyWorld;

impl bevy::ecs::world::Command for EmptyWorld {
    fn apply(self, world : &mut World) {
        world.clear_entities(); // todo: don't clear (or do respawn) things that should stick around, like walls
    }
}


fn client_health_check(mut commands : Commands, mut events : EventReader<ClientKilledEvent>, mut clients : ResMut<ClientMap>, pieces : Query<(Option<&Territory>, &GamePiece, Entity)>, mut state : ResMut<GameState>, config : Res<GameConfig>) {
    // checks:
    // * if the client is still present (if the client disconnected, it's dead by default!), exit early
    // * if the client has any remaining Territory, it's not dead, false alarm
    // if we determined that the client is in fact dead, send a Lose message and update the state accordingly.
    // At the end, if there is 1 or 0 players left, send a Win broadcast as appropriate and reset the state for the next game.
    let mut did_something = false;
    for ev in events.read() {
        if clients.contains_key(&ev.client) { // if the client's already disconnected, we can't exactly tell them they lost
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
        if state.playing && state.currently_playing < 2 {
            if state.currently_playing == 1 {
                let mut winid = PlayerId::SYSTEM;
                for (id, client) in clients.iter() {
                    if client.alive {
                        winid = *id;
                        break;
                    }
                }
                for (_, client) in clients.iter() {
                    client.send(ServerMessage::Winner { id : winid });
                    client.send(ServerMessage::Disconnect);
                }
            }
            state.playing = false;
            state.strategy = false;
            state.tick = 0;
            state.time_in_stage = config.wait_period;
            state.currently_playing = 0;
            commands.add(EmptyWorld{});
        }
        if state.currently_playing < config.min_player_slots {
            state.playing = false;
            state.tick = 0;
            state.time_in_stage = config.wait_period;
            state.strategy = false;
        }
    }
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PhysicsSchedule;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlaySchedule;


fn run_play_schedule(world : &mut World) {
    let state = world.get_resource::<GameState>().expect("gamestate resource not loaded!");
    if state.playing && !state.strategy {
        world.run_schedule(PhysicsSchedule);
        world.run_schedule(PlaySchedule);
    }
}


fn setup_board(mut commands : Commands, config : Res<GameConfig>) { // set up the gameboard
    // this runs after every board clear
    commands.spawn((RigidBody::Fixed, StaticWall{}, TransformBundle::from(Transform::from_xyz(config.width / 2.0, -100.0, 0.0)), Collider::cuboid(config.width / 2.0, 100.0)));
    commands.spawn((RigidBody::Fixed, StaticWall{}, TransformBundle::from(Transform::from_xyz(config.width / 2.0, config.height + 100.0, 0.0)), Collider::cuboid(config.width / 2.0, 100.0)));
    commands.spawn((RigidBody::Fixed, StaticWall{}, TransformBundle::from(Transform::from_xyz(-100.0, config.height / 2.0, 0.0)), Collider::cuboid(100.0, config.height / 2.0)));
    commands.spawn((RigidBody::Fixed, StaticWall{}, TransformBundle::from(Transform::from_xyz(config.width + 100.0, config.height / 2.0, 0.0)), Collider::cuboid(100.0, config.height / 2.0)));
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
                let my_id = PlayerId(*topid);
                *topid += 1;
                drop(topid);
                let (mut client_tx, mut client_rx) = client.split();
                let (from_bevy_tx, mut from_bevy_rx) = tokio::sync::mpsc::channel(128);
                let mut me_verified = false;
                let mut cl = Some(Client {
                    has_placed_castle : false,
                    id : my_id,
                    nickname : "None".to_string(),
                    slot : 0,
                    channel : std::sync::Mutex::new(from_bevy_tx),
                    alive : false,
                    money : 0,
                    connected : false
                });
                if let Err(_) = client_tx.send(warp::ws::Message::binary(ServerMessage::Test("EXOSPHERE".to_string(), 128, 4096, 115600, 123456789012345, -64, -4096, -115600, -123456789012345, -4096.512, -8192.756, VERSION).encode().unwrap())).await {
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
                                            if let Ok(frame) = ClientMessage::decode(&msg.as_bytes()) {
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
                                    if let Err(_) = client_tx.send(warp::ws::Message::binary(msg.encode().unwrap())).await {
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
                                    if let Err(_) = client_tx.send(warp::ws::Message::binary(msg.encode().unwrap())).await {
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
        .add_systems(
            PlaySchedule,
            (
                move_spaceshipoids, shoot, ttl, seed_mature, handle_collisions
            )
        )
        .init_schedule(PlaySchedule)
        .init_schedule(PhysicsSchedule)
        .edit_schedule(PhysicsSchedule, |schedule| {
            schedule.configure_sets((
                PhysicsSet::SyncBackend,
                PhysicsSet::StepSimulation,
                PhysicsSet::Writeback
            ).chain());
        })
        .add_event::<NewClientEvent>()
        .add_event::<ClientKilledEvent>()
        .add_event::<PlaceEvent>()
        .add_event::<ExplosionEvent>()
        .add_event::<PieceDestroyedEvent>()
        .insert_resource(config)
        .insert_resource(ClientMap(HashMap::new()))
        .add_plugins(bevy_time::TimePlugin)
        .insert_resource(Receiver(to_bevy_rx))
        .insert_resource(Sender(from_bevy_broadcast_tx))
        .insert_resource(GameConfig {
            width: 5000.0,
            height: 5000.0,
            wait_period: 10 * UPDATE_RATE as u16, // todo: config files
            play_period: 10 * UPDATE_RATE as u16, // probably gonna be json because I have no balls
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
            make_thing, boom, explosion_clear.before(boom).after(handle_collisions),
            on_piece_dead.after(handle_collisions).after(ttl).after(seed_mature),
            update_field_sensors,
            client_health_check
        )) // health checking should be BEFORE handle_collisions so there's a frame gap in which the entities are actually despawned
        .add_systems(Startup, (setup, setup_board))
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
