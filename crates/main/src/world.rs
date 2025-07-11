mod components;
pub mod constraints;

use crate::world::components::{Acceleration, Mass, Position, SubjectToPhysic, Velocity};
use crate::world::constraints::{
    AnchorConstraint, Constraint, ConstraintWidget, DistanceConstraint, PlaneConstraint,
    PulleyConstraint,
};
use hecs::{Entity, World};
use nalgebra::{DMatrix, DVector, Vector2};
use std::time::{Duration, Instant};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Solver {
    FirstOrder,
    SecondOrder,
    FirstOrderWithPrepass,
    HybridV2,
    HybridV3,
    HybridV3cgm,
    HybridV4,
    Pbd,
    HybridV3Pbd,
}

impl Solver {
    pub const LIST: [Solver; 9] = [
        Solver::FirstOrder,
        Solver::SecondOrder,
        Solver::FirstOrderWithPrepass,
        Solver::HybridV2,
        Solver::HybridV3,
        Solver::HybridV3cgm,
        Solver::HybridV4,
        Solver::Pbd,
        Solver::HybridV3Pbd,
    ];
}
pub struct GameContent {
    pub world: World,
    physic_index_to_entity: Vec<Entity>,
    constraints: Vec<Box<dyn Constraint>>,
    applied_correction: DVector<f32>,
    gravity: Vector2<f32>,
    time_step: f32,
    age: u32,
    calculation_time: Duration,
    violation_mean: f32,
    pub solver: Solver,
}

impl GameContent {
    pub fn empty(time_step: f32) -> Self {
        Self {
            world: World::new(),
            constraints: Vec::new(),
            physic_index_to_entity: Vec::new(),
            gravity: Vector2::new(0.0, -9.81),
            applied_correction: DVector::zeros(0),
            time_step,
            age: 0,
            calculation_time: Duration::from_millis(0),
            violation_mean: 0.0,
            solver: Solver::HybridV3,
        }
    }

    pub fn clear(&mut self) {
        self.world.clear();
        self.physic_index_to_entity.clear();
        self.constraints.clear();
        self.age = 0;
    }

    pub fn add_body(&mut self, pos: Vector2<f32>, velocity: Vector2<f32>, mass: f32) -> Entity {
        self.world.spawn((
            Position {
                actual: pos,
                last_tick: pos - velocity * self.time_step,
            },
            Velocity(velocity),
            Acceleration(Vector2::new(0.0, 0.0)),
            SubjectToPhysic(0), // dummy value, this need to be updated in the build index system
            Mass {
                mass,
                inv_mass: mass.recip(),
            },
        ))
    }

    pub fn add_constraint(&mut self, constraint: impl Constraint + 'static) {
        self.constraints.push(Box::new(constraint));
    }

    pub fn simple(&mut self) {
        self.clear();
        self.gravity = Vector2::new(0.0, -9.81);
        let body = self.add_body(Vector2::new(1.0, 0.0), Vector2::new(0.0, 12.0), 1.0);
        self.add_constraint(AnchorConstraint {
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
        self.add_constraint(DistanceConstraint {
            body_a: body1,
            body_b: body2,
            distance: 1.0,
        });
        self.add_constraint(AnchorConstraint {
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
        self.add_constraint(DistanceConstraint {
            body_a: body1,
            body_b: body2,
            distance: 1.0,
        });
        self.add_constraint(DistanceConstraint {
            body_a: body2,
            body_b: body3,
            distance: 1.0,
        });
        self.add_constraint(AnchorConstraint {
            body: body1,
            anchor: Vector2::new(0.0, 0.0),
            distance: 1.0,
        });
    }

    pub fn rope(&mut self) {
        self.clear();
        self.gravity = Vector2::new(0.0, -9.81);
        let mut last_body = None;
        for i in 0..20 {
            let body = self.add_body(
                Vector2::new(i as f32 * 0.25 - 5.0, 0.0),
                Vector2::new(0.0, 4.0),
                0.1,
            );
            if let Some(last_body) = last_body {
                self.add_constraint(DistanceConstraint {
                    body_a: last_body,
                    body_b: body,
                    distance: 0.25,
                });
            }
            last_body = Some(body);
        }
        if let Some(last_body) = last_body {
            self.add_constraint(AnchorConstraint {
                body: last_body,
                anchor: Vector2::new(0.0, 0.0),
                distance: 0.25,
            });
        }
    }

    pub fn rail(&mut self) {
        self.clear();
        self.gravity = Vector2::new(0.0, -9.81);
        let body1 = self.add_body(Vector2::new(-1.0, 1.0), Vector2::new(-1.0, 1.0), 1.0);
        let body2 = self.add_body(Vector2::new(1.0, 1.0), Vector2::new(0.0, 0.0), 1.0);
        self.add_constraint(DistanceConstraint {
            body_a: body1,
            body_b: body2,
            distance: 2.0,
        });
        let body3 = self.add_body(Vector2::new(0.0, 1.0), Vector2::new(0.0, 0.0), 1.0);
        self.add_constraint(PlaneConstraint::new(
            body1,
            Vector2::new(1.0, 1.0),
            Vector2::new(-1.0, 1.0),
        ));
        self.add_constraint(PlaneConstraint::new(
            body2,
            Vector2::new(-1.0, 1.0),
            Vector2::new(1.0, 1.0),
        ));
        self.add_constraint(DistanceConstraint {
            body_a: body1,
            body_b: body3,
            distance: 1.0,
        })
    }

    pub fn structure(&mut self) {
        self.clear();
        self.gravity = Vector2::new(0.0, -9.81);
        let b1 = self.add_body(Vector2::new(1.0, 0.0), Vector2::new(0.0, 0.0), 1.0);
        let b2 = self.add_body(Vector2::new(1.25, 0.25), Vector2::new(0.0, 0.0), 1.0);
        let b3 = self.add_body(Vector2::new(1.5, 0.0), Vector2::new(0.0, 0.0), 1.0);
        let b4 = self.add_body(Vector2::new(1.25, -0.25), Vector2::new(0.0, 0.0), 1.0);
        let r2 = 2.0f32.sqrt() * 0.25;
        self.add_constraint(AnchorConstraint {
            body: b2,
            anchor: Vector2::new(0.0, 0.5),
            distance: r2,
        });
        self.add_constraint(DistanceConstraint {
            body_a: b1,
            body_b: b2,
            distance: r2,
        });
        self.add_constraint(DistanceConstraint {
            body_a: b2,
            body_b: b3,
            distance: r2,
        });
        self.add_constraint(DistanceConstraint {
            body_a: b3,
            body_b: b4,
            distance: r2,
        });
        self.add_constraint(DistanceConstraint {
            body_a: b4,
            body_b: b1,
            distance: r2,
        });
        self.add_constraint(DistanceConstraint {
            body_a: b1,
            body_b: b3,
            distance: 1.0,
        });
    }

    pub fn pulley(&mut self) {
        self.clear();
        self.gravity = Vector2::new(0.0, -9.81);
        let b1 = self.add_body(Vector2::new(-1.0, 0.0), Vector2::new(5.0, 0.0), 1.0);
        let b2 = self.add_body(Vector2::new(1.0, 0.0), Vector2::new(-1.0, 0.0), 1.0);
        self.add_constraint(PulleyConstraint {
            body_a: b1,
            body_b: b2,
            anchor_a: Vector2::new(-1.0, 1.0),
            anchor_b: Vector2::new(1.0, 1.0),
            distance: 2.0,
        });
    }

    pub fn pulley_and_rail(&mut self) {
        self.clear();
        self.gravity = Vector2::new(0.0, -9.8);
        let b1 = self.add_body(Vector2::new(-5.0, 0.0), Vector2::new(0.0, 0.0), 1.0);
        let b2 = self.add_body(Vector2::new(1.0, 0.0), Vector2::new(5.0, 0.0), 2.0);
        self.add_constraint(PlaneConstraint::new(
            b1,
            Vector2::new(0.0, 1.0),
            Vector2::new(0.0, 0.0),
        ));
        self.add_constraint(PulleyConstraint {
            body_a: b1,
            body_b: b2,
            anchor_a: Vector2::new(-1.0, 3.0),
            anchor_b: Vector2::new(1.0, 1.0),
            distance: 6.0,
        });
    }

    fn add_triangle_strip(&mut self, entities: &[Entity]) {
        let view = self.world.view::<&Position>();

        let distance = |a, b| {
            let a = view.get(a).unwrap();
            let b = view.get(b).unwrap();
            (a.actual - b.actual).norm()
        };

        let mut add_constraint = |a| {
            self.constraints.push(Box::new(a));
        };

        if entities.len() < 2 {
            return;
        };
        add_constraint(DistanceConstraint {
            body_a: entities[0],
            body_b: entities[1],
            distance: distance(entities[0], entities[1]),
        });
        for trio in entities.windows(3) {
            let (duo, [a]) = trio.split_at(2) else {
                return;
            };
            for b in duo {
                add_constraint(DistanceConstraint {
                    body_a: *a,
                    body_b: *b,
                    distance: distance(*a, *b),
                });
            }
        }
    }

    pub fn bridge(&mut self) {
        self.clear();
        self.gravity = Vector2::new(0.0, -9.8);
        let bodies = [
            self.add_body(Vector2::new(-1.5, 0.0), Vector2::new(0.0, 0.0), 1.0),
            self.add_body(Vector2::new(-1.0, 0.0), Vector2::new(0.0, 0.0), 1.0),
            self.add_body(Vector2::new(-1.0, 0.5), Vector2::new(0.0, 0.0), 1.0),
            self.add_body(Vector2::new(-0.5, 0.0), Vector2::new(0.0, 0.0), 1.0),
            self.add_body(Vector2::new(-0.5, 0.5), Vector2::new(0.0, 0.0), 1.0),
            self.add_body(Vector2::new(-0.0, 0.0), Vector2::new(0.0, 0.0), 1.0),
            self.add_body(Vector2::new(-0.0, 0.5), Vector2::new(0.0, 0.0), 1.0),
            self.add_body(Vector2::new(0.5, 0.0), Vector2::new(0.0, 0.0), 1.0),
            self.add_body(Vector2::new(0.5, 0.5), Vector2::new(0.0, 0.0), 1.0),
            self.add_body(Vector2::new(1.0, 0.0), Vector2::new(0.0, 0.0), 1.0),
            self.add_body(Vector2::new(1.0, 0.5), Vector2::new(0.0, 0.0), 1.0),
            self.add_body(Vector2::new(1.5, 0.0), Vector2::new(0.0, 0.0), 1.0),
        ];
        let load = self.add_body(Vector2::new(0.0, 1.0), Vector2::new(0.01, 0.0), 10.0);
        self.add_constraint(DistanceConstraint {
            body_a: bodies[5],
            body_b: load,
            distance: 1.0,
        });
        self.add_triangle_strip(&bodies);
        self.add_constraint(AnchorConstraint {
            body: bodies[0],
            anchor: Vector2::new(-1.5, 1.0),
            distance: 1.0,
        });
        self.add_constraint(AnchorConstraint {
            body: bodies[11],
            anchor: Vector2::new(1.5, 1.0),
            distance: 1.0,
        });
    }

    /// update all indices for all the bodies... this theoretically be lazy, but exact solver are slow anyway
    fn update_solver_index(&mut self) {
        let mut query = self
            .world
            .query::<&mut SubjectToPhysic>()
            .with::<(&Position, &Velocity, &Mass)>();
        let iter = query.into_iter();
        self.physic_index_to_entity.clear();
        self.physic_index_to_entity.reserve(iter.len());
        for (i, (entity, index)) in iter.enumerate() {
            *index = SubjectToPhysic(i);
            self.physic_index_to_entity.push(entity)
        }
        self.applied_correction = DVector::zeros(self.constraints.len());
    }

    /// q_dot is the combined velocity of all bodies
    fn q_dot_vector(&self) -> DVector<f32> {
        let mut query = self.world.query::<&Velocity>();
        let view = query.view();
        let size = self.physic_index_to_entity.len() * 2;
        DVector::from_iterator(
            size,
            self.physic_index_to_entity
                .iter()
                .cloned()
                .flat_map(|e| view.get(e).unwrap().as_slice().iter().cloned()),
        )
    }

    fn j_matrix(&self) -> DMatrix<f32> {
        let mut query = self.world.query::<(&Position, &SubjectToPhysic)>();
        let len = query.iter().len();
        let view = query.view();
        let mut j = DMatrix::zeros(self.constraints.len(), len * 2);
        let row_iter = j.row_iter_mut();
        let constraint_iter = self.constraints.iter();
        for (row, constraint) in row_iter.zip(constraint_iter) {
            constraint.build_j_row(&view, row);
        }
        j
    }

    fn inv_mass_matrix(&self) -> DMatrix<f32> {
        let mut query = self.world.query::<&Mass>();
        let view = query.view();
        let size = self.physic_index_to_entity.len() * 2;
        let mut iter = self.physic_index_to_entity.iter().cloned().flat_map(|e| {
            let mass = view.get(e).unwrap();
            [mass.inv_mass, mass.inv_mass].into_iter()
        });
        DMatrix::from_fn(
            size,
            size,
            |i, j| {
                if i == j { iter.next().unwrap() } else { 0.0 }
            },
        )
    }

    fn force_vector(&self) -> DVector<f32> {
        let mut query = self.world.query::<&Mass>();
        let view = query.view();
        let size = self.physic_index_to_entity.len() * 2;
        let iter = self.physic_index_to_entity.iter().cloned().flat_map(|e| {
            let mass = view.get(e).unwrap();
            let gravity = self.gravity * mass.mass;
            [gravity.x, gravity.y].into_iter()
        });
        DVector::from_iterator(size, iter)
    }

    fn j_dot_q_dot(&self) -> DVector<f32> {
        let mut query = self.world.query::<(&Position, &Velocity)>();
        let view = query.view();
        DVector::from_iterator(
            self.constraints.len(),
            self.constraints
                .iter()
                .map(|c| c.compute_j_dot_q_dot(&view)),
        )
    }

    fn compute_ddot_q_dot_plus_j_dot_q_ddot(&self) -> DVector<f32> {
        for (_, (mass, acceleration)) in self.world.query::<(&Mass, &mut Acceleration)>().iter() {
            acceleration.0 = Vector2::new(0.0, -9.81) * mass.mass;
        }

        let mut query = self.world.query::<(&Position, &Velocity, &Acceleration)>();
        let view = query.view();
        DVector::from_iterator(
            self.constraints.len(),
            self.constraints
                .iter()
                .map(|c| c.compute_ddot_q_dot_plus_j_dot_q_ddot(&view)),
        )
    }

    fn c_dot_vector(&mut self) -> DVector<f32> {
        let mut query = self.world.query::<(&Position, &Velocity)>();
        let view = query.view();
        let len = self.constraints.len();
        DVector::from_iterator(
            len,
            self.constraints.iter().map(|c| c.evaluate_c_dot(&view)),
        )
    }

    fn c_vector(&mut self) -> DVector<f32> {
        let mut query = self.world.query::<&Position>();
        let view = query.view();
        let mut acc = 0.0;
        let len = self.constraints.len();
        let vec = DVector::from_iterator(
            len,
            self.constraints.iter().map(|c| {
                let c = c.evaluate_c(&view);
                acc += c.abs();
                c
            }),
        );
        self.violation_mean = acc / len as f32;
        vec
    }

    pub fn take_snapshot(&mut self) -> WorldSnapshot {
        let mut query = self.world.query::<(&Position, &Velocity, &Mass)>();
        let (kinetic_energy, potential_energy) = query
            .into_iter()
            .map(|(_, (pos, velocity, mass))| {
                let kinetic_energy = 0.5 * (1.0 / mass.inv_mass) * velocity.norm_squared();
                let potential_energy = -mass.mass * self.gravity.dot(pos);
                (kinetic_energy, potential_energy)
            })
            .fold((0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));

        let mut query = self.world.query::<&Position>();
        let view = query.view();
        let pos = self
            .physic_index_to_entity
            .iter()
            .cloned()
            .map(|e| view.get(e).unwrap().actual)
            .collect::<Vec<_>>();

        let mut query = self.world.query::<&SubjectToPhysic>();
        let view = query.view();
        let convertor = |e: Entity| view.get(e).unwrap().0;

        let widget_iter = self.constraints.iter().map(|c| c.widget(&convertor));
        let force_iter = self.applied_correction.iter().cloned();
        let links = widget_iter.zip(force_iter).collect();

        let r = WorldSnapshot {
            pos,
            links, // TODO: FIX constraint link
            kinetic_energy,
            potential_energy,
            date: self.age,
            violation_mean: self.violation_mean,
            calculation_time: self.calculation_time,
        };
        self.age += 1;
        r
    }

    pub fn solve(&mut self) {
        if self.physic_index_to_entity.is_empty() {
            self.update_solver_index()
        }
        let begin = Instant::now();
        match self.solver {
            Solver::FirstOrder => self.solve_1st_order(),
            Solver::SecondOrder => self.solve_2nd_order(),
            Solver::FirstOrderWithPrepass => self.solve_with_prepass(),
            Solver::HybridV2 => self.hybrid_v2(),
            Solver::HybridV3 => self.hybrid_v3(),
            Solver::HybridV3cgm => self.hydrid_v3_cgm(),
            Solver::HybridV4 => self.hybrid_v4(),
            Solver::Pbd => self.pdb(),
            Solver::HybridV3Pbd => self.hybrid_v3_pbd(),
        }
        self.calculation_time = begin.elapsed();
    }

    /// this solver is fine most of the time, but fail we enter a "too wrong" state, mainly where acceleration is too high and needs to be damped
    fn solve_1st_order(&mut self) {
        let inv_mass_matrix = self.inv_mass_matrix();
        let j = self.j_matrix();
        let jt = j.transpose();
        let k = &j * &inv_mass_matrix * &jt;

        for (_, velocity) in self.world.query::<&mut Velocity>().iter() {
            // euler integration
            velocity.0 += self.gravity * self.time_step;
        }

        // this velocity vector tends to violate the constraints
        let q_dot = self.q_dot_vector();

        let c = self.c_vector();

        let cholesky = k.cholesky().unwrap();
        let b = -j * q_dot - (0.0 / self.time_step) * c;
        let lambda = cholesky.solve(&b);
        self.applied_correction = &lambda / self.time_step;
        let applied_momentum = jt * &lambda;

        // correct the velocity
        for (i, (_, (pos, velocity, mass))) in self
            .world
            .query::<(&mut Position, &mut Velocity, &Mass)>()
            .iter()
            .enumerate()
        {
            let momentum = Vector2::new(applied_momentum[i * 2], applied_momentum[i * 2 + 1]);
            velocity.0 += momentum * mass.inv_mass;
            pos.actual += velocity.0 * self.time_step;
        }
    }

    /// this solver tends handle a bit better the case but is unable to maintain high rigidity such as the first order solver
    fn solve_2nd_order(&mut self) {
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

        let cholesky = k.cholesky().unwrap();
        let b = -j_w_q2dot
            - j_dot_q_dot
            - (0.0 / self.time_step) * c_dot
            - (0.0 / (self.time_step * self.time_step)) * c;
        let lambda = cholesky.solve(&b);
        let applied_force = (jt * &lambda) + force;
        self.applied_correction = lambda; // this solver is working with force, so we got what we want

        //integrate velocity and position
        for (i, (_, (pos, velocity, mass))) in self
            .world
            .query::<(&mut Position, &mut Velocity, &Mass)>()
            .iter()
            .enumerate()
        {
            let acceleration =
                Vector2::new(applied_force[i * 2], applied_force[i * 2 + 1]) * mass.inv_mass;
            velocity.0 += acceleration * self.time_step;
            pos.actual +=
                velocity.0 * self.time_step + 0.5 * self.time_step * self.time_step * acceleration;
            // verlet integration is in appropriate here since approximation on the velocity vector is too dirty
        }
    }

    /// So, new guess, I'll preprocess applied forces via a second order solver, and then use the first order solver to correct the velocity and apply Baumgarte stabilization
    fn solve_with_prepass(&mut self) {
        // both solvers work in a similar way, and can share a lot of calculations
        // the most expensive is by far the inversion of the K matrix, which is luckily common to both
        let inv_mass_matrix = self.inv_mass_matrix();
        let j = self.j_matrix();
        let jt = j.transpose();
        let k = &j * &inv_mass_matrix * &jt;
        let cholesky = k.cholesky().unwrap();
        let inv_k = cholesky.inverse();

        // force pass
        let force = self.force_vector();
        let j_w_q2dot = &j * &inv_mass_matrix * &force;
        let j_dot_q_dot = self.j_dot_q_dot();
        let c_dot = self.c_dot_vector();
        let c = self.c_vector();
        let b = -j_w_q2dot
            - j_dot_q_dot
            - (0.0 / self.time_step) * c_dot
            - (0.0 / (self.time_step * self.time_step)) * &c;
        let lambda = &inv_k * b;
        let applied_acceleration = (&jt * &lambda) + force;

        // velocity pass
        for (i, (_, (velocity, mass))) in self
            .world
            .query::<(&mut Velocity, &Mass)>()
            .iter()
            .enumerate()
        {
            let acceleration =
                Vector2::new(applied_acceleration[i * 2], applied_acceleration[i * 2 + 1])
                    * mass.inv_mass;
            // euler integration
            velocity.0 += acceleration * self.time_step;
        }

        // this velocity vector tends to violate the constraints
        let q_dot = self.q_dot_vector();

        let b = -j * q_dot - (0.0 / self.time_step) * c;
        let lambda = inv_k * b;
        let applied_momentum = jt * &lambda;

        // correct the velocity and integrate position
        for (i, (_, (pos, velocity, mass))) in self
            .world
            .query::<(&mut Position, &mut Velocity, &Mass)>()
            .iter()
            .enumerate()
        {
            //let acceleration = Vector2::new(applied_acceleration[i * 2], applied_acceleration[i * 2 + 1]) * body.inv_mass;
            let momentum = Vector2::new(applied_momentum[i * 2], applied_momentum[i * 2 + 1]);
            velocity.0 += momentum * mass.inv_mass;
            let acceleration =
                Vector2::new(applied_acceleration[i * 2], applied_acceleration[i * 2 + 1])
                    * mass.inv_mass;
            // euler integration
            pos.actual +=
                velocity.0 * self.time_step + 0.5 * acceleration * self.time_step * self.time_step;
            // I don't have any clue why 2nd order taylor expansion is **less** accurate than 1st order here
            // maybe because the 1st order solver already provide a better approximation of the velocity
        }
    }

    /// this new attempt change when order are applied
    /// the "quality" of the solver can be measured by "how much the c constraint is far from 0"
    /// Baugmarte stabilization is the reason why there are energy losses, so this attempt try to minimize it usage (before implementation of soft constraint)
    /// first we make the velocity valid is this position, without applying any force
    /// then we apply forces at this location and update the velocity
    /// and we integrate the position
    ///
    /// Taking a step back, wasn't bad, but was poorly set up
    /// In order to get the best accuracy, the new position need to validate constraints
    /// that mean the velocity need to be in the right "direction"
    fn hybrid_v2(&mut self) {
        // both solvers work in a similar way, and can share a lot of calculations
        // the most expensive is by far the inversion of the K matrix, which is luckily common to both
        // the J matrix only depends on the position, so it's common to both because position is updated at the end
        let inv_mass_matrix = self.inv_mass_matrix();
        let j = self.j_matrix();
        let jt = j.transpose();
        let k = &j * &inv_mass_matrix * &jt;
        let cholesky = k.cholesky().unwrap();
        let inv_k = cholesky.inverse();
        let c = self.c_vector();

        // first, make velocity valid in this position
        let q_dot = self.q_dot_vector();

        let b = -&j * q_dot - (0.0 / self.time_step) * &c; //no Baumgarte for now
        let lambda = &inv_k * b;
        let applied_momentum = &jt * &lambda;

        for (i, (_, (velocity, mass))) in self
            .world
            .query::<(&mut Velocity, &Mass)>()
            .iter()
            .enumerate()
        {
            let momentum = Vector2::new(applied_momentum[i * 2], applied_momentum[i * 2 + 1]);
            velocity.0 += momentum * mass.inv_mass;
        }
        // now, we apply forces at this location and update the velocity
        let force = self.force_vector();
        let j_w_q2dot = &j * &inv_mass_matrix * &force;
        let j_dot_q_dot = self.j_dot_q_dot();
        let c_dot = self.c_dot_vector();

        let b = -j_w_q2dot
            - j_dot_q_dot
            - (0.0 / self.time_step) * c_dot
            - (0.0 / (self.time_step * self.time_step)) * c;
        let lambda = inv_k * b;
        let applied_force = (jt * &lambda) + force;

        //integrate velocity and position
        for (i, (_, (pos, velocity, mass))) in self
            .world
            .query::<(&mut Position, &mut Velocity, &Mass)>()
            .iter()
            .enumerate()
        {
            let acceleration =
                Vector2::new(applied_force[i * 2], applied_force[i * 2 + 1]) * mass.inv_mass;
            let temp = pos.actual;
            let old_pos = pos.last_tick;
            //Verlet integration
            pos.actual = pos.actual + pos.actual - pos.last_tick
                + acceleration * self.time_step * self.time_step;
            pos.last_tick = temp;
            velocity.0 = (pos.actual - old_pos) / (2.0 * self.time_step);
        }
    }

    /// I guessed most of this solver by playing around
    /// this is, by far, the most accurate solver, while being one of the fastest
    /// My original idea was to apply the second part of the second order to add a little extra information of acceleration
    /// turn out, I wasn't far from being successful, but I was missing a few things, where do this 1/2 applied on J_dot_q_dot comes from ?
    /// With more investigation, I found that a second order taylor expansion on "the position" gave this
    /// x(t + Dt) = x(t) + v(t) * Dt + 1/2 * a(t) * Dt^2
    /// (x(t + Dt) - x(t)) / Dt = v(t) + 1/2 * a(t) * Dt
    /// Some piece are still missing, but it looks like this solver is doing big part of the integration while working on the velocity
    /// which could explain it being so accurate
    /// My theory on the remaining energy loss, while being very low, is that the new velocity isn't really tangent to the movement
    fn hybrid_v3(&mut self) {
        let inv_mass_matrix = self.inv_mass_matrix();
        let j = self.j_matrix();
        let jt = j.transpose();
        let k = &j * &inv_mass_matrix * &jt;

        for (_, velocity) in self.world.query::<&mut Velocity>().iter() {
            // euler integration
            velocity.0 += self.gravity * self.time_step;
        }

        // this velocity vector tends to violate the constraints
        let q_dot = self.q_dot_vector();
        let j_dot_q_dot = self.j_dot_q_dot();

        let _c = self.c_vector();

        let cholesky = k.cholesky().unwrap();
        let b = -j * q_dot - j_dot_q_dot * self.time_step * 0.5;
        let lambda = cholesky.solve(&b);

        self.applied_correction = &lambda / self.time_step; //since we are working with momentum, we need to divide by the time step to get the applied force

        let applied_momentum = jt * &lambda;

        // correct the velocity
        for (i, (_, (pos, velocity, mass))) in self
            .world
            .query::<(&mut Position, &mut Velocity, &Mass)>()
            .iter()
            .enumerate()
        {
            let momentum = Vector2::new(applied_momentum[i * 2], applied_momentum[i * 2 + 1]);
            velocity.0 += momentum * mass.inv_mass;
            pos.actual += velocity.0 * self.time_step;
        }
    }

    fn hydrid_v3_cgm(&mut self) {
        let inv_mass_matrix = self.inv_mass_matrix();
        let j = self.j_matrix();
        let jt = j.transpose();
        let a = &j * &inv_mass_matrix * &jt;

        for (_, velocity) in self.world.query::<&mut Velocity>().iter() {
            // euler integration
            velocity.0 += self.gravity * self.time_step;
        }

        // this velocity vector tends to violate the constraints
        let q_dot = self.q_dot_vector();
        let j_dot_q_dot = self.j_dot_q_dot();

        let _c = self.c_vector();

        let b = -j * q_dot - j_dot_q_dot * self.time_step * 0.5;

        // conjugate gradient method here
        //let mut lambda = DVector::zeros(self.constraints.len());
        let mut lambda = &self.applied_correction * self.time_step; // it should be a nice guess, at least better than zeros
        {
            let mut r = b - &a * &lambda;
            let mut p = r.clone();
            let mut k = 0;
            loop {
                k += 1;
                let alpha = r.dot(&r) / p.dot(&(&a * &p));
                lambda += alpha * &p;
                let r_dot_r = r.dot(&r);
                r -= alpha * &a * &p;
                if r.norm() < 1e-6 { break; }
                let beta = r.dot(&r) / r_dot_r;
                p = &r + beta * &p;
            }
            println!("CGM iterations: {}", k);
        }

        self.applied_correction = &lambda / self.time_step; //since we are working with momentum, we need to divide by the time step to get the applied force

        let applied_momentum = jt * &lambda;

        // correct the velocity
        for (i, (_, (pos, velocity, mass))) in self.world.query::<(&mut Position, &mut Velocity, &Mass)>().iter().enumerate() {
            let momentum = Vector2::new(applied_momentum[i * 2], applied_momentum[i * 2 + 1]);
            velocity.0 += momentum * mass.inv_mass;
            pos.actual += velocity.0 * self.time_step;
        }
    }

    fn hybrid_v4(&mut self) {
        let inv_mass_matrix = self.inv_mass_matrix();
        let j = self.j_matrix();
        let jt = j.transpose();
        let k = &j * &inv_mass_matrix * &jt;

        for (_, velocity) in self.world.query::<&mut Velocity>().iter() {
            // euler integration
            velocity.0 += self.gravity * self.time_step;
        }

        // this velocity vector tends to violate the constraints
        let q_dot = self.q_dot_vector();
        let j_dot_q_dot = self.j_dot_q_dot();

        let scary_thing = self.compute_ddot_q_dot_plus_j_dot_q_ddot();
        let cholesky = k.cholesky().unwrap();

        let b = -j * q_dot
            - j_dot_q_dot * self.time_step * 0.5
            - (1.0 / 6.0) * self.time_step * self.time_step * scary_thing;
        let lambda = cholesky.solve(&b);
        self.applied_correction = &lambda / self.time_step;
        let applied_momentum = jt * &lambda;

        // correct the velocity
        for (i, (_, (pos, velocity, mass))) in self
            .world
            .query::<(&mut Position, &mut Velocity, &Mass)>()
            .iter()
            .enumerate()
        {
            let momentum = Vector2::new(applied_momentum[i * 2], applied_momentum[i * 2 + 1]);
            velocity.0 += momentum * mass.inv_mass;
            pos.actual += velocity.0 * self.time_step;
        }

        let _c = self.c_vector();
    }

    fn pdb(&mut self) {
        // first step, Verlet integration
        for (_, pos) in self.world.query::<&mut Position>().iter() {
            // euler integration
            let new_pos = 2.0 * pos.actual - pos.last_tick + self.gravity * (self.time_step * self.time_step);
            pos.last_tick = pos.actual;
            pos.actual = new_pos;
        }
        // only first order correction
        let inv_mass_matrix = self.inv_mass_matrix();
        let j = self.j_matrix();
        let jt = j.transpose();
        let k = &j * &inv_mass_matrix * &jt;
        let cholesky = k.cholesky().unwrap();
        let b = -self.c_vector();
        let lambda = cholesky.solve(&b);
        self.applied_correction = &lambda / (self.time_step * self.time_step);
        let applied_integrated_momentum = jt * &lambda;

        for (i, (_, (pos, vel, mass))) in self.world.query::<(&mut Position, &mut Velocity, &Mass)>().iter().enumerate() {
            let integrated_momentum = Vector2::new(applied_integrated_momentum[i * 2], applied_integrated_momentum[i * 2 + 1]);
            pos.actual += integrated_momentum * mass.inv_mass;
            vel.0 = (pos.last_tick - pos.actual) / self.time_step;
        }
        self.c_vector();
    }

    fn hybrid_v3_pbd(&mut self) {
        let inv_mass_matrix = self.inv_mass_matrix();
        let j = self.j_matrix();
        let jt = j.transpose();
        let k = &j * &inv_mass_matrix * &jt;

        for (_, velocity) in self.world.query::<&mut Velocity>().iter() {
            // euler integration
            velocity.0 += self.gravity * self.time_step;
        }

        // this velocity vector tends to violate the constraints
        let q_dot = self.q_dot_vector();
        let j_dot_q_dot = self.j_dot_q_dot();

        let _c = self.c_vector();

        let cholesky = k.cholesky().unwrap();
        let b = -j * q_dot - j_dot_q_dot * self.time_step * 0.5;
        let lambda = cholesky.solve(&b);

        self.applied_correction = &lambda / self.time_step; //since we are working with momentum, we need to divide by the time step to get the applied force

        let applied_momentum = jt * &lambda;

        // correct the velocity
        for (i, (_, (pos, velocity, mass))) in self
            .world
            .query::<(&mut Position, &mut Velocity, &Mass)>()
            .iter()
            .enumerate()
        {
            let momentum = Vector2::new(applied_momentum[i * 2], applied_momentum[i * 2 + 1]);
            velocity.0 += momentum * mass.inv_mass;
            pos.actual += velocity.0 * self.time_step;
        }

        let inv_mass_matrix = self.inv_mass_matrix();
        let j = self.j_matrix();
        let jt = j.transpose();
        let k = &j * &inv_mass_matrix * &jt;
        let cholesky = k.cholesky().unwrap();
        let b = -self.c_vector();
        let lambda = cholesky.solve(&b);
        self.applied_correction += &lambda / (self.time_step * self.time_step);
        let applied_integrated_momentum = jt * &lambda;

        for (i, (_, (pos, mass))) in self.world.query::<(&mut Position, &Mass)>().iter().enumerate() {
            let integrated_momentum = Vector2::new(applied_integrated_momentum[i * 2], applied_integrated_momentum[i * 2 + 1]);
            pos.actual += integrated_momentum * mass.inv_mass;
        }
        self.c_vector();
    }
}

#[derive(Default)]
pub struct WorldSnapshot {
    pub pos: Vec<Vector2<f32>>,
    pub links: Vec<(ConstraintWidget, f32)>,
    pub kinetic_energy: f32,
    pub potential_energy: f32,
    pub date: u32,
    pub calculation_time: Duration,
    pub violation_mean: f32,
}
