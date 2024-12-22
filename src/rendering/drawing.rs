/*use std::sync::Arc;
use std::time::{Duration, Instant};
use vello::kurbo::{Affine, Vec2};
use vello::Scene;


pub struct Transform {
    pub pos: Vec2,
    pub rotation: f64,
    pub scale: Vec2,
}

impl Transform {

    pub fn new(pos: impl Into<Vec2>, rotation: f64, scale: impl Into<Vec2>) -> Self {
        Self { pos: pos.into(), rotation, scale: scale.into() }
    }
    pub fn from_pos_and_rot(pos: impl Into<Vec2>, rotation: f64) -> Self {
        Self { pos: pos.into(), rotation, scale: Vec2::new(1.0, 1.0) }
    }

    pub fn from_pos(pos: impl Into<Vec2>) -> Self {
        Self { pos: pos.into(), rotation: 0.0, scale: Vec2::new(1.0, 1.0) }
    }

    pub fn to_affine(&self) -> Affine {
        Affine::rotate(self.rotation).then_scale_non_uniform(self.scale.x, self.scale.y).then_translate(self.pos)
    }

    pub fn interpolate(&self, other: &Transform, t: f64) -> Self {
        let b = t;
        let a = 1.0 - t;
        Self {
            pos: a * self.pos + b * other.pos,
            rotation: a * self.rotation + b * other.rotation,
            scale: a * self.scale + b * other.scale,
        }
    }
}

pub enum Interpolation {
    /** Linear interpolation between two points **/
    Linear(Transform, Transform),
    /** No interpolation **/
    None(Transform),
    /** Linear interpolation with a function to change progress rate **/
    Custom(Transform, Transform, fn(f64) -> f64),
}

impl Interpolation {
    pub fn interpolate(&self, t: f64) -> Affine {
        match self {
            Interpolation::Linear(a, b) =>
                Transform::interpolate(a, b, t)
                    .to_affine(),
            Interpolation::None(a) =>
                a.to_affine(),
            Interpolation::Custom(a, b, trans) =>
                Transform::interpolate(a, b, trans(t))
                    .to_affine(),
        }
    }
}

pub struct DrawCommand {
    sprite: Arc<Scene>,
    interpolation: Interpolation,
}

impl DrawCommand {
    pub fn new(sprite: Arc<Scene>, interpolation: Interpolation) -> Self {
        Self { sprite, interpolation }
    }

    pub fn append_to_scene(&self, scene: &mut Scene, t: f64) {
        scene.append(&self.sprite, Some(self.interpolation.interpolate(t)))
    }
}

pub struct CommandBundle {
    pub commands: Vec<DrawCommand>,
    pub camera_transform: Interpolation,
    pub tick_start: Instant,
}

impl CommandBundle {
    pub fn new_empty() -> Self {
        Self {
            commands: vec![],
            camera_transform: Interpolation::None(Transform::from_pos((0.0, 0.0))),
            tick_start: Instant::now(),
        }
    }

    pub fn append_to_scene(&self, scene: &mut Scene, tick_duration: &Duration) {
        let progress = self.tick_start.elapsed().as_secs_f64();
        let t = progress / tick_duration.as_secs_f64();
        for command in self.commands.iter() {
            command.append_to_scene(scene, t);
        }
    }
}


pub struct CommandBundle {}

impl CommandBundle {
    pub fn new_empty() -> Self {
        Self{}
    }
}*/