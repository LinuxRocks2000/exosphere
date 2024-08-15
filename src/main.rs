/*
    Exosphere is the third member of the Strategy Game franchise. It is a direct continuation of MMOSG v2, but with more realistic physics and smoother gameplay.
    Unlike MMOSG v2, which is written with a custom websocket server and game engine, Exosphere uses Bevy and Warp.
*/

use bevy::prelude::*;
use warp::Filter;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::{Mutex, mpsc, broadcast};
use std::sync::Arc;
use std::collections::HashMap;

pub mod protocol;
use protocol::Protocol;
use protocol::ProtocolRoot;
use crate::protocol::DecodeError;


const UPDATE_RATE : u64 = 30; // 30hz by default
const FRAME_TIME : std::time::Duration = std::time::Duration::from_millis(1000 / UPDATE_RATE); // milliseconds per frame

const MAX_FRAME_SIZE : usize = 1024; // maximum size of an incoming websocket frame

#[derive(Debug, ProtocolRoot)]
enum ClientMessage {
    Test(String, u16, f32)
}


#[derive(Debug, ProtocolRoot, Clone)]
enum ServerMessage {
    Test(String, u16, f32)
}


struct Client {
    id : u64,
    channel : tokio::sync::mpsc::Sender<ServerMessage>
}


enum Comms {
    ClientConnect(Client),
    ClientDisconnect(u64),
    MessageFrom(u64, ClientMessage)
}


fn tick(mut clients : ResMut<ClientMap>, mut receiver : ResMut<Receiver>, broadcast : ResMut<Sender>) {
    loop { // loops receiver.try_recv(), until it returns empty
        match receiver.try_recv() {
            Ok(message) => {
                match message {
                    Comms::ClientConnect(cli) => {
                        broadcast.send(ServerMessage::Test("hi everybody! we have a new client!".to_string(), 0, 1.0)).expect("critical channel failure: broken connection to webserver");
                        cli.channel.try_send(ServerMessage::Test("hello, client".to_string(), 1024, 420.69)).unwrap();
                        clients.insert(cli.id, cli);
                    },
                    Comms::ClientDisconnect(id) => {
                        clients.remove(&id);
                    },
                    Comms::MessageFrom(id, msg) => {
                        println!("client {} sent message {:?}", id, msg);
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
    // do other stuff down here
}

#[derive(Resource, Deref, DerefMut)]
struct ClientMap(HashMap<u64, Client>);

#[derive(Resource, Deref, DerefMut)] // todo: better names (or generic type arguments)
struct Receiver(mpsc::Receiver<Comms>);

#[derive(Resource, Deref, DerefMut)]
struct Sender(broadcast::Sender<ServerMessage>);



#[tokio::main]
async fn main() {
    let top_id = Arc::new(Mutex::new(0_u64)); // POSSIBLE BUG: if the client id goes beyond 18,446,744,073,709,551,615, it may overflow and assign duplicate IDs
    // this is not likely to be a real problem
    let (to_bevy_tx, to_bevy_rx) = mpsc::channel::<Comms>(32);
    let (from_bevy_broadcast_tx, _) = broadcast::channel::<ServerMessage>(32);
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
                let cl = Client {
                    id : my_id,
                    channel : from_bevy_tx
                };
                if let Err(_) = to_bevy.send(Comms::ClientConnect(cl)).await {
                    println!("channel failure 1: lost connection to game engine");
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
                                                if let Err(_) = to_bevy.send(Comms::MessageFrom(my_id, frame)).await {
                                                    println!("channel failure 1.125: lost connection to game engine");
                                                    break 'cli_loop;
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
                                    if let Err(_) = to_bevy.send(Comms::ClientDisconnect(my_id)).await {
                                        println!("channel failure 3: lost connection to game engine");
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
    App::new()
        .insert_resource(ClientMap(HashMap::new()))
        .insert_resource(Receiver(to_bevy_rx))
        .insert_resource(Sender(from_bevy_broadcast_tx))
        .add_systems(Update, tick)
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
