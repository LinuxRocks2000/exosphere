/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// ALL of the component declarations and their respective impls

pub use crate::client_components::*;

use crate::solve_spaceship::*;
use crate::systems::shoot::Bullets;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::Query;
use bevy::prelude::Vec2;
use common::fab::FabLevels;
use common::pathfollower::PathFollower;
use common::pathfollower::*;
use common::types::*;
use common::{PieceId, PlayerId};
use std::f32::consts::PI;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Component)]
pub(crate) struct GamePiece {
    pub(crate) tp: PieceType, // the type of this piece
    // assigned by the gamepiece builder functions
    pub(crate) owner: PlayerId, // entry in the Clients hashmap
    pub(crate) slot: u8,        // identity slot of the owner
    // in the future, we may want to eliminate this and instead do lookups in the HashMap (which is swisstable, so it's pretty fast)
    // but for now it's convenient
    pub(crate) last_update_pos: Vec2,
    pub(crate) last_update_ang: f32,
    pub(crate) c_vel: Vec2,
    pub(crate) start_health: f32,
    pub(crate) health: f32,
}

impl GamePiece {
    pub(crate) fn new(tp: PieceType, owner: PlayerId, slot: u8, health: f32) -> Self {
        Self {
            tp,
            owner,
            slot,
            last_update_pos: Vec2::ZERO,
            c_vel: Vec2::ZERO,
            last_update_ang: 0.0,
            start_health: health,
            health: health,
        }
    }
}

#[derive(Component)]
pub(crate) struct Territory {
    // a territory control radius produced by a castle or fort.
    pub(crate) radius: f32,
}

impl Territory {
    pub(crate) fn castle() -> Self {
        Self { radius: 600.0 }
    }
}

#[derive(Component)]
pub(crate) struct Fabber {
    // a fabber bay with a radius
    pub(crate) radius: f32,
    pub(crate) levels: FabLevels,
}

impl Fabber {
    pub(crate) fn castle() -> Self {
        Self {
            // Large-M4S2E2D3B2
            radius: 500.0,
            levels: FabLevels {
                missiles: 4,
                ships: 3,
                econ: 2,
                defense: 3,
                buildings: 2,
            },
        }
    }

    pub(crate) fn is_available(&self, tp: PieceType) -> bool {
        // determine if this fabber can produce an object
        self.levels >= tp.fabber()
    }
}

#[derive(Component)]
pub(crate) struct TimeToLive {
    pub(crate) lifetime: u16,
}

#[derive(Component)]
pub(crate) struct Bullet {
    pub(crate) tp: Bullets,
} // bullet collision semantics
  // normal collisions between entities are only destructive if greater than a threshold
  // bullet collisions are always destructive

#[derive(Component)]
pub(crate) struct Seed {
    pub(crate) time_to_grow: u16, // remaining time before this seed sprouts
    pub(crate) growing: bool,     // are we actively growing?
}

impl Seed {
    pub(crate) fn new() -> Self {
        Self {
            time_to_grow: 100,
            growing: false,
        }
    }
}

#[derive(Component)]
pub(crate) struct Chest {}

#[derive(Component)]
pub(crate) struct Gun {
    pub(crate) enabled: bool,
    pub(crate) cd: u16, // cooldown ticks between shots
    pub(crate) bullets: Bullets,
    pub(crate) repeats: u16,   // number of repeater shots
    pub(crate) repeat_cd: u16, // time between repeater shots
    // state fields (don't touch):
    pub(crate) r_point: u16, // current repeater position
    // the repeat pattern is pretty simple. when a bullet is fired, r_point is incremented by one, and if it's less than the number of repeats, `tick` is set to
    // repeat_cd instead of cd. when r_point >= repeats, r_point = 0 and tick = cd.
    pub(crate) tick: u16, // current tick
    pub(crate) barrels: u16,
    pub(crate) barrel_spacing: f32,
    pub(crate) center_offset: f32,
    pub(crate) scatter_barrels: bool, // randomly pick a single barrel to fire from every shot
}

impl Gun {
    pub(crate) fn mediocre() -> Self {
        Self {
            enabled: true,
            cd: 20,
            bullets: Bullets::MinorBullet(40),
            repeats: 0,
            repeat_cd: 0,
            r_point: 0,
            tick: 1,
            barrels: 1,
            barrel_spacing: 0.0,
            center_offset: 40.0,
            scatter_barrels: false,
        }
    }

    pub(crate) fn basic_repeater(repeats: u16) -> Self {
        Self {
            enabled: true,
            cd: 25,
            bullets: Bullets::MinorBullet(50),
            repeats,
            repeat_cd: 5,
            r_point: 0,
            tick: 1,
            barrels: 1,
            barrel_spacing: 0.0,
            center_offset: 40.0,
            scatter_barrels: false,
        }
    }

    pub(crate) fn sniper() -> Self {
        Self {
            enabled: true,
            cd: 150,
            bullets: Bullets::MinorBullet(200),
            repeats: 0,
            repeat_cd: 0,
            r_point: 0,
            tick: 1,
            barrels: 1,
            barrel_spacing: 0.0,
            center_offset: 40.0,
            scatter_barrels: false,
        }
    }

    pub(crate) fn bomber() -> Self {
        Self {
            enabled: true,
            cd: 120,
            bullets: Bullets::Bomb(ExplosionProperties::small(), 75),
            repeats: 0,
            repeat_cd: 0,
            r_point: 0,
            tick: 1,
            barrels: 1,
            barrel_spacing: 0.0,
            center_offset: 40.0,
            scatter_barrels: false,
        }
    }

    pub(crate) fn extended_barrels(mut self, num: u16, spacing: f32) -> Self {
        self.barrels += num;
        self.barrel_spacing = spacing;

        self.scatter_barrels = true;
        self.cd /= num;

        self
    }

    pub(crate) fn offset(mut self, off: f32) -> Self {
        self.center_offset = off;
        self
    }
}

#[derive(Component)]
pub(crate) struct Farmhouse {}

#[derive(Component)]
pub(crate) struct FieldSensor {
    pub(crate) attached_to: Entity,
}

impl FieldSensor {
    pub(crate) fn new(piece: Entity) -> Self {
        Self { attached_to: piece }
    }
}

#[derive(Component)]
pub(crate) struct CollisionExplosion {
    // entities with this component explode whenever they hit anything
    pub(crate) explosion: ExplosionProperties,
}

#[derive(Copy, Clone, Component)]
pub struct ExplosionProperties {
    pub radius: f32,
    pub damage: f32,
}

impl ExplosionProperties {
    pub fn small() -> Self {
        Self {
            radius: 100.0,
            damage: 2.0,
        }
    }
}

#[derive(Component)]
pub struct StaticWall;

pub(crate) struct Missile {
    pub(crate) decelerator: f32,
    pub(crate) acc_profile: f32,
    pub(crate) target_lock: Option<PieceId>, // missiles can be target-locked to a gamepiece. they will ignore the pathfollower after locking.
    pub(crate) intercept_burn: f32, // during the intercept burn, it heavily side-corrects, angular positioning gets much more accurate, and it accelerates at intercept_burn_power.
    pub(crate) intercept_burn_power: f32, // intercept burn begins when it's locked to a target that is nearer than intercept_burn units away.
}

impl Missile {
    pub(crate) fn ballistic() -> Self {
        Self {
            decelerator: 0.01,
            acc_profile: 8.0,
            target_lock: None,
            intercept_burn: 0.0,
            intercept_burn_power: 60.0,
        }
    }

    pub(crate) fn cruise() -> Self {
        // accelerates faster but has a lower max speed
        Self {
            decelerator: 0.04,
            acc_profile: 18.0,
            target_lock: None,
            intercept_burn: 0.0,
            intercept_burn_power: 60.0,
        }
    }

    pub(crate) fn hypersonic() -> Self {
        Self {
            decelerator: 0.015,
            acc_profile: 24.0,
            target_lock: None,
            intercept_burn: 0.0,
            intercept_burn_power: 60.0,
        }
    }

    pub(crate) fn with_intercept_burn(mut self, burn: f32) -> Self {
        self.intercept_burn = burn;
        self
    }
}

pub(crate) struct Ship {
    pub(crate) speed: f32,
    pub(crate) acc_profile: f32, // in percentage of speed
}

impl Ship {
    pub(crate) fn normal() -> Self {
        return Self {
            speed: 16.0,
            acc_profile: 0.33,
        };
    }

    pub(crate) fn fast() -> Self {
        return Self {
            speed: 32.0,
            acc_profile: 0.5,
        };
    }

    pub(crate) fn slow() -> Self {
        return Self {
            speed: 12.0,
            acc_profile: 0.33,
        };
    }
}

impl SpaceshipKinematics for Ship {
    fn to_position(&mut self, offset: Vec2, angle: f32, vel: Vec2, angvel: f32) -> KinematicResult {
        if offset.length() > 15.0 {
            let mut thrust = if loopify(angle, offset.to_angle()).abs() < PI / 6.0 {
                Vec2::from_angle(angle)
                    * linear_maneuvre(
                        offset,
                        vel,
                        self.speed * 10.0,
                        self.speed * 10.0 * self.acc_profile,
                    )
            } else {
                Vec2::ZERO
            };
            thrust -= vel.project_onto(offset.perp()) * 0.2; // linear deviation correction thrusters
            let torque = (-loopify(offset.to_angle(), angle) * 10.0 - angvel * 7.0) * 80.0;
            KinematicResult::Thrust(thrust, torque)
        } else {
            KinematicResult::Done(vel * -0.1, 0.0) // simple holding pattern
        }
    }

    fn to_angle(&mut self, offset: f32, vel: Vec2, angvel: f32) -> KinematicResult {
        let torque = (offset * 10.0 - angvel * 8.0) * 120.0;
        if offset.abs() < 0.1 {
            KinematicResult::Done(vel * -0.1, torque)
        } else {
            KinematicResult::Thrust(vel * -0.1, torque)
        }
    }
}

impl SpaceshipKinematics for Missile {
    fn to_position(&mut self, offset: Vec2, angle: f32, vel: Vec2, angvel: f32) -> KinematicResult {
        if offset.length() > 20.0 && offset.length() < self.intercept_burn {
            // if we're in intercept strike mode
            let mut impulse = Vec2::from_angle(angle) * self.intercept_burn_power;
            impulse -= vel.project_onto(offset.perp()) * 0.2;
            KinematicResult::Thrust(
                impulse,
                -loopify(offset.to_angle(), vel.to_angle()) * 1600.0 - angvel * 1120.0,
            )
        } else if offset.length() > 30.0 {
            let mut impulse = Vec2::from_angle(angle) * self.acc_profile * offset.length().sqrt()
                / 10.0
                - vel * self.decelerator * 2.0;
            impulse -= vel.project_onto(offset.perp()) * 0.1;
            KinematicResult::Thrust(
                impulse,
                -loopify(offset.to_angle(), vel.to_angle()) * 800.0 - angvel * 560.0,
            )
        } else {
            KinematicResult::Done(Vec2::ZERO, 0.0)
        }
    }

    fn to_angle(&mut self, _offset: f32, _vel: Vec2, _angvel: f32) -> KinematicResult {
        KinematicResult::Done(Vec2::ZERO, 0.0) // missiles don't do the angles thing
    }

    fn node_override(&mut self) -> Option<PathNode> {
        match self.target_lock {
            Some(target) => Some(PathNode::Target(target)),
            None => None,
        }
    }

    fn override_complete(&mut self) {
        self.target_lock = None;
    }

    fn sensor_tripped(&mut self, thing: PieceId, tp: PieceType) {
        match tp {
            PieceType::Bullet => {}
            _ => {
                self.target_lock = Some(thing);
            }
        }
    }
}

#[derive(Component)]
pub struct Spaceshipoid {
    pub pathfollower: PathFollower,
    pub kinematics: Box<dyn SpaceshipKinematics + Send + Sync>,
}

impl Spaceshipoid {
    pub fn of(ship: impl SpaceshipKinematics + Send + Sync + 'static, x: f32, y: f32) -> Self {
        Self {
            pathfollower: PathFollower::start(x, y),
            kinematics: Box::new(ship),
        }
    }

    pub fn sensor_tripped(&mut self, thing: PieceId, tp: PieceType) {
        self.kinematics.sensor_tripped(thing, tp);
    }
}

#[derive(Component)]
pub struct LaserNode {
    pub slots: Arc<RwLock<Vec<Entity>>>, // every laser node within range
    pub allowable: usize,                // the maximum number of connections it can act on
}

impl LaserNode {
    pub fn new(slots: usize) -> Self {
        Self {
            slots: Arc::new(RwLock::new(Vec::new())),
            allowable: slots,
        }
    }

    pub fn disconnect(&self, thing: Entity) {
        if let Some(index) = self.slots.read().unwrap().iter().position(|x| *x == thing) {
            self.slots.write().unwrap().remove(index);
        }
    }

    pub fn connect(&self, thing: Entity) {
        println!("connected");
        for slot in self.slots.read().unwrap().iter() {
            // don't duplicate connections
            if *slot == thing {
                return;
            }
        }
        self.slots.write().unwrap().push(thing);
    }

    pub fn recalculate(&self, all_nodes: &Query<&LaserNode>) {
        // only the top `allowable` connections will be actually serviced, so we need to sort `slots` to make it as even as possible
        // we sort by the number of nodes connected to the node in consideration.
        self.slots.write().unwrap().sort_by(|a, b| {
            if let Ok(a) = all_nodes.get(*a) {
                if let Ok(b) = all_nodes.get(*b) {
                    return a
                        .slots
                        .read()
                        .unwrap()
                        .len()
                        .cmp(&b.slots.read().unwrap().len());
                }
            }
            std::cmp::Ordering::Equal
        });
    }
}

#[derive(Component)]
pub struct ScrapShip {
    pub seeds_in_range: Vec<Entity>,
    pub ind: usize,
}

impl ScrapShip {
    pub fn new() -> Self {
        Self {
            seeds_in_range: vec![],
            ind: 0,
        }
    }

    pub fn enter(&mut self, e: Entity) {
        for seed in self.seeds_in_range.iter() {
            // don't duplicate
            if *seed == e {
                return;
            }
        }
        self.seeds_in_range.push(e);
    }

    pub fn leave(&mut self, e: Entity) {
        if let Some(index) = self.seeds_in_range.iter().position(|x| *x == e) {
            self.seeds_in_range.remove(index);
        }
    }
}

pub trait TargetingAlgorithm {
    fn will_attack(&self, tp: PieceType) -> bool;

    fn get_target_angle(&self, off_vec: Vec2, other_vel: Vec2) -> f32;

    fn swivel_kinematics(&self, offset: f32, vel: f32) -> f32 {
        (offset * 20.0 - vel.cbrt() * 5.0) * 20000.0
    }
}

pub struct StandardTargeting;

impl TargetingAlgorithm for StandardTargeting {
    fn will_attack(&self, tp: PieceType) -> bool {
        match tp {
            PieceType::Bullet => false,
            _ => true,
        }
    }

    fn get_target_angle(&self, off_vec: Vec2, _: Vec2) -> f32 {
        off_vec.to_angle()
    }
}

pub struct SmartTargeting;

impl TargetingAlgorithm for SmartTargeting {
    fn will_attack(&self, tp: PieceType) -> bool {
        match tp {
            PieceType::Bullet => false,
            _ => true,
        }
    }

    fn get_target_angle(&self, off_vec: Vec2, vel: Vec2) -> f32 {
        // see https://www.gamedev.net/forums/topic/582894-target-leading-in-2d/
        const BULLET_SPEED: f32 = 450.0 / 30.0;
        let a = vel.dot(vel) - BULLET_SPEED * BULLET_SPEED;
        let b = 2.0 * vel.dot(off_vec);
        let c = off_vec.dot(off_vec);

        let in_root = b * b - 4.0 * a * c;
        if in_root < 0.0 {
            // because we're taking the square root, if in_root is less than 0, there are no real firing solutions
            return off_vec.to_angle();
        }
        let r1 = (b + in_root.sqrt()) / (2.0 * a);
        let r2 = (b - in_root.sqrt()) / (2.0 * a);
        let t = if r1 < r2 { r1 } else { r2 };
        return (off_vec + vel * t).to_angle();
    }

    fn swivel_kinematics(&self, offset: f32, vel: f32) -> f32 {
        (offset * 400.0 - vel * 20.0) * 1000.0
    }
}

#[derive(Component)]
pub struct Turret {
    pub targeting_algorithm: Box<dyn TargetingAlgorithm + Send + Sync>,
    pub in_range: Vec<Entity>,
}

impl Turret {
    pub fn new(thing: impl TargetingAlgorithm + Send + Sync + 'static) -> Self {
        Self {
            targeting_algorithm: Box::new(thing),
            in_range: vec![],
        }
    }

    pub fn enter(&mut self, e: Entity) {
        for seed in self.in_range.iter() {
            // don't duplicate
            if *seed == e {
                return;
            }
        }
        self.in_range.push(e);
    }

    pub fn leave(&mut self, e: Entity) {
        if let Some(index) = self.in_range.iter().position(|x| *x == e) {
            self.in_range.remove(index);
        }
    }
}

#[derive(Component)]
pub struct PresolveVelocity(pub Vec2); // the velocity before the last solver step, useful for collision handling

#[derive(Component)]
pub struct Sensored {
    // an object with a sensor attached
    pub sensor: Entity,
}
