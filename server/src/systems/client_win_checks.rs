// checks if there's a winner
use crate::components::*;
use crate::events::*;
use crate::resources::*;
use bevy::prelude::*;
use common::comms::ServerMessage;
use common::PlayerId;

pub fn client_win_checks(
    mut state: ResMut<GameState>,
    mut events: EventReader<ClientLostEvent>,
    players: Query<&ClientPlaying>,
    clients: Res<ClientMap>,
    broadcast: Res<Sender>,
    config: Res<Config>,
) {
    if events.read().len() > 0 {
        if !state.io {
            let currently_playing = players.iter().len();
            if state.playing && currently_playing < 2 {
                if currently_playing == 1 {
                    let mut winid = PlayerId::SYSTEM;
                    for (id, client) in clients.iter() {
                        if players.contains(*client) {
                            winid = *id;
                            break;
                        }
                    }
                    broadcast.send(ServerMessage::Winner { id: winid }).unwrap();
                    broadcast.send(ServerMessage::Disconnect).unwrap();
                }
                state.playing = false;
                state.strategy = false;
                state.tick = 0;
                state.time_in_stage = config.times.wait_period;
            }
            if currently_playing < config.counts.min_players as usize {
                state.playing = false;
                state.tick = 0;
                state.time_in_stage = config.times.wait_period;
                state.strategy = false;
            }
        }
    }
}
