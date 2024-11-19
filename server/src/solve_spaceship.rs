/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// space kinematics solvers
// in a separate file because main.rs is getting bloated


// everything uses bevy's Vec2, which conveniently is also what rapier provides! so most of these functions are pretty easy to use
use bevy::math::f32::Vec2;
use std::f32::consts::PI;
use common::pathfollower::PathNode;
use common::PieceId;


// the quintessential point-a to point-b linear space maneuver, but stateless
// you pass it the current position, the current velocity, and the goalpost, and it outputs your target acceleration
// relies on course-correction; if your spacecraft is pointing in the wrong direction, it won't end up in the right place.
// best used in conjunction with space_gyro.
// because it's stateless, it can respond adaptively to unexpected changes or inaccuracies without significantly degrading quality.
// physically naive: it does not account for mass, so it expects that if you pass it a current velocity of [-2, 3] and it returns [1, 1], the next velocity
// you pass it will be [-1, 4]. bevy_rapier2d does not have this behavior, so some tuning may be required.
// cap speed is not used naively: the algorithm assumes you want to travel at that speed most of the time. nominal_acceleration has the same behavior.
// it DOES naively assume that the speed is never greater than cap_speed
pub fn linear_maneuvre(delta : Vec2, current_velocity : Vec2, cap_speed : f32, nominal_acceleration : f32) -> f32 {
    let decel_time = current_velocity.length() / nominal_acceleration;
    let decel_dist = 0.5 * nominal_acceleration * decel_time * decel_time; // second integral of acceleration = position = p(t) = 1/2 * nominal_acceleration * t^2
    // t = decel_time
    /*if (delta - current_velocity).length() > delta.length() {
        return nominal_acceleration;
    }
    else */if delta.length() <= decel_dist {
        return -1.0 * nominal_acceleration;
    }
    else if current_velocity.length() < cap_speed {
        return nominal_acceleration;
    }
    else {
        return 0.0;
    }
}

pub fn coterminal(mut thing : f32) -> f32 {
    // todo: optimize
    while thing < 0.0 {
        thing += 2.0 * PI;
    }
    while thing >= 2.0 * PI {
        thing -= 2.0 * PI;
    }
    return thing;
}

pub fn loopify(one : f32, two : f32) -> f32 { // return the circle displacement between two angles
    // it returns the *minimum amount* you'd have to add to `one` for it to be *coterminal* with `two`
    // this property means that it will never return a number with absolute value greater than PI, because there would be a better direction to take!
    let rdist = two - one;
    if rdist > PI { // if we would have to travel more than 180 degrees, there is guaranteed to be a better solution by loopifying
        return rdist - 2.0 * PI; // if we're at 20,340 the normal response is 320 degrees. we want it to instead return -40 degrees.
    }
    if rdist < -PI {
        return rdist + 2.0 * PI; // if we're at 340,20 the normal response is -320, we want it to return 40deg
    }
    // literally looping around zero
    rdist
}


// does automatic course correction using an algorithm similar to linear_maneuvre
// todo: make this crisper (we should always land EXACTLY on point, without jostling!)
pub fn space_gyro(current_position : f32, goal_position : f32, current_rotational_velocity : f32, cap_rotational_velocity : f32, nominal_radial_acceleration : f32) -> f32 {
    let decel_time = current_rotational_velocity.abs() / nominal_radial_acceleration;
    let decel_rotation = 0.5 * nominal_radial_acceleration * decel_time * decel_time;
    let delta = loopify(current_position, goal_position);
    if delta.abs() < decel_rotation {
        return delta.signum() * nominal_radial_acceleration.min(current_rotational_velocity.abs());
    }
    else if current_rotational_velocity.signum() == delta.signum() { // we need to turn around!
        return delta.signum() * -1.0 * nominal_radial_acceleration;
    }
    else if current_rotational_velocity.abs() < cap_rotational_velocity {
        return delta / delta.abs() * -1.0 * nominal_radial_acceleration;
    }
    else {
        return 0.0;
    }
}


pub enum KinematicResult {
    Thrust(Vec2, f32), // apply impulses
    Noop, // we don't want to do anything
    Done(Vec2, f32) // calculated offsets and velocities are in target range, next question!
    // (allows the spaceshipoid to continue providing thrust data, if it wants to airbrake or something)
}


pub trait SpaceshipKinematics {
    // move to a position (it doesn't have to know the details, just that it has to get there), given the offset and the current angle of the ship
    fn to_position(&mut self, offset : Vec2, angle : f32, vel : Vec2, angvel : f32) -> KinematicResult; // (impulse, torque_impulse)

    // same as to_position, except it's tracking a piece rather than going to a static point
    fn to_position_tracking(&mut self, offset : Vec2, angle : f32, vel : Vec2, angvel : f32) -> KinematicResult {
        self.to_position(offset, angle, vel, angvel)
    }

    // rotate to an angle given the current angle offset
    fn to_angle(&mut self, offset : f32, vel : Vec2, angvel : f32) -> KinematicResult; // torque_impulse

    // the spaceshipoid may optionally provide a supplementary target through this function
    fn node_override(&mut self) -> Option<PathNode> {
        None
    }

    // if the overridden target was acquired by report (or failed to target track), this is called:
    fn override_complete(&mut self) {
        
    }

    // a sensor attached to this thing has collided with an enemy piece!
    fn sensor_tripped(&mut self, _thing : PieceId) {

    }

    // the return values are automatically multiplied by mass to get sane outputs
}