/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// consumer for LaserCastEvent


use bevy::prelude::*;
use crate::events::*;
use common::comms::*;
use crate::resources::*;
use bevy_rapier2d::prelude::*;
use crate::components::*;


pub fn lasers(mut events : EventReader<LaserCastEvent>, mut pieces : Query<&mut GamePiece>, broadcast : ResMut<Sender>, rapier_context: Res<RapierContext>, clients : Res<ClientMap>, mut piece_destroy : EventWriter<PieceDestroyedEvent>) {
    for cast in events.read() {
        let cast_owner = if let Ok(piece) = pieces.get(cast.caster) {
            Some(piece.owner)
        } else {
            None
        };
        if let Some(cast_owner) = cast_owner {
            let mut to = cast.from + cast.dir * cast.max_dist;
            if let Some((entity, toi)) = rapier_context.cast_ray(cast.from, cast.dir, cast.max_dist, true, QueryFilter::default().exclude_sensors().exclude_collider(cast.caster)) {
                if let Some(e) = cast.exclusive {
                    if entity != e {
                        continue;
                    }
                }
                if let Ok(mut piece) = pieces.get_mut(entity) {
                    piece.health -= cast.dmg;
                    if let Some(client) = clients.get(&piece.owner) {
                        client.send(ServerMessage::Health { id : entity.into(), health : piece.health / piece.start_health });
                    }
                    if piece.health <= 0.0 {
                        piece_destroy.send(PieceDestroyedEvent { piece : entity, responsible : cast_owner });
                    }
                }
                to = cast.from + cast.dir * toi;
            }
            else if let Some(_) = cast.exclusive {
                continue;
            }
            let _ = broadcast.send(ServerMessage::LaserCast {
                caster : cast.caster.into(),
                from_x : cast.from.x,
                from_y : cast.from.y,
                to_x : to.x,
                to_y : to.y
            });
        }
    }
}