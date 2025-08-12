/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// keeps field sensors attached to their objects

use crate::components::*;
use bevy::prelude::*;

pub fn update_field_sensors(
    mut sensors: Query<(&FieldSensor, &mut Transform)>,
    pieces: Query<&Transform, Without<FieldSensor>>,
) {
    for (sensor, mut pos) in sensors.iter_mut() {
        if let Ok(piece_pos) = pieces.get(sensor.attached_to) {
            pos.translation = piece_pos.translation;
        }
    }
}
