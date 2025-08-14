// handle pieces being hurt

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use bevy::prelude::*;
use common::comms::ServerMessage;

pub fn piece_harm(
    mut hurt: EventReader<PieceHarmEvent>,
    mut destroy: EventWriter<PieceDestroyedEvent>,
    mut pieces: Query<&mut GamePiece>,
    clients: Res<ClientMap>,
) {
    for event in hurt.read() {
        if let Ok(mut piece) = pieces.get_mut(event.piece) {
            piece.health -= event.harm_amount;
            if let Some(client) = clients.get(&piece.owner) {
                client.send(ServerMessage::Health {
                    id: event.piece.into(),
                    health: piece.health / piece.start_health,
                });
            }
            if piece.health <= 0.0 {
                destroy.write(PieceDestroyedEvent {
                    piece: event.piece,
                    responsible: event.responsible,
                });
            }
        }
    }
}
