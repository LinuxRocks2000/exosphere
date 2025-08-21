/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

use crate::components::*;
use crate::events::*;
use crate::placer::Placer;
use crate::resources::*;
use bevy::prelude::*;
use common::types::*;

pub fn client_place(
    mut events: EventReader<ClientPlaceEvent>,
    mut commands: Commands,
    place: EventWriter<PlaceEvent>,
    mut state: ResMut<GameState>,
    castle_placed: Query<&ClientHasPlacedCastle>,
    config: Res<Config>,
    mut client_collect: EventWriter<ClientCollectEvent>,
    territory: Query<(&Transform, &Territory)>,
    meta: Query<(&Client, &ClientAffiliation)>,
    money: Query<&ClientMoney>,
) {
    let mut place = Placer(place);
    // do a ton of validation on a place event
    // before passing it through to the (unchecked) placer queue. indirection much?
    for ClientPlaceEvent { x, y, tp, client } in events.read() {
        let mut kill = false;
        if let Ok((Client { id }, meta)) = meta.get(*client) {
            if let PieceType::Castle = tp {
                if !state.playing || state.io {
                    if castle_placed.contains(*client) {
                        println!("client attempted to place an extra castle. dropping.");
                        kill = true;
                    } else {
                        let mut is_okay = true;
                        for (transform, territory) in territory.iter() {
                            let dx = transform.translation.x - x;
                            let dy = transform.translation.y - y;
                            let d = (dx * dx + dy * dy).sqrt();
                            if d < territory.radius + 600.0 {
                                // if the territories would intersect
                                is_okay = false;
                                break;
                            }
                        }
                        if is_okay {
                            state.currently_playing += 1;
                            commands.entity(*client).insert(ClientHasPlacedCastle);
                            commands.entity(*client).insert(ClientPlaying);
                            client_collect.write(ClientCollectEvent {
                                client: *client,
                                amount: config.client_setup.money as i32,
                            });
                            let slot = meta.slot;
                            for thing in config.client_setup.area.iter() {
                                thing.place(&mut place, *x, *y, 0.0, *id, slot);
                            }
                        }
                    }
                }
            } else if state.playing && state.strategy {
                let slot = meta.slot;
                if tp.user_placeable() {
                    if money.get(*client).unwrap().money > tp.price() {
                        client_collect.write(ClientCollectEvent {
                            client: *client,
                            amount: -1 * tp.price() as i32,
                        });
                        place.p_simple(*x, *y, *id, slot, *tp);
                    }
                }
            }
        }
        if kill {
            commands.entity(*client).despawn();
            // TODO: remove records from the ClientMap; currently we're leaking memory
        }
    }
}
