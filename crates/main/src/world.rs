pub mod constraints;

use std::ops::Deref;
use hecs::{Entity, View, World};
use nalgebra::{DMatrix, DVector, Vector2};
use crate::world::constraints::{AnchorConstraint, Constraint, ConstraintWidget, DistanceConstraint};


#[derive(Debug, Clone, Copy, PartialEq)]
struct Velocity(Vector2<f32>);

impl Deref for Velocity {
    type Target = Vector2<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Position(Vector2<f32>);

impl Deref for Position {
    type Target = Vector2<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Mass {
    mass: f32,
    inv_mass: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Solver {
    FirstOrder,
    SecondOrder,
    FirstOrderWithPrepass,
}

impl Solver {
    pub const LIST: [Solver; 3] = [
        Solver::FirstOrder,
        Solver::SecondOrder,
        Solver::FirstOrderWithPrepass,
    ];
}
pub struct GameContent {
    pub world: World,
    gravity: Vector2<f32>,
    time_step: f32,
    age: u32,
    pub solver: Solver,
}

impl GameContent {
    pub fn empty(time_step: f32) -> Self {
        Self {
            world: World::new(),
            gravity: Vector2::new(0.0, -9.81),
            time_step,
            age: 0,
            solver: Solver::FirstOrderWithPrepass,
        }
    }

    pub fn clear(&mut self) {
        self.world.clear();
        self.age = 0;
    }

    pub fn add_body(&mut self, pos: Vector2<f32>, velocity: Vector2<f32>, inv_mass: f32) -> Entity {
        self.world.spawn((
            Position(pos),
            Velocity(velocity),
            Mass {
                mass: 1.0 / inv_mass,
                inv_mass,
            },
        ))
    }

    pub fn add_constrait(&mut self, constraint: impl Constraint) {
        self.world.spawn(constraint);
    }

    pub fn simple(&mut self) {
        self.clear();
        self.gravity = Vector2::new(0.0, -0.0);
        let body = self.add_body(Vector2::new(1.0, 0.0), Vector2::new(0.0, 9.0), 1.0);
        self.add_constrait(AnchorConstraint {
            body,
            anchor: Vector2::new(0.0, 0.0),
            distance: 1.0,
        });

    }

    pub fn double(&mut self) {
        self.clear();
        self.gravity = Vector2::new(0.0, -9.81);
        let body1 = self.add_body(Vector2::new(1.0, 0.0), Vector2::new(0.0, 0.0), 1.0);
        let body2 = self.add_body(Vector2::new(1.0, 1.0), Vector2::new(-0.0, 0.0), 1.0);
        self.add_constrait(DistanceConstraint {
            body_a: body1,
            body_b: body2,
            distance: 1.0,
        });
        self.add_constrait(AnchorConstraint {
            body: body1,
            anchor: Vector2::new(0.0, 0.0),
            distance: 1.0,
        });
    }

    pub fn triple(&mut self) {
        self.clear();
        self.gravity = Vector2::new(0.0, -9.81);
        let body1 = self.add_body(Vector2::new(1.0, 0.0), Vector2::new(0.0, 0.0), 1.0);
        let body2 = self.add_body(Vector2::new(1.0, 1.0), Vector2::new(-0.0, 0.0), 1.0);
        let body3 = self.add_body(Vector2::new(2.0, 1.0), Vector2::new(-0.0, 0.0), 1.0);
        self.add_constrait(DistanceConstraint {
            body_a: body1,
            body_b: body2,
            distance: 1.0,
        });
        self.add_constrait(DistanceConstraint {
            body_a: body2,
            body_b: body3,
            distance: 1.0,
        });
        self.add_constrait(AnchorConstraint {
            body: body1,
            anchor: Vector2::new(0.0, 0.0),
            distance: 1.0,
        });
    }

    fn j_matrix(&self, view: &View<(&Position, &Velocity, &Mass)>, view_len: usize) -> DMatrix<f32> {
        let anchor_query = self.world.query::<&AnchorConstraint>().iter();
        let distance_query = self.world.query::<&DistanceConstraint>().iter();
        let len = anchor_query.len() + distance_query.len();


        let mut j = DMatrix::zeros(len, view_len * 2);
        let mut row_iter = j.row_iter_mut();

        // we need two loops, but we do not pay the cost of virtual calls
        for (_, constraint) in anchor_query {
            let row = row_iter.next().unwrap();
            constraint.build_j_row(&view, row);
        }
        for (_, constraint) in distance_query {
            let row = row_iter.next().unwrap();
            constraint.build_j_row(&view, row);
        }
        j
    }

    fn inv_mass_matrix(&self) -> DMatrix<f32> {
        DMatrix::from_fn(self.bodies.len() * 2, self.bodies.len() * 2, |i, j| {
            if i == j {
                self.bodies[i >> 1].inv_mass
            } else {
                0.0
            }
        })
    }

    /// q_dot is the combined velocity of all bodies
    fn q_dot_vector(&self) -> DVector<f32> {
        DVector::from_fn(self.bodies.len() * 2, |i, _| {
            if i & 1 == 0 {
                self.bodies[i >> 1].velocity.x
            } else {
                self.bodies[i >> 1].velocity.y
            }
        })
    }

    fn force_vector(&self) -> DVector<f32> {
        DVector::from_fn(self.bodies.len() * 2, |i, _| {
            if i & 1 == 0 {
                self.gravity.x / self.bodies[i >> 1].inv_mass
            } else {
                self.gravity.y / self.bodies[i >> 1].inv_mass
            }
        })
    }

    fn j_dot_q_dot(&self) -> DVector<f32> {
        DVector::from_fn(self.constraints.len(), |i, _| {
            self.constraints[i].compute_j_dot_q_dot(&self.bodies)
        })
    }

    fn c_dot_vector(&self) -> DVector<f32> {
        DVector::from_fn(self.constraints.len(), |i, _| {
            self.constraints[i].evaluate_c_dot(&self.bodies)
        })
    }

    fn c_vector(&self) -> DVector<f32> {
        DVector::from_fn(self.constraints.len(), |i, _| {
            self.constraints[i].evaluate_c(&self.bodies)
        })
    }

    pub fn solve(&mut self) -> WorldSnapshot {

        match self.solver {
            Solver::FirstOrder => self.solve_1st_order(),
            Solver::SecondOrder => self.solve_2nd_order(),
            Solver::FirstOrderWithPrepass => self.solve_with_prepass(),
        }
    }

    /// this solver is fine most of the time, but fail we enter a "too wrong" state, mainly where acceleration is too high and needs to be damped
    fn solve_1st_order(self) -> WorldSnapshot {

        let mut query = self.world.query::<(&Position, &Velocity, &Mass)>();
        let view_len = query.iter().len();
        let view = self.world.query::<(&Position, &Velocity, &Mass)>().view();


        let inv_mass_matrix = self.inv_mass_matrix();
        let j = self.j_matrix();
        let jt = j.transpose();
        let k = &j * &inv_mass_matrix * &jt;

        for body in self.bodies.iter_mut() {
            // euler integration
            body.velocity += self.gravity * self.time_step;
        }

        // this velocity vector tends to violate the constraints
        let q_dot = self.q_dot_vector();

        let c = self.c_vector();

        let lu = k.lu();
        let b = -j * q_dot - (1.0/self.time_step) * c;
        let lambda = lu.solve(&b).unwrap();
        let applied_momentum = jt * &lambda;

        // correct the velocity
        for (i, body) in self.bodies.iter_mut().enumerate() {
            let momentum = Vector2::new(applied_momentum[i * 2], applied_momentum[i * 2 + 1]);
            body.velocity += momentum * body.inv_mass;
            body.pos += body.velocity * self.time_step;
        }

        let kinetic_energy = self.bodies.iter().map(|b| 0.5 * (1.0 / b.inv_mass) * b.velocity.norm_squared()).sum();
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

    /// this solver tends handle a bit better the case but is unable to maintain high rigidity such as the first order solver
    fn solve_2nd_order(&mut self) -> WorldSnapshot {
        // Here we are solving for JWJt * lambda = -J * M^-1 * F - J_dot_q_dot as described by Andrew Witkin
        let inv_mass_matrix = self.inv_mass_matrix();
        let j = self.j_matrix();
        let jt = j.transpose();
        let k = &j * &inv_mass_matrix * &jt;
        let force = self.force_vector();
        let j_w_q2dot = &j * &inv_mass_matrix * &force;
        let j_dot_q_dot = self.j_dot_q_dot();
        let c_dot = self.c_dot_vector();
        let c = self.c_vector();

        let lu= k.lu();
        let b = - j_w_q2dot - j_dot_q_dot - (0.00/self.time_step) * c_dot - (1.0/(self.time_step * self.time_step)) * c;
        let lambda = lu.solve(&b).unwrap();
        let applied_force = (jt * &lambda) + force;

        //integrate velocity and position
        for (i, body) in self.bodies.iter_mut().enumerate() {
            let acceleration = Vector2::new(applied_force[i * 2], applied_force[i * 2 + 1]) * body.inv_mass;
            let temp = body.pos;
            let old_pos = body.last_pos;
            //Verlet integration
            body.pos = body.pos + body.pos - body.last_pos + acceleration * self.time_step * self.time_step;
            body.last_pos = temp;
            body.velocity = (body.pos - old_pos) / (2.0 * self.time_step);
        }

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

    /// So, new guess, I'll preprocess applied forces via a second order solver, and then use the first order solver to correct the velocity and apply Baumgarte stabilization
    fn solve_with_prepass(&mut self) -> WorldSnapshot {
        let view = self.world.query::<(&Position, &Velocity)>().view();


        // both solvers work in a similar way, and can share a lot of calculations
        // the most expensive is by far the inversion of the K matrix, which is luckily common to both
        let inv_mass_matrix = self.inv_mass_matrix();
        let j = self.j_matrix();
        let jt = j.transpose();
        let k = &j * &inv_mass_matrix * &jt;
        let lu = k.lu();
        let inv_k = lu.try_inverse().unwrap();

        // force pass
        let force = self.force_vector();
        let j_w_q2dot = &j * &inv_mass_matrix * &force;
        let j_dot_q_dot = self.j_dot_q_dot();
        let c_dot = self.c_dot_vector();
        let c = self.c_vector();
        let b = - j_w_q2dot - j_dot_q_dot - (1.0/self.time_step) * c_dot - (0.0/(self.time_step * self.time_step)) * &c;
        let lambda = &inv_k * b;
        let applied_acceleration = (&jt * &lambda) + force;

        // velocity pass
        for (i, body) in self.bodies.iter_mut().enumerate() {
            let acceleration = Vector2::new(applied_acceleration[i * 2], applied_acceleration[i * 2 + 1]) * body.inv_mass;
            // euler integration
            body.velocity += acceleration * self.time_step;
        }

        // this velocity vector tends to violate the constraints
        let q_dot = self.q_dot_vector();

        let b = -j * q_dot - (1.0/self.time_step) * c;
        let lambda = inv_k * b;
        let applied_momentum = jt * &lambda;

        // correct the velocity and integrate position
        for (i, body) in self.bodies.iter_mut().enumerate() {
            //let acceleration = Vector2::new(applied_acceleration[i * 2], applied_acceleration[i * 2 + 1]) * body.inv_mass;
            let momentum = Vector2::new(applied_momentum[i * 2], applied_momentum[i * 2 + 1]);
            body.velocity += momentum * body.inv_mass;
            body.pos += body.velocity * self.time_step; //- 0.5 * body.acceleration * self.time_step * self.time_step;
            // I don't have any clue why 2nd order taylor expansion is **less** accurate than 1st order here
            // maybe because the 1st order solver already provide a better approximation of the velocity
        }

        let kinetic_energy = self.bodies.iter().map(|b| 0.5 * (1.0 / b.inv_mass) * b.velocity.norm_squared()).sum();
        let potential_energy = self.bodies.iter().map(|b| - (1.0 / b.inv_mass) * self.gravity.dot(&b.pos)).sum();
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