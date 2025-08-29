// handle presolve velocity

use crate::components::*;
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn handle_presolve(mut velocities: Query<(&LinearVelocity, &mut PresolveVelocity)>) {
    for (velocity, mut presolve) in velocities.iter_mut() {
        presolve.0 = **velocity;
    }
}
