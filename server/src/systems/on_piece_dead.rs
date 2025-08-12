/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// when pieces die, handle the effects!

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use crate::Bullets;
use crate::PieceType;
use bevy::prelude::*;
use common::comms::ServerMessage;

pub fn on_piece_dead(
    mut commands: Commands,
    broadcast: ResMut<Sender>,
    pieces: Query<&GamePiece>,
    bullets: Query<(&Bullet, &Transform)>,
    chests: Query<&Chest>,
    mut events: EventReader<PieceDestroyedEvent>,
    mut explosions: EventWriter<ExplosionEvent>,
    mut client_kill: EventWriter<ClientKilledEvent>,
    mut clients: ResMut<ClientMap>,
) {
    for evt in events.read() {
        if let Ok(piece) = pieces.get(evt.piece) {
            if let Ok((bullet, pos)) = bullets.get(evt.piece) {
                if let Bullets::Bomb(explosion, _) = bullet.tp {
                    explosions.send(ExplosionEvent {
                        x: pos.translation.x,
                        y: pos.translation.y,
                        props: explosion,
                    });
                }
            }
            if let Ok(_) = chests.get(evt.piece) {
                if let Some(cl) = clients.get_mut(&evt.responsible) {
                    cl.collect(20); // kill the chest, collect some dough, that's life, yo!
                }
            }
            if piece.tp == PieceType::Castle {
                client_kill.send(ClientKilledEvent {
                    client: piece.owner,
                });
            }
            commands.entity(evt.piece).despawn();
            if let Err(_) = broadcast.send(ServerMessage::DeleteObject {
                id: evt.piece.into(),
            }) {
                println!(
                    "game engine lost connection to webserver. this is probably not critical."
                );
            }
        }
    }
}
