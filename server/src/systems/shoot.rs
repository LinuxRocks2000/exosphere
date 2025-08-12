/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// fires bullets from items with guns

use crate::components::*;
use crate::discharge_barrel;
use crate::resources::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

pub fn shoot(
    mut commands: Commands,
    mut pieces: Query<(&Transform, &Velocity, &GamePiece, &mut Gun)>,
    broadcast: ResMut<Sender>,
) {
    for (position, velocity, piece, mut gun) in pieces.iter_mut() {
        if gun.enabled {
            if gun.tick == 0 {
                gun.r_point += 1;
                if gun.r_point >= gun.repeats {
                    gun.tick = gun.cd;
                    gun.r_point = 0;
                } else {
                    gun.tick = gun.repeat_cd;
                }
                if gun.scatter_barrels {
                    discharge_barrel(
                        &mut commands,
                        piece.owner,
                        rand::thread_rng().gen_range(0..gun.barrels),
                        &gun,
                        position,
                        velocity,
                        &broadcast,
                    );
                } else {
                    for barrel in 0..gun.barrels {
                        discharge_barrel(
                            &mut commands,
                            piece.owner,
                            barrel,
                            &gun,
                            position,
                            velocity,
                            &broadcast,
                        );
                    }
                }
            }
            gun.tick -= 1;
        }
    }
}
