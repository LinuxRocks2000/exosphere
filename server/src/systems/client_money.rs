// manage events about money

use crate::components::*;
use crate::events::*;
use bevy::prelude::*;

pub fn client_money(
    mut events: EventReader<ClientCollectEvent>,
    mut clients: Query<(&mut ClientMoney, &ClientChannel, &Client)>,
) {
    for event in events.read() {
        if let Ok((mut money, channel, client)) = clients.get_mut(event.client) {
            money.money = (money.money as i32 + event.amount) as u32;
            channel.send(common::comms::ServerMessage::Money {
                id: client.id,
                amount: money.money,
            });
        }
    }
}
