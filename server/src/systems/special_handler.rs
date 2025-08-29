/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use bevy::prelude::*;
use common::comms::*;

pub fn special_handler(
    mut events: EventReader<ClientSpecialObjectEvent>,
    state: Res<GameState>,
    mut guns_pieces: Query<(&GamePiece, &mut Gun)>,
    client_meta: Query<&ClientMeta>,
) {
    if state.playing && state.strategy {
        for ClientSpecialObjectEvent(client, id, evt) in events.read() {
            match evt {
                ObjectSpecialPropertySet::GunState(state) => {
                    if let Ok((piece, mut gun)) = guns_pieces.get_mut((*id).into()) {
                        if piece.owner == client_meta.get(*client).unwrap().id {
                            gun.enabled = *state;
                        }
                    }
                }
            }
        }
    }
}
