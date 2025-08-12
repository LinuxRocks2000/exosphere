use crate::events::*;
use crate::PieceType;
use bevy::prelude::*;
use common::PlayerId;

pub(crate) struct Placer<'a>(pub EventWriter<'a, PlaceEvent>);

impl Placer<'_> {
    pub(crate) fn p_simple(&mut self, x: f32, y: f32, client: PlayerId, slot: u8, tp: PieceType) {
        self.0.send(PlaceEvent {
            x,
            y,
            a: 0.0,
            owner: client,
            slot,
            tp,
            free: false,
        });
    }

    pub(crate) fn basic_fighter_free(
        &mut self,
        x: f32,
        y: f32,
        a: f32,
        client: PlayerId,
        slot: u8,
    ) {
        self.0.send(PlaceEvent {
            x,
            y,
            a,
            owner: client,
            slot,
            tp: PieceType::BasicFighter,
            free: true,
        });
    }

    pub(crate) fn small_lasernode_free(&mut self, x: f32, y: f32, client: PlayerId, slot: u8) {
        self.0.send(PlaceEvent {
            x,
            y,
            a: 0.0,
            owner: client,
            slot,
            tp: PieceType::LaserNode,
            free: true,
        });
    }

    pub(crate) fn chest_free(&mut self, x: f32, y: f32) {
        self.0.send(PlaceEvent {
            x,
            y,
            a: 0.0,
            owner: PlayerId::SYSTEM,
            slot: 0,
            tp: PieceType::Chest,
            free: true,
        });
    }

    pub(crate) fn castle(&mut self, x: f32, y: f32, client: PlayerId, slot: u8) {
        self.0.send(PlaceEvent {
            x,
            y,
            a: 0.0,
            owner: client,
            slot,
            tp: PieceType::Castle,
            free: true,
        });
    }
}
