// event structures. NOT event handlers.

use bevy::prelude::Event;
use crate::PlaceType;
use crate::ExplosionProperties;

#[derive(Event)]
pub struct NewClientEvent {
    pub id : u64
}


#[derive(Event)]
pub struct PlaceEvent {
    pub x : f32,
    pub y : f32,
    pub a : f32,
    pub owner : u64,
    pub slot : u8,
    pub tp : PlaceType,
    pub free : bool // do we need to fabber check this one? if free is set to true, fabber and territory checks are skipped
}


#[derive(Event)]
pub struct ClientKilledEvent { // something happened that could have killed a client
    // we'll healthcheck to see if the client actually died and update game state accordingly
    pub client : u64
}


#[derive(Event)]
pub struct ExplosionEvent { // an explosion was initiated!
    pub x : f32,
    pub y : f32,
    pub props : ExplosionProperties
}