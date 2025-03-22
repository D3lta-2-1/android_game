pub mod constraints;

use std::iter;
use nalgebra::{DMatrix, DVector, Vector2};
use crate::world::constraints::{Constraint, ConstraintWidget};

/// this is my first attempt to implement an impulse solver as described by Erin Catto in 2014

#[derive(Debug)]
pub struct Body {
    position: Vector2<f32>, // m
    velocity: Vector2<f32>, // m/s
    inv_mass: f32, // kg^-1 // inverse mass sound more useful than mass, and a mass of 0 doesn't make any sense
}

pub struct World {
    pub bodies: Vec<Body>,
    constraints: Vec<Box<dyn Constraint>>,
    gravity: Vector2<f32>,
    pub time_step: f32,
}

impl World {

    pub fn simple(time_step: f32) -> Self {
        Self {
            bodies: vec![
                Body {
                    position: Vector2::new(1.0, 0.0),
                    velocity: Vector2::new(0.0, 0.0),
                    inv_mass: 1.0,
                },
            ],
            constraints: vec![
                Box::new(constraints::AnchorConstraint {
                    body: 0,
                    anchor: Vector2::new(0.0, 0.0),
                    distance: 1.0,
                    bias: 0.01,
                }),
            ],
            gravity: Vector2::new(0.0, -9.81),
            time_step,
        }
    }

    pub fn double(time_step: f32) -> Self {
        Self {
            bodies: vec![
                Body {
                    position: Vector2::new(1.0, 0.0),
                    velocity: Vector2::new(0.0, 0.0),
                    inv_mass: 1.0,
                },
                Body {
                    position: Vector2::new(1.0, 1.0),
                    velocity: Vector2::new(-10.0, 0.0),
                    inv_mass: 1.0,
                },
            ],
            constraints: vec![
                Box::new(constraints::AnchorConstraint {
                    body: 0,
                    anchor: Vector2::new(0.0, 0.0),
                    distance: 1.0,
                    bias: 0.05,
                }),
                Box::new(constraints::DistanceConstraint {
                    body_a: 0,
                    body_b: 1,
                    distance: 1.0,
                    bias: 0.05,
                }),
            ],
            gravity: Vector2::new(0.0, -9.81),
            time_step,
        }
    }

    pub fn triple(time_step: f32) -> Self {
        Self {
            bodies: vec![
                Body {
                    position: Vector2::new(1.0, 1.0),
                    velocity: Vector2::new(0.0, 4.0),
                    inv_mass: 1.0,
                },
                Body {
                    position: Vector2::new(1.0, 2.0),
                    velocity: Vector2::new(0.0, 0.0),
                    inv_mass: 1.0,
                },
                Body {
                    position: Vector2::new(1.0, 1.0),
                    velocity: Vector2::new(4.0, 0.0),
                    inv_mass: 1.0,
                },
            ],
            constraints: vec![
                Box::new(constraints::AnchorConstraint {
                    body: 0,
                    anchor: Vector2::new(0.0, 1.0),
                    distance: 1.0,
                    bias: 0.01,
                }),
                Box::new(constraints::DistanceConstraint {
                    body_a: 0,
                    body_b: 1,
                    distance: 1.0,
                    bias: 0.01,
                }),
                Box::new(constraints::DistanceConstraint {
                    body_a: 1,
                    body_b: 2,
                    distance: 1.0,
                    bias: 0.01,
                }),
            ],
            gravity: Vector2::new(0.0, -9.81),
            time_step,
        }
    }

    pub fn rope(time_step: f32) -> Self {
        const N: usize = 30;
        const DISTANCE: f32 = 0.3;

        let bodies: Vec<Body> = (0..N).map(|i| {
            Body {
                position: Vector2::new(((i + 1) as f32) * DISTANCE - N as f32 * DISTANCE * 0.5, 0.0),
                velocity: Vector2::new(0.0, 10.0),
                inv_mass: 1.0 / 0.1,
            }
        }).collect();

        let constraints: Vec<Box<dyn Constraint>> = iter::once({
            let b: Box<dyn Constraint> = Box::new(constraints::AnchorConstraint {
                body: 0,
                anchor: Vector2::new(-(N as f32 * DISTANCE * 0.5), 0.0),
                distance: DISTANCE,
                bias: 0.01,
            });
            b
        }).chain((1..N).map(|i| {
            let b: Box<dyn Constraint> = Box::new(constraints::DistanceConstraint {
                body_a: i - 1,
                body_b: i,
                distance: DISTANCE,
                bias: 0.01,
            });
            b
        })).chain(iter::once({
            let b: Box<dyn Constraint> = Box::new(constraints::AnchorConstraint {
                body: N - 1,
                anchor: Vector2::new((N-2) as f32 * DISTANCE * 0.5, 0.0),
                distance: DISTANCE,
                bias: 0.01,
            });
            b
        })).collect();

        Self {
            bodies,
            constraints,
            gravity: Vector2::new(0.0, -9.81),
            time_step,
        }
    }

    pub fn pendulum_in_rail(time_step: f32) -> Self {
        Self {
            bodies: vec![
                Body {
                    position: Vector2::new(0.0, 0.0),
                    velocity: Vector2::new(0.0, 0.0),
                    inv_mass: 1.0,
                },
                Body {
                    position: Vector2::new(1.0, 0.0),
                    velocity: Vector2::new(0.0, 0.0),
                    inv_mass: 1.0,
                },
            ],
            constraints: vec![
                Box::new(constraints::DistanceConstraint {
                    body_a: 0,
                    body_b: 1,
                    distance: 1.0,
                    bias: 0.01,
                }),
                Box::new(constraints::HorizontalRail {
                    body: 0,
                    y_position: 0.0,
                    bias: 0.01,
                }),
            ],
            gravity: Vector2::new(0.0, -9.81),
            time_step,
        }
    }
    
    pub fn square(time_step: f32) -> Self {


        let make = |x, y| {
            Body{
                position: Vector2::new(x, y),
                velocity: Vector2::new(0.0, 0.0),
                inv_mass: 1.0,
            }
        };

        Self {
            bodies: vec![
                make(0.25, 0.25),
                make(0.0, 0.0),
                make(0.25, -0.25),
                make(0.5, 0.0),

            ],
            constraints: vec![
                Box::new(constraints::AnchorConstraint {
                    body: 0,
                    anchor: Vector2::new(-1.25, 0.25),
                    distance: 1.5,
                    bias: 0.01,
                }),
                Box::new(constraints::DistanceConstraint {
                    body_a: 0,
                    body_b: 1,
                    distance: f32::sqrt(1.0/8.0),
                    bias: 0.01,
                }),
                Box::new(constraints::DistanceConstraint {
                    body_a: 1,
                    body_b: 2,
                    distance: f32::sqrt(1.0/8.0),
                    bias: 0.01,
                }),
                Box::new(constraints::DistanceConstraint {
                    body_a: 2,
                    body_b: 3,
                    distance: f32::sqrt(1.0/8.0),
                    bias: 0.01,
                }),
                Box::new(constraints::DistanceConstraint {
                    body_a: 3,
                    body_b: 0,
                    distance: f32::sqrt(1.0/8.0),
                    bias: 0.01,
                }),
                Box::new(constraints::DistanceConstraint {
                    body_a: 1,
                    body_b: 3,
                    distance: 0.5,
                    bias: 0.01,
                }),
            ],
            gravity: Vector2::new(0.0, -9.8),
            time_step,
        }
    }

    // there is no broad-phase nor narrow-phase collision detection

    pub fn integrate(&mut self) {
        let gravity = self.gravity;
        for body in &mut self.bodies {
            // integrate position, BEFORE velocity
            body.position += body.velocity * self.time_step;

            // integrate velocity, the new velocity violates the constraints, therefore we need to correct it in the solve method
            body.velocity += gravity * self.time_step;
        }
    }

    pub fn solve(&mut self) -> WorldSnapshot {
        // when we get here, the new velocity probably violates the constraints
        // TODO: use sparse matrix, and find a better way to write theses equations
        let mut j = DMatrix::zeros(self.constraints.len(), self.bodies.len() * 2);
        let mut b = DVector::zeros(self.constraints.len());
        for (i,(row, constraint)) in j.row_iter_mut().zip(self.constraints.iter()).enumerate() {
            b[i] = constraint.set_partial_derivative(&self.bodies, row);
        }
        let jt = j.transpose();
        let mass_matrix = DMatrix::from_fn(self.bodies.len() * 2, self.bodies.len() * 2, |i, j| {
            if i == j {
                self.bodies[i >> 1].inv_mass
            } else {
                0.0
            }
        });
        let v = DVector::from_fn(self.bodies.len() * 2, |i, _| {
            if i & 1 == 0 {
                self.bodies[i >> 1].velocity.x
            } else {
                self.bodies[i >> 1].velocity.y
            }
        });
        let jv = &j * v;
        let k = &j * mass_matrix * &jt;
        let lu= k.lu();
        let b = -jv + (1.0 / self.time_step) * b;
        let lambda = lu.solve(&b).unwrap();
        let correction = jt * &lambda;
        for (i, body) in self.bodies.iter_mut().enumerate() {
            body.velocity.x += correction[i * 2] * body.inv_mass;
            body.velocity.y += correction[i * 2 + 1] * body.inv_mass;
        }


        WorldSnapshot {
            pos: self.bodies.iter().map(|b| b.position).collect(),
            links: self.constraints.iter().zip(lambda.as_slice()).map(|(c, v)| (c.widget(), *v)).collect(),
        }
    }
}

#[derive(Default)]
pub struct WorldSnapshot {
    pub pos: Vec<Vector2<f32>>,
    pub links: Vec<(ConstraintWidget, f32)>,
}