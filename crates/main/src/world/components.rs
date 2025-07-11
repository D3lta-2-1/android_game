use nalgebra::Vector2;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub actual: Vector2<f32>,
    pub last_tick: Vector2<f32>,
}

impl Deref for Position {
    type Target = Vector2<f32>;
    fn deref(&self) -> &Self::Target {
        &self.actual
    }
}

impl DerefMut for Position {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.actual
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity(pub Vector2<f32>);

impl Deref for Velocity {
    type Target = Vector2<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Velocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Acceleration(pub Vector2<f32>);

impl Deref for Acceleration {
    type Target = Vector2<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Acceleration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mass {
    pub mass: f32,
    pub inv_mass: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubjectToPhysic(pub usize);
