/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// sets up the board after board clearing

use crate::components::*;
use crate::events::*;
use crate::placer::Placer;
use crate::resources::*;
use avian2d::prelude::*;
use bevy::prelude::*;
use common::PlayerId;

pub fn setup_board(mut commands: Commands, config: Res<Config>, place: EventWriter<PlaceEvent>) {
    // set up the gameboard
    // this runs after every board clear
    commands.spawn((
        // top
        RigidBody::Static,
        StaticWall {},
        Transform::from_xyz(config.board.width / 2.0, -100.0, 0.0),
        Collider::rectangle(config.board.width, 200.0),
    ));
    commands.spawn((
        // bottom
        RigidBody::Static,
        StaticWall {},
        Transform::from_xyz(config.board.width / 2.0, config.board.height + 100.0, 0.0),
        Collider::rectangle(config.board.width, 200.0),
    ));
    commands.spawn((
        // left
        RigidBody::Static,
        StaticWall {},
        Transform::from_xyz(-100.0, config.board.height / 2.0, 0.0),
        Collider::rectangle(200.0, config.board.height),
    ));
    commands.spawn((
        // right
        RigidBody::Static,
        StaticWall {},
        Transform::from_xyz(config.board.width + 100.0, config.board.height / 2.0, 0.0),
        Collider::rectangle(200.0, config.board.height),
    ));
    let mut placer = Placer(place);
    for thing in &config.board.things {
        thing.place(&mut placer, 0.0, 0.0, 0.0, PlayerId::SYSTEM, 0);
    }
}
