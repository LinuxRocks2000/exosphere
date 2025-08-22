/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// all of the ECS systems stick in this module.
pub mod make_thing;
pub use make_thing::*;

pub mod client_tick;
pub use client_tick::*;

pub mod spaceshipoids;
pub use spaceshipoids::*;

pub mod handle_collisions;
pub use handle_collisions::*;

pub mod lasernodes;
pub use lasernodes::*;

pub mod lasers;
pub use lasers::*;

pub mod scrapships;
pub use scrapships::*;

pub mod turret;
pub use turret::*;

pub mod seed_mature;
pub use seed_mature::*;

pub mod shoot;
pub use shoot::*;

pub mod ttl;
pub use ttl::*;

pub mod on_piece_dead;
pub use on_piece_dead::*;

pub mod boom;
pub use boom::*;

pub mod explosion_clear;
pub use explosion_clear::*;

pub mod send_objects;
pub use send_objects::*;

pub mod position_updates;
pub use position_updates::*;

pub mod frame_broadcast;
pub use frame_broadcast::*;

pub mod update_field_sensors;
pub use update_field_sensors::*;

pub mod setup;
pub use setup::*;

pub mod setup_board;
pub use setup_board::*;

pub mod client_health_check;
pub use client_health_check::*;

pub mod handle_presolve;
pub use handle_presolve::*;

pub mod piece_harm;
pub use piece_harm::*;

pub mod setup_client;
pub use setup_client::*;

pub mod client_place;
pub use client_place::*;

pub mod client_connection;
pub use client_connection::*;

pub mod client_disconnection;
pub use client_disconnection::*;

pub mod client_flow_password;
pub use client_flow_password::*;

pub mod strategy_path_handler;
pub use strategy_path_handler::*;

pub mod special_handler;
pub use special_handler::*;

pub mod client_flow_team;
pub use client_flow_team::*;

pub mod client_win_checks;
pub use client_win_checks::*;
