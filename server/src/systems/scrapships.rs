/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// scrapship handler system
use bevy::prelude::*;
use crate::components::*;
use crate::events::*;


pub fn scrapships(mut scrapships : Query<(Entity, &mut ScrapShip, &Transform)>, seeds : Query<&Transform>, mut lasers : EventWriter<LaserCastEvent>) {
    for (shipentity, mut ship, shippos) in scrapships.iter_mut() {
        if ship.seeds_in_range.len() > 0 {
            ship.ind += 1;
            if ship.ind >= ship.seeds_in_range.len() {
                ship.ind = 0;
            }
            let seed = ship.seeds_in_range[ship.ind];
            if let Ok(pos) = seeds.get(seed) {
                lasers.send(LaserCastEvent {
                    caster : shipentity,
                    from : shippos.translation.truncate(),
                    dir : (pos.translation - shippos.translation).truncate().normalize(),
                    max_dist : 300.0,
                    dmg : 0.03,
                    exclusive : Some(seed)
                });
            }
            else {
                let ind = ship.ind;
                ship.seeds_in_range.swap_remove(ind);
            }
        }
    }
}
