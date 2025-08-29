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

pub fn strategy_path_handler(
    mut events: EventReader<StrategyPathModifiedEvent>,
    state: Res<GameState>,
    client_meta: Query<&ClientMeta>,
    mut pieces: Query<(&GamePiece, &mut Spaceshipoid)>,
) {
    if state.playing && state.strategy {
        for StrategyPathModifiedEvent(client, event) in events.read() {
            let id = client_meta.get(*client).unwrap().id;
            match event {
                StrategyPathModification::Insert(piece, index, node) => {
                    if let Ok((piece, mut shipoid)) = pieces.get_mut((*piece).into()) {
                        if piece.owner == id {
                            shipoid.pathfollower.insert_node(*index, *node);
                        } else {
                            println!("client attempted to move thing it doesn't own [how rude]");
                        }
                    }
                }
                StrategyPathModification::Clear(piece) => {
                    if let Ok((piece, mut shipoid)) = pieces.get_mut((*piece).into()) {
                        if piece.owner == id {
                            shipoid.pathfollower.clear();
                        }
                    }
                }
                StrategyPathModification::Set(piece, index, node) => {
                    if let Ok((piece, mut shipoid)) = pieces.get_mut((*piece).into()) {
                        if piece.owner == id {
                            shipoid.pathfollower.update_node(*index, *node);
                        }
                    }
                }
                StrategyPathModification::Delete(piece, index) => {
                    if let Ok((piece, mut shipoid)) = pieces.get_mut((*piece).into()) {
                        if piece.owner == id {
                            shipoid.pathfollower.remove_node(*index);
                        }
                    }
                }
            }
        }
    }
}
