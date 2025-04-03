use nalgebra::{Dyn, MatrixViewMut, Vector2, U1};
use crate::world::Body;

pub enum ConstraintWidget {
    None,
    Link(usize, usize),
    Anchor(usize, Vector2<f32>),
    Horizontal(f32),
}


pub trait Constraint : Send + Sync {
    fn build_j_row(&self, bodies: &[Body], row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>);
    fn compute_j_dot_q_dot(&self, _bodies: &[Body]) -> f32 { 0.0 }
    fn evaluate_c_dot(&self, _bodies: &[Body]) -> f32 { 0.0 }
    fn evaluate_c(&self, _bodies: &[Body]) -> f32 { 0.0 }
    fn widget(&self) -> ConstraintWidget { ConstraintWidget::None }
}

pub struct DistanceConstraint {
    pub body_a: usize,
    pub body_b: usize,
    pub distance: f32,
}

impl Constraint for DistanceConstraint {
    fn build_j_row(&self, bodies: &[Body], mut row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>) {
        let Body{ pos: pos1, velocity: vel1, .. } = &bodies[self.body_a];
        let Body{ pos: pos2, velocity: vel2, .. } = &bodies[self.body_b];

        let relative = pos2 - pos1;
        let distance = relative.norm();

        let a_row_pos = self.body_a * 2;
        let b_row_pos = self.body_b * 2;

        row_view[a_row_pos] = -relative.x / distance;
        row_view[a_row_pos + 1] = -relative.y / distance;
        row_view[b_row_pos] = relative.x / distance;
        row_view[b_row_pos + 1] = relative.y / distance;
    }

    fn compute_j_dot_q_dot(&self, bodies: &[Body]) -> f32 {
        let Body{ pos: pos1, velocity: vel1, .. } = &bodies[self.body_a];
        let Body{ pos: pos2, velocity: vel2, .. } = &bodies[self.body_b];

        let x = pos2.x - pos1.x;
        let y = pos2.y - pos1.y;
        let vx = vel2.x - vel1.x;
        let vy = vel2.y - vel1.y;

        (x * vy - y * vx).powi(2) / (x * x + y * y).powf(3.0/2.0)
    }

    fn evaluate_c_dot(&self, bodies: &[Body]) -> f32 {
        let Body{ pos: pos1, velocity: vel1, .. } = &bodies[self.body_a];
        let Body{ pos: pos2, velocity: vel2, .. } = &bodies[self.body_b];

        let x = pos2.x - pos1.x;
        let y = pos2.y - pos1.y;
        let vx = vel2.x - vel1.x;
        let vy = vel2.y - vel1.y;
        (x * vx + y * vy) / (x * x + y * y).sqrt()
    }

    fn evaluate_c(&self, bodies: &[Body]) -> f32 {
        let Body{ pos: pos1, .. } = &bodies[self.body_a];
        let Body{ pos: pos2, .. } = &bodies[self.body_b];

        let x = pos2.x - pos1.x;
        let y = pos2.y - pos1.y;
        (x * x + y * y).sqrt() - self.distance
    }

    fn widget(&self) -> ConstraintWidget {
        ConstraintWidget::Link(self.body_a, self.body_b)
    }
}

pub struct AnchorConstraint {
    pub body: usize,
    pub anchor: Vector2<f32>,
    pub distance: f32,
}

impl Constraint for AnchorConstraint {
    fn build_j_row(&self, bodies: &[Body], mut row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>) {
        let Body{ pos: pos, .. } = &bodies[self.body];

        let relative = self.anchor - pos;
        let distance = relative.norm();

        let row_pos = self.body * 2;

        row_view[row_pos] = -relative.x / distance;
        row_view[row_pos + 1] = -relative.y / distance;
    }

    fn compute_j_dot_q_dot(&self, bodies: &[Body]) -> f32 {
        let Body{ pos: pos1, velocity: vel1, .. } = &bodies[self.body];

        let x = self.anchor.x - pos1.x;
        let y = self.anchor.y - pos1.y;
        let vx = -vel1.x;
        let vy = -vel1.y;
        (x * vy - y * vx).powi(2) / (x * x + y * y).powf(3.0/2.0)
    }

    fn evaluate_c_dot(&self, bodies: &[Body]) -> f32 {
        let Body{ pos: pos1, velocity: vel1, .. } = &bodies[self.body];

        let x = self.anchor.x - pos1.x;
        let y = self.anchor.y - pos1.y;
        let vx = -vel1.x;
        let vy = -vel1.y;
        (x * vx + y * vy) / (x * x + y * y).sqrt()
    }

    fn evaluate_c(&self, bodies: &[Body]) -> f32 {
        let Body{ pos: pos1, .. } = &bodies[self.body];

        let x = self.anchor.x - pos1.x;
        let y = self.anchor.y - pos1.y;
        (x * x + y * y).sqrt() - self.distance
    }

    fn widget(&self) -> ConstraintWidget {
       ConstraintWidget::Anchor(self.body, self.anchor)
    }
}

pub struct HorizontalRail {
    pub body: usize,
    pub y_position: f32,
    pub bias: f32,
}

impl Constraint for HorizontalRail {
    fn build_j_row(&self, bodies: &[Body], mut row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>) {
        let row_pos = self.body * 2;
        row_view[row_pos + 1] = 1.0;
    }

    fn widget(&self) -> ConstraintWidget {
        ConstraintWidget::Horizontal(self.y_position)
    }
}