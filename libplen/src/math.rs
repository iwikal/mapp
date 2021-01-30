use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};
use serde_derive::{Serialize, Deserialize};
use crate::constants;

pub type Vec2 = ultraviolet::Vec2;

pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

pub fn vec2_from_direction(angle: f32, length: f32) -> Vec2 {
    vec2(
        angle.cos() * length,
        angle.sin() * length,
    )
}

pub fn modulo(x: f32, div: f32) -> f32 {
    (x % div + div) % div
}

pub fn wrap_around(pos: Vec2) -> Vec2 {
    vec2(
        modulo(pos.x, constants::WORLD_SIZE),
        modulo(pos.y, constants::WORLD_SIZE),
    )
}

pub fn angle_diff(source_angle: f32, target_angle: f32) -> f32 {
    // From https://stackoverflow.com/a/7869457
    use std::f32::consts::PI;
    modulo(target_angle - source_angle + PI, 2. * PI) - PI
}
