/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// sends position updates to clients

use crate::components::*;
use crate::resources::*;
use crate::solve_spaceship::loopify;
use bevy::prelude::*;
use common::comms::ServerMessage;

pub fn position_updates(
    broadcast: ResMut<Sender>,
    mut objects: Query<(Entity, &mut GamePiece, &Transform), Changed<Transform>>,
) {
    for (entity, mut piece, transform) in objects.iter_mut() {
        let pos = transform.translation.truncate();
        let ang = transform.rotation.to_euler(EulerRot::ZYX).0;
        // updates on position
        piece.c_vel = piece.last_update_pos - pos;
        if (pos - piece.last_update_pos).length() > 1.0
            || loopify(ang, piece.last_update_ang).abs() > 0.01
        {
            // are basically straight lines.
            let _ = broadcast.send(ServerMessage::ObjectMove {
                // ignore the errors
                id: entity.into(),
                x: pos.x,
                y: pos.y,
                a: transform.rotation.to_euler(EulerRot::ZYX).0,
            });
            piece.last_update_pos = pos;
            piece.last_update_ang = ang;
        }
    }
}
