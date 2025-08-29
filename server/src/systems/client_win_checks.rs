// checks if there's a winning client or team
use crate::components::*;
use crate::events::*;
use crate::resources::*;
use bevy::prelude::*;
use common::comms::ServerMessage;
use common::PlayerId;

pub fn client_win_checks(
    mut state: ResMut<GameState>,
    mut events: EventReader<ClientLostEvent>,
    players: Query<&ClientAffiliation, With<ClientPlaying>>,
    clients: Res<ClientMap>,
    broadcast: Res<Sender>,
    config: Res<Config>,
) {
    if events.read().len() == 0 {
        return;
    }
    if !state.io {
        let mut last_slot = None;
        let mut is_team_variety = false; // if there are ANY free agents, or MORE THAN 1 teams have living members, this should be true.
        let mut currently_playing = 0; // the number of total players playing
        for affiliation in players.iter() {
            currently_playing += 1;
            if let Some(s) = last_slot {
                if s != affiliation.slot {
                    is_team_variety = true;
                }
            }
            if affiliation.slot == 1 {
                is_team_variety = true;
            } else if affiliation.slot > 1 {
                last_slot = Some(affiliation.slot);
            }
        }
        if state.playing && (currently_playing < 2 || !is_team_variety) {
            if currently_playing == 1 {
                let mut winid = PlayerId::SYSTEM;
                for (id, client) in clients.iter() {
                    if players.contains(*client) {
                        winid = *id;
                        break;
                    }
                }
                broadcast.send(ServerMessage::Winner { id: winid }).unwrap();
            } else if let Some(slot) = last_slot {
                broadcast.send(ServerMessage::TeamWin { id: slot }).unwrap();
            }
            broadcast.send(ServerMessage::Disconnect).unwrap();
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
