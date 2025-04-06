pub mod constraints;

use std::iter;
use nalgebra::{DMatrix, DVector, Vector2};
use crate::world::constraints::{Constraint, ConstraintWidget};

/// this is my first attempt to implement an impulse solver as described by Erin Catto in 2014

#[derive(Debug)]
pub struct Body {
    pub pos: Vector2<f32>, // m
    pub last_pos: Vector2<f32>,
    pub velocity: Vector2<f32>, // m/s
    inv_mass: f32, // kg^-1 // inverse mass sound more useful than mass, and a mass of 0 doesn't make any sense
}

pub struct World {
    pub bodies: Vec<Body>,
    constraints: Vec<Box<dyn Constraint>>,
    gravity: Vector2<f32>,
    pub time_step: f32,
    age: u32
}

impl World {
    pub fn empty(time_step: f32) -> Self {
        Self {
            bodies: vec![],
            constraints: vec![],
            gravity: Vector2::new(0.0, -9.81),
            time_step,
            age: 0
        }
    }

    pub fn clear(&mut self) {
        self.bodies.clear();
        self.constraints.clear();
        self.age = 0;
    }

    pub fn add_body(&mut self, pos: Vector2<f32>, velocity: Vector2<f32>, inv_mass: f32) {
        let body = Body {
            pos,
            last_pos: pos - velocity * self.time_step,
            velocity,
            inv_mass,
        };
        self.bodies.push(body);
    }

    pub fn simple(&mut self) {
        self.clear();
        self.add_body(Vector2::new(1.0, 0.0), Vector2::new(0.0, 0.0), 1.0);
        self.constraints = vec![
            Box::new(constraints::AnchorConstraint {
                body: 0,
                anchor: Vector2::new(0.0, 0.0),
                distance: 1.0,
            }),
        ];
        self.gravity = Vector2::new(0.0, -9.81);
    }

    pub fn double(&mut self) {
        self.clear();
        self.add_body(Vector2::new(1.0, 0.0), Vector2::new(0.0, 0.0), 1.0);
        self.add_body(Vector2::new(1.0, 1.0), Vector2::new(-0.0, 0.0), 1.0);
        self.constraints = vec![
            /*Box::new(constraints::DistanceConstraint {
                body_a: 0,
                body_b: 1,
                distance: 1.0,
            }),*/
            Box::new(constraints::AnchorConstraint {
                body: 0,
                anchor: Vector2::new(0.0, 0.0),
                distance: 1.0,
            }),


        ];
        self.gravity = Vector2::new(0.0, -9.81);
    }

    // there is no broad-phase nor narrow-phase collision detection

    /*pub fn integrate(&mut self) {
        let gravity = self.gravity;
        for body in &mut self.bodies {
            // integrate position, BEFORE velocity
            body.position += body.velocity * self.time_step;

            // integrate velocity, the new velocity violates the constraints, therefore we need to correct it in the solve method
            body.velocity += gravity * self.time_step;
        }
    }*/

    pub fn solve(&mut self) -> WorldSnapshot {
        // when we get here, the new velocity probably violates the constraints
        // TODO: use sparse matrix, and find a better way to write theses equations
        let mut j = DMatrix::zeros(self.constraints.len(), self.bodies.len() * 2);
        for (row, constraint) in j.row_iter_mut().zip(self.constraints.iter()) {
            constraint.build_j_row(&self.bodies, row);
        }

        let jt = j.transpose();
        let inv_mass_matrix = DMatrix::from_fn(self.bodies.len() * 2, self.bodies.len() * 2, |i, j| {
            if i == j {
                self.bodies[i >> 1].inv_mass
            } else {
                0.0
            }
        });

        /*let v = DVector::from_fn(self.bodies.len() * 2, |i, _| {
            if i & 1 == 0 {
                self.bodies[i >> 1].velocity.x
            } else {
                self.bodies[i >> 1].velocity.y
            }
        });*/

        let force = DVector::from_fn(self.bodies.len() * 2, |i, _| {
            if i & 1 == 0 {
                self.gravity.x / self.bodies[i >> 1].inv_mass
            } else {
                self.gravity.y / self.bodies[i >> 1].inv_mass
            }
        });

        let j_w_q2dot = &j * &inv_mass_matrix * &force;

        let j_dot_q_dot = DVector::from_fn(self.constraints.len(), |i, _| {
            self.constraints[i].compute_j_dot_q_dot(&self.bodies)
        });

        let c_dot = DVector::from_fn(self.constraints.len(), |i, _| {
            self.constraints[i].evaluate_c_dot(&self.bodies)
        });

        let c = DVector::from_fn(self.constraints.len(), |i, _| {
            self.constraints[i].evaluate_c(&self.bodies)
        });

        let k = &j * &inv_mass_matrix * &jt;
        let lu= k.lu();
        let b = - j_w_q2dot - j_dot_q_dot - (0.0/self.time_step) * c_dot - (0.0/(self.time_step * self.time_step)) * c;
        let lambda = lu.solve(&b).unwrap();
        let applied_acceleration = (jt * &lambda) + force;

        //integrate velocity and position
        for (i, body) in self.bodies.iter_mut().enumerate() {

            let acceleration = Vector2::new(applied_acceleration[i * 2], applied_acceleration[i * 2 + 1]) * body.inv_mass;
            let temp = body.pos;
            let old_pos = body.last_pos;
            //Verlet integration
            body.pos = body.pos + body.pos - body.last_pos + acceleration * self.time_step * self.time_step;
            body.last_pos = temp;
            body.velocity = (body.pos - old_pos) / (2.0 * self.time_step);
        }

        /*for (i, body) in self.bodies.iter_mut().enumerate() {
            body.velocity.x += correction[i * 2] * body.inv_mass;
            body.velocity.y += correction[i * 2 + 1] * body.inv_mass;
        }*/

        let kinetic_energy = self.bodies.iter().map(|b| 0.5 *  (1.0 / b.inv_mass) * b.velocity.norm_squared()).sum();
        let potential_energy = self.bodies.iter().map(|b| -(1.0 / b.inv_mass) * self.gravity.dot(&b.pos)).sum();

        let r = WorldSnapshot {
            pos: self.bodies.iter().map(|b| b.pos).collect(),
            links: self.constraints.iter().zip(lambda.as_slice()).map(|(c, v)| (c.widget(), *v)).collect(),
            kinetic_energy,
            potential_energy,
            date: self.age,
        };
        self.age += 1;
        r
    }
}

#[derive(Default)]
pub struct WorldSnapshot {
    pub pos: Vec<Vector2<f32>>,
    pub links: Vec<(ConstraintWidget, f32)>,
    pub kinetic_energy: f32,
    pub potential_energy: f32,
    pub date: u32,
}