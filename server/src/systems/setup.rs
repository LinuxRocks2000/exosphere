/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// resets the game

use crate::components::*;
use crate::resources::*;
use crate::systems::*;
use bevy::prelude::*;

pub fn setup(mut commands: Commands, mut state: ResMut<GameState>, config: Res<GameConfig>) {
    // todo: construct board (walls, starting rubble, etc)
    state.tick = 0;
    state.time_in_stage = config.wait_period;
    let system = commands.register_system(setup_board);
    commands.spawn(BoardSetup(system));
}
