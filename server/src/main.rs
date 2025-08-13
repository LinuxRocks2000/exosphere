/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// TODO TODO TODO: fix the sensor leak: when a sensored thing dies, its sensor persists. the sensor doesn't do anything because of attachment checks, but it's still *there*
// wasting memory and cpu cycles.

// TODO: fix the ghost bug (under some circumstances, cleanup doesn't seem to correctly delete some pieces, leaving "ghosts" that persist until server reset)

// note:
/*
    user ids are u64s.
    user slots are u8s.
    the id is guaranteed to be unique, but slots often collide. the 0 slot means spectator - clients in slot 0 can't do anything. the 1 slot means free agent;
    they are not allied with anybody. slots 2-255 are the team slots. if two players have the same slot, they're allies. yay!

    user id 0 is the system, which does not ever have to obey territory or fabber boundaries.
*/

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_macros::*;
use std::collections::HashMap;

pub use common::comms;
use common::types::*;
use common::PlayerId;
use common::VERSION;
use comms::{ClientMessage, ServerMessage};

pub enum Comms {
    // webserver -> game engine
    ClientConnect(Client),                // (client) a client connected
    ClientDisconnect(PlayerId),           // (id) a client disconnected
    MessageFrom(PlayerId, ClientMessage), // (id, message) a client sent a message that was successfully decoded and filtered
}

pub mod solve_spaceship;

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

pub mod placer;
use placer::*;

pub mod websocket;

pub struct Client {
    id: PlayerId,
    nickname: String,
    slot: u8,
    channel: crossbeam::channel::Sender<(PlayerId, ServerMessage)>,
    has_placed_castle: bool,
    alive: bool,
    money: u32, // if I make it a u16 richard will crash the server by somehow farming up >66k money
    connected: bool,
}

impl Client {
    fn send(&self, msg: ServerMessage) {
        if let Err(_) = self.channel.try_send((self.id, msg)) {
            println!("failed to send message on channel");
        }
    }

    fn collect(&mut self, amount: u32) {
        self.money += amount;
        self.send(ServerMessage::Money {
            id: self.id,
            amount: self.money,
        });
    }

    fn charge(&mut self, amount: u32) -> bool {
        // returns if we actually successfully made the charge or not
        if self.money >= amount {
            self.money -= amount;
            self.send(ServerMessage::Money {
                id: self.id,
                amount: self.money,
            });
            return true;
        }
        return false;
    }
}

struct EmptyWorld;

impl Command for EmptyWorld {
    fn apply(self, world: &mut World) {
        world.clear_entities(); // todo: don't clear (or do respawn) things that should stick around, like walls
    }
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PhysicsSchedule;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlaySchedule;

fn run_play_schedule(world: &mut World) {
    let (playing, strategy) = {
        let state = world
            .get_resource::<GameState>()
            .expect("gamestate resource not loaded!");
        (state.playing, state.strategy)
    };
    if playing && !strategy {
        world.run_schedule(PlaySchedule);
        world
            .get_resource_mut::<Time<Physics>>()
            .expect("physics time not loaded")
            .unpause();
    } else {
        world
            .get_resource_mut::<Time<Physics>>()
            .expect("physics time not loaded")
            .pause();
    }
}

fn main() {
    let (to_bevy_tx, to_bevy_rx) = crossbeam::channel::unbounded();
    let (from_bevy_broadcast_tx, from_bevy_broadcast_rx) = crossbeam::channel::unbounded();
    let (from_bevy_specific_tx, from_bevy_specific_rx) =
        crossbeam::channel::unbounded::<(PlayerId, ServerMessage)>();

    std::thread::spawn(move || {
        let mut server = websocket::Server::new("0.0.0.0:3000").unwrap();
        struct ClientProperties {
            has_tested: bool, // successful test response received
        }
        use std::collections::HashMap;
        let mut clients: HashMap<websocket::ClientId, ClientProperties> = HashMap::new();
        loop {
            server.do_poll(
                &mut clients,
                |id, m: ClientMessage, server, clients| {
                    let clprops = clients.get_mut(&id).unwrap();
                    if clprops.has_tested {
                        if let Err(_) = to_bevy_tx.send(Comms::MessageFrom(id.into(), m)) {
                            println!("channel failure: this is probably fatal");
                        }
                    } else {
                        if m == ClientMessage::Test(
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
                        ) {
                            clprops.has_tested = true;
                            let client = Client {
                                has_placed_castle: false,
                                money: 0,
                                nickname: "None".to_string(),
                                slot: 0,
                                alive: false,
                                id: id.into(),
                                connected: false,
                                channel: from_bevy_specific_tx.clone(),
                            };
                            if let Err(_) = to_bevy_tx.send(Comms::ClientConnect(client)) {
                                println!("channel failure: this is probably fatal");
                            }
                        } else {
                            server.close(id);
                        }
                    }
                },
                |id, server, clients| {
                    clients.insert(id, ClientProperties { has_tested: false });
                    println!("new client {:?}", id);
                    server.send_to(
                        id,
                        ServerMessage::Test(
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
                        ),
                    )
                },
                |id, clients| {
                    if let Some(client) = clients.get(&id) {
                        if client.has_tested {
                            if let Err(_) = to_bevy_tx.send(Comms::ClientDisconnect(id.into())) {
                                println!("channel failure: this is probably fatal");
                            }
                        }
                    }
                },
            );
            loop {
                match from_bevy_broadcast_rx.try_recv() {
                    Ok(message) => {
                        server.broadcast(message);
                    }
                    Err(crossbeam::channel::TryRecvError::Empty) => {
                        break;
                    }
                    _ => {
                        panic!("channel failure!");
                    }
                }
            }
            loop {
                match from_bevy_specific_rx.try_recv() {
                    Ok((id, message)) => {
                        server.send_to(id.into(), message);
                    }
                    Err(crossbeam::channel::TryRecvError::Empty) => {
                        break;
                    }
                    _ => {
                        panic!("channel failure!");
                    }
                }
            }
        }
    });

    App::new()
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(avian2d::dynamics::solver::SolverDiagnostics::default())
        .insert_resource(avian2d::collision::CollisionDiagnostics::default())
        .insert_resource(avian2d::spatial_query::SpatialQueryDiagnostics::default())
        .add_systems(
            PlaySchedule,
            (
                move_spaceshipoids,
                shoot,
                ttl,
                seed_mature,
                handle_collisions,
                handle_destructive_collisions,
                lasernodes,
                lasers,
                scrapships,
                turrets,
            ),
        )
        .init_schedule(PlaySchedule)
        .add_event::<NewClientEvent>()
        .add_event::<ClientKilledEvent>()
        .add_event::<PlaceEvent>()
        .add_event::<ExplosionEvent>()
        .add_event::<PieceDestroyedEvent>()
        .add_event::<LaserCastEvent>()
        .insert_resource(ClientMap(HashMap::new())) // -225, -39.5, -516.9
        .add_plugins(bevy_time::TimePlugin)
        .insert_resource(Receiver(to_bevy_rx))
        .insert_resource(Gravity(Vec2::new(0.0, 0.0)))
        .insert_resource(Sender(from_bevy_broadcast_tx))
        .insert_resource(GameConfig {
            width: 5000.0,
            height: 5000.0,
            wait_period: 1 * UPDATE_RATE as u16, // todo: config files
            play_period: 10 * UPDATE_RATE as u16,
            strategy_period: 5 * UPDATE_RATE as u16, // [2024-11-21] it's always a "joy" reading comments I wrote months ago.
            max_player_slots: 1000,
            min_player_slots: 1,
        })
        .insert_resource(GameState {
            playing: false,
            io: true,
            strategy: false,
            tick: 0,
            time_in_stage: 0,
            currently_attached_players: 0,
            currently_playing: 0,
        })
        .add_systems(PreUpdate, (run_play_schedule,))
        .add_systems(
            FixedPostUpdate,
            handle_presolve.in_set(avian2d::schedule::PhysicsSet::Prepare),
        )
        .add_systems(
            Update,
            (
                client_tick,
                send_objects,
                position_updates,
                frame_broadcast.before(position_updates),
                make_thing,
                boom,
                explosion_clear.before(boom).after(handle_collisions),
                on_piece_dead
                    .after(handle_collisions)
                    .after(ttl)
                    .after(seed_mature),
                update_field_sensors,
                client_health_check,
            ),
        ) // health checking should be BEFORE handle_collisions so there's a frame gap in which the entities are actually despawned
        .add_systems(Startup, (setup, setup_board))
        .set_runner(|mut app| loop {
            let start = std::time::Instant::now();
            app.update();
            let time_elapsed = start.elapsed();
            if time_elapsed < FRAME_TIME {
                let time_remaining = FRAME_TIME - time_elapsed;
                std::thread::sleep(time_remaining);
            }
        })
        .run();
}
