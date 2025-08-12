/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// consumer for LaserCastEvent

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use avian2d::prelude::*;
use bevy::prelude::*;
use common::comms::*;

pub fn lasers(
    mut events: EventReader<LaserCastEvent>,
    mut pieces: Query<&mut GamePiece>,
    broadcast: ResMut<Sender>,
    space_query: SpatialQuery,
    clients: Res<ClientMap>,
    mut piece_destroy: EventWriter<PieceDestroyedEvent>,
) {
    for cast in events.read() {
        let cast_owner = if let Ok(piece) = pieces.get(cast.caster) {
            piece.owner
        } else {
            continue;
        };
        let filter = SpatialQueryFilter::default()
            .with_excluded_entities([cast.caster])
            .with_mask(LayerMask::DEFAULT);
        let dir = if let Ok(dir) = Dir2::new(cast.dir) {
            dir
        } else {
            println!("trying to shoot myself");
            continue;
        };
        let hit =
            if let Some(hit) = space_query.cast_ray(cast.from, dir, cast.max_dist, true, &filter) {
                hit
            } else {
                let to = cast.from + cast.dir * cast.max_dist;
                let _ = broadcast.send(ServerMessage::LaserCast {
                    // TODO: send laser cast broadcasts even if the laser didn't hit anything
                    caster: cast.caster.into(),
                    from_x: cast.from.x,
                    from_y: cast.from.y,
                    to_x: to.x,
                    to_y: to.y,
                });
                continue;
            };
        if let Ok(mut piece) = pieces.get_mut(hit.entity) {
            println!("lasering a piece");
            piece.health -= cast.dmg;
            if let Some(client) = clients.get(&piece.owner) {
                client.send(ServerMessage::Health {
                    id: hit.entity.into(),
                    health: piece.health / piece.start_health,
                });
            }
            if piece.health <= 0.0 {
                piece_destroy.write(PieceDestroyedEvent {
                    piece: hit.entity,
                    responsible: cast_owner,
                });
            }
        }
        let to = cast.from + cast.dir * hit.distance;
        let _ = broadcast.send(ServerMessage::LaserCast {
            // TODO: send laser cast broadcasts even if the laser didn't hit anything
            caster: cast.caster.into(),
            from_x: cast.from.x,
            from_y: cast.from.y,
            to_x: to.x,
            to_y: to.y,
        });
    }
}
