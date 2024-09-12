// space kinematics solvers
// in a separate file because main.rs is getting bloated


// everything uses bevy's Vec2, which conveniently is also what rapier provides! so most of these functions are pretty easy to use
use bevy::math::f32::Vec2;
use std::f32::consts::PI;


// the quintessential point-a to point-b linear space maneuver, but stateless
// you pass it the current position, the current velocity, and the goalpost, and it outputs your target acceleration
// relies on course-correction; if your spacecraft is pointing in the wrong direction, it won't end up in the right place.
// best used in conjunction with space_gyro.
// because it's stateless, it can respond adaptively to unexpected changes or inaccuracies without significantly degrading quality.
// physically naive: it does not account for mass, so it expects that if you pass it a current velocity of [-2, 3] and it returns [1, 1], the next velocity
// you pass it will be [-1, 4]. bevy_rapier2d does not have this behavior, so some tuning may be required.
// cap speed is not used naively: the algorithm assumes you want to travel at that speed most of the time. nominal_acceleration has the same behavior.
// it DOES naively assume that the speed is never greater than cap_speed
pub fn linear_maneuvre(current_position : Vec2, goal_position : Vec2, current_velocity : Vec2, cap_speed : f32, nominal_acceleration : f32) -> f32 {
    let decel_time = current_velocity.length() / nominal_acceleration;
    let decel_dist = 0.5 * nominal_acceleration * decel_time * decel_time; // second integral of acceleration = position = p(t) = 1/2 * nominal_acceleration * t^2
    // t = decel_time
    let delta = goal_position - current_position;
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
    while (thing < 0.0) {
        thing += 2.0 * PI;
    }
    while (thing >= 2.0 * PI) {
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
    else if (current_rotational_velocity.signum() == delta.signum()) { // we need to turn around!
        return delta.signum() * -1.0 * nominal_radial_acceleration;
    }
    else if (current_rotational_velocity.abs() < cap_rotational_velocity) {
        return delta / delta.abs() * -1.0 * nominal_radial_acceleration;
    }
    else {
        return 0.0;
    }
}