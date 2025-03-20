mod constraints;

use nalgebra::{DMatrix, DVector, Vector2};
use crate::pendulum::constraints::Constraint;

/// this is my first attempt to implement an impulse solver as described by Erin Catto in 2014

pub struct Body {
    pub position: Vector2<f32>, // m
    velocity: Vector2<f32>, // m/s
    inv_mass: f32, // kg^-1 // inverse mass sound more useful than mass, and a mass of 0 doesn't make any sense
}

pub struct PendulumSystem {
    pub bodies: Vec<Body>,
    constraints: Vec<Box<dyn Constraint>>,
    gravity: Vector2<f32>,
    pub(crate) time_step: f32,
}

impl PendulumSystem {

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
                Box::new(constraints::DistanceConstraint {
                    body_a: 0,
                    body_b: 1,
                    distance: 1.0,
                    bias: 0.01,
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
                    position: Vector2::new(1.0, 0.0),
                    velocity: Vector2::new(0.0, 4.0),
                    inv_mass: 1.0,
                },
                Body {
                    position: Vector2::new(1.0, 1.0),
                    velocity: Vector2::new(0.0, 0.0),
                    inv_mass: 1.0,
                },
                Body {
                    position: Vector2::new(1.0, 0.0),
                    velocity: Vector2::new(4.0, 0.0),
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

    pub fn solve(&mut self) {
        // when we get here, the new velocity probably violates the constraints
        // TODO: use sparse matrix, and find a better way to write theses equations
        let mut jacobian = DMatrix::zeros(self.constraints.len(), self.bodies.len() * 2);
        let mut b = DVector::zeros(self.constraints.len());
        for (i,(row, constraint)) in jacobian.row_iter_mut().zip(self.constraints.iter()).enumerate() {
            b[i] = constraint.set_partial_derivative(&self.bodies, row);
        }
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
        let jv = jacobian.clone() * v;
        let k = jacobian.clone() * mass_matrix * jacobian.transpose();
        let lu = k.lu();
        let b = -jv + (1.0 / self.time_step) * b;
        let lambda = lu.solve(&b).unwrap();
        let correction = jacobian.transpose() * lambda;
        for (i, body) in self.bodies.iter_mut().enumerate() {
            body.velocity.x += correction[i * 2];
            body.velocity.y += correction[i * 2 + 1];
        }
    }
}