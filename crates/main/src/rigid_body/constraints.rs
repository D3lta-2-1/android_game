use crate::rigid_body::components::{Acceleration, Position, SubjectToPhysic, Velocity};
use hecs::{Entity, View};
use nalgebra::{Dyn, MatrixViewMut, U1, Vector2};

pub enum ConstraintWidget {
    None,
    Link(usize, usize),
    Anchor(usize, Vector2<f32>),
    Plane(Vector2<f32>, f32),
    Pulley(usize, usize, Vector2<f32>, Vector2<f32>),
}

pub trait Constraint: Send + Sync {
    fn build_j_row(
        &self,
        bodies: &View<(&Position, &SubjectToPhysic)>,
        row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>,
    );
    fn compute_j_dot_q_dot(&self, _bodies: &View<(&Position, &Velocity)>) -> f32 {
        0.0
    }
    fn evaluate_c_dot(&self, _bodies: &View<(&Position, &Velocity)>) -> f32 {
        0.0
    }
    fn evaluate_c(&self, _bodies: &View<&Position>) -> f32 {
        0.0
    }
    fn compute_ddot_q_dot_plus_j_dot_q_ddot(
        &self,
        _bodies: &View<(&Position, &Velocity, &Acceleration)>,
    ) -> f32 {
        0.0
    }
    // TODO: find a better way to do this
    fn widget(&self, _convertor: &dyn Fn(Entity) -> usize) -> ConstraintWidget {
        ConstraintWidget::None
    }
}

pub struct DistanceConstraint {
    pub body_a: Entity,
    pub body_b: Entity,
    pub distance: f32,
}

impl Constraint for DistanceConstraint {
    fn build_j_row(
        &self,
        bodies: &View<(&Position, &SubjectToPhysic)>,
        mut row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>,
    ) {
        let Some((pos1, SubjectToPhysic(index_a))) = bodies.get(self.body_a) else {
            return;
        };
        let Some((pos2, SubjectToPhysic(index_b))) = bodies.get(self.body_b) else {
            return;
        };

        let relative = pos1.actual - pos2.actual;
        let distance = relative.norm();

        let a_row_pos = index_a * 2;
        let b_row_pos = index_b * 2;

        row_view[a_row_pos] = relative.x / distance;
        row_view[a_row_pos + 1] = relative.y / distance;
        row_view[b_row_pos] = -relative.x / distance;
        row_view[b_row_pos + 1] = -relative.y / distance;
    }

    fn compute_j_dot_q_dot(&self, bodies: &View<(&Position, &Velocity)>) -> f32 {
        let Some((pos1, vel1)) = bodies.get(self.body_a) else {
            return 0.0;
        };
        let Some((pos2, vel2)) = bodies.get(self.body_b) else {
            return 0.0;
        };

        let x = pos1.x - pos2.x;
        let y = pos1.y - pos2.y;
        let vx = vel1.x - vel2.x;
        let vy = vel1.y - vel2.y;

        (x * vy - y * vx).powi(2) / (x * x + y * y).powf(3.0 / 2.0)
    }

    fn evaluate_c_dot(&self, bodies: &View<(&Position, &Velocity)>) -> f32 {
        let Some((pos1, vel1)) = bodies.get(self.body_a) else {
            return 0.0;
        };
        let Some((pos2, vel2)) = bodies.get(self.body_b) else {
            return 0.0;
        };

        let x = pos1.x - pos2.x;
        let y = pos1.y - pos2.y;
        let vx = vel1.x - vel2.x;
        let vy = vel1.y - vel2.y;
        (x * vx + y * vy) / (x * x + y * y).sqrt()
    }

    fn evaluate_c(&self, bodies: &View<&Position>) -> f32 {
        let Some(pos1) = bodies.get(self.body_a) else {
            return 0.0;
        };
        let Some(pos2) = bodies.get(self.body_b) else {
            return 0.0;
        };

        (pos1.actual - pos2.actual).norm() - self.distance
    }

    fn compute_ddot_q_dot_plus_j_dot_q_ddot(
        &self,
        bodies: &View<(&Position, &Velocity, &Acceleration)>,
    ) -> f32 {
        let Some((pos1, vel1, accel1)) = bodies.get(self.body_a) else {
            return 0.0;
        };
        let Some((pos2, vel2, accel2)) = bodies.get(self.body_b) else {
            return 0.0;
        };

        let vel = vel1.0 - vel2.0;
        let pos = pos1.actual - pos2.actual;
        let accel = accel1.0 - accel2.0;

        let a = 2.0
            * (accel.dot(&vel) * pos.norm_squared() + vel.norm_squared() * vel.dot(&pos)
                - (vel.norm_squared() + pos.dot(&accel)) * (pos.dot(&vel)))
            / pos.norm_squared().powf(3.0 / 2.0);
        let b = ((vel.norm_squared() * pos.norm_squared() - pos.dot(&vel).powi(2))
            * 3.0
            * (pos.dot(&vel)))
            / pos.norm_squared().powf(5.0 / 2.0);
        a - b
    }

    fn widget(&self, convertor: &dyn Fn(Entity) -> usize) -> ConstraintWidget {
        ConstraintWidget::Link(convertor(self.body_a), convertor(self.body_b))
    }
}

pub struct AnchorConstraint {
    pub body: Entity,
    pub anchor: Vector2<f32>,
    pub distance: f32,
}

impl Constraint for AnchorConstraint {
    fn build_j_row(
        &self,
        bodies: &View<(&Position, &SubjectToPhysic)>,
        mut row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>,
    ) {
        let Some((pos, SubjectToPhysic(index))) = bodies.get(self.body) else {
            return;
        };

        let relative = pos.actual - self.anchor;
        let distance = relative.norm();

        let row_pos = index * 2;

        row_view[row_pos] = relative.x / distance;
        row_view[row_pos + 1] = relative.y / distance;
    }

    fn compute_j_dot_q_dot(&self, bodies: &View<(&Position, &Velocity)>) -> f32 {
        let Some((pos, vel)) = bodies.get(self.body) else {
            return 0.0;
        };

        let x = pos.x - self.anchor.x;
        let y = pos.y - self.anchor.y;
        let vx = vel.x;
        let vy = vel.y;
        (x * vy - y * vx).powi(2) / (x * x + y * y).powf(3.0 / 2.0)
    }

    fn evaluate_c_dot(&self, bodies: &View<(&Position, &Velocity)>) -> f32 {
        let Some((pos, vel)) = bodies.get(self.body) else {
            return 0.0;
        };

        let x = pos.x - self.anchor.x;
        let y = pos.y - self.anchor.y;
        let vx = vel.x;
        let vy = vel.y;
        (x * vx + y * vy) / (x * x + y * y).sqrt()
    }

    fn evaluate_c(&self, bodies: &View<&Position>) -> f32 {
        let Some(pos) = bodies.get(self.body) else {
            return 0.0;
        };
        (pos.actual - self.anchor).norm() - self.distance
    }

    fn compute_ddot_q_dot_plus_j_dot_q_ddot(
        &self,
        bodies: &View<(&Position, &Velocity, &Acceleration)>,
    ) -> f32 {
        let Some((pos, vel, accel)) = bodies.get(self.body) else {
            return 0.0;
        };

        let a = 2.0
            * (accel.dot(&vel) * pos.norm_squared() + vel.norm_squared() * vel.dot(&pos)
                - (vel.norm_squared() + pos.dot(&accel)) * (pos.dot(&vel)))
            / pos.norm_squared().powf(3.0 / 2.0);
        let b = ((vel.norm_squared() * pos.norm_squared() - pos.dot(&vel).powi(2))
            * 3.0
            * (pos.dot(&vel)))
            / pos.norm_squared().powf(5.0 / 2.0);
        a - b
    }

    fn widget(&self, convertor: &dyn Fn(Entity) -> usize) -> ConstraintWidget {
        ConstraintWidget::Anchor(convertor(self.body), self.anchor)
    }
}

/// Keep the body collinear with the director
pub struct PlaneConstraint {
    pub body: Entity,
    pub normal: Vector2<f32>,
    origin: f32,
}

impl PlaneConstraint {
    pub fn new(body: Entity, normal: Vector2<f32>, origin: Vector2<f32>) -> Self {
        let normal = normal.normalize();
        Self {
            body,
            normal,
            origin: normal.dot(&origin),
        }
    }
}

impl Constraint for PlaneConstraint {
    fn build_j_row(
        &self,
        bodies: &View<(&Position, &SubjectToPhysic)>,
        mut row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>,
    ) {
        let Some((_, SubjectToPhysic(index))) = bodies.get(self.body) else {
            return;
        };
        let row_pos = index * 2;
        row_view[row_pos] = self.normal.x;
        row_view[row_pos + 1] = self.normal.y
    }

    // j_dot_q_dot is in fact 0...

    fn evaluate_c_dot(&self, bodies: &View<(&Position, &Velocity)>) -> f32 {
        let Some((_, vel)) = bodies.get(self.body) else {
            return 0.0;
        };
        vel.dot(&self.normal)
    }

    fn evaluate_c(&self, bodies: &View<&Position>) -> f32 {
        let Some(pos) = bodies.get(self.body) else {
            return 0.0;
        };
        pos.dot(&self.normal) - self.origin
    }

    fn widget(&self, _convertor: &dyn Fn(Entity) -> usize) -> ConstraintWidget {
        ConstraintWidget::Plane(self.normal, -self.origin)
    }
}

pub struct PulleyConstraint {
    pub body_a: Entity,
    pub body_b: Entity,
    pub anchor_a: Vector2<f32>,
    pub anchor_b: Vector2<f32>,
    pub distance: f32,
}

impl Constraint for PulleyConstraint {
    fn build_j_row(
        &self,
        bodies: &View<(&Position, &SubjectToPhysic)>,
        mut row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>,
    ) {
        let Some((pos_a, SubjectToPhysic(index_a))) = bodies.get(self.body_a) else {
            return;
        };
        let Some((pos_b, SubjectToPhysic(index_b))) = bodies.get(self.body_b) else {
            return;
        };

        let relative_a = pos_a.actual - self.anchor_a;
        let relative_b = pos_b.actual - self.anchor_b;

        let distance_a = relative_a.norm();
        let distance_b = relative_b.norm();

        let a_row_pos = index_a * 2;
        let b_row_pos = index_b * 2;

        row_view[a_row_pos] = relative_a.x / distance_a;
        row_view[a_row_pos + 1] = relative_a.y / distance_a;
        row_view[b_row_pos] = relative_b.x / distance_b;
        row_view[b_row_pos + 1] = relative_b.y / distance_b;
    }

    fn evaluate_c(&self, bodies: &View<&Position>) -> f32 {
        let Some(pos_a) = bodies.get(self.body_a) else {
            return 0.0;
        };
        let Some(pos_b) = bodies.get(self.body_b) else {
            return 0.0;
        };
        (pos_a.actual - self.anchor_a).norm() + (pos_b.actual - self.anchor_b).norm()
            - self.distance
    }

    fn compute_j_dot_q_dot(&self, bodies: &View<(&Position, &Velocity)>) -> f32 {
        let Some((pos_a, vel_a)) = bodies.get(self.body_a) else {
            return 0.0;
        };
        let Some((pos_b, vel_b)) = bodies.get(self.body_b) else {
            return 0.0;
        };

        let xa = pos_a.x - self.anchor_a.x;
        let ya = pos_a.y - self.anchor_a.y;
        let vxa = vel_a.x;
        let vya = vel_a.y;

        let xb = pos_b.x - self.anchor_b.x;
        let yb = pos_b.y - self.anchor_b.y;
        let vxb = vel_b.x;
        let vyb = vel_b.y;

        (xa * vya - ya * vxa).powi(2) / (xa * xa + ya * ya).powf(3.0 / 2.0)
            + (xb * vyb - yb * vxb).powi(2) / (xb * xb + yb * yb).powf(3.0 / 2.0)
    }

    fn widget(&self, convertor: &dyn Fn(Entity) -> usize) -> ConstraintWidget {
        ConstraintWidget::Pulley(
            convertor(self.body_a),
            convertor(self.body_b),
            self.anchor_a,
            self.anchor_b,
        )
    }
}
