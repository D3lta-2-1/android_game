use hecs::{Entity, View};
use nalgebra::{Dyn, MatrixViewMut, Vector2, U1};
use crate::world::{Mass, Position, Velocity};

pub enum ConstraintWidget {
    None,
    Link(usize, usize),
    Anchor(usize, Vector2<f32>),
    Horizontal(f32),
}


pub trait Constraint : Send + Sync {
    fn build_j_row(&self, bodies: &View<(&Position, &Velocity, &Mass)>, row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>);
    fn compute_j_dot_q_dot(&self, bodies: &View<(&Position, &Velocity, &Mass)>) -> f32 { 0.0 }
    fn evaluate_c_dot(&self, bodies: &View<(&Position, &Velocity, &Mass)>) -> f32 { 0.0 }
    fn evaluate_c(&self, bodies: &View<(&Position, &Velocity, &Mass)>) -> f32 { 0.0 }
    fn widget(&self) -> ConstraintWidget { ConstraintWidget::None }
}

pub struct DistanceConstraint {
    pub body_a: Entity,
    pub body_b: Entity,
    pub distance: f32,
}

impl Constraint for DistanceConstraint {
    fn build_j_row(&self, bodies: &View<(&Position, &Velocity, &Mass)>, mut row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>) {
        let Some((pos1, _ , _)) = bodies.get(self.body_a) else { return; };
        let Some((pos2, _ , _)) = bodies.get(self.body_b) else { return; };

        let relative = pos1 - pos2;
        let distance = relative.norm();

        let a_row_pos = self.body_a * 2;
        let b_row_pos = self.body_b * 2;

        row_view[a_row_pos] = relative.x / distance;
        row_view[a_row_pos + 1] = relative.y / distance;
        row_view[b_row_pos] = -relative.x / distance;
        row_view[b_row_pos + 1] = -relative.y / distance;
    }

    fn compute_j_dot_q_dot(&self, bodies: &View<(&Position, &Velocity, &Mass)>) -> f32 {
        let Some((pos1, vel1, _)) = bodies.get(self.body_a) else { return 0.0; };
        let Some((pos2, vel2, _)) = bodies.get(self.body_b) else { return 0.0; };

        let x = pos1.x - pos2.x;
        let y = pos1.y - pos2.y;
        let vx = vel1.x - vel2.x;
        let vy = vel1.y - vel2.y;

        (x * vy - y * vx).powi(2) / (x * x + y * y).powf(3.0/2.0)
    }

    fn evaluate_c_dot(&self, bodies: &View<(&Position, &Velocity, &Mass)>) -> f32 {
        let Some((pos1, vel1, _)) = bodies.get(self.body_a) else { return 0.0; };
        let Some((pos2, vel2, _)) = bodies.get(self.body_b) else { return 0.0; };

        let x = pos1.x - pos2.x;
        let y = pos1.y - pos2.y;
        let vx = vel1.x - vel2.x;
        let vy = vel1.y - vel2.y;
        (x * vx + y * vy) / (x * x + y * y).sqrt()
    }

    fn evaluate_c(&self, bodies: &View<(&Position, &Velocity, &Mass)>) -> f32 {
        let Some((pos1, _ , _)) = bodies.get(self.body_a) else { return 0.0; };
        let Some((pos2, _ , _)) = bodies.get(self.body_b) else { return 0.0; };

        let x = pos1.x - pos2.x;
        let y = pos1.y - pos2.y;
        (x * x + y * y).sqrt() - self.distance
    }

    fn widget(&self) -> ConstraintWidget {
        ConstraintWidget::Link(0, 0)
    }
}

pub struct AnchorConstraint {
    pub body: Entity,
    pub anchor: Vector2<f32>,
    pub distance: f32,
}

impl Constraint for AnchorConstraint {
    fn build_j_row(&self, bodies: &View<(&Position, &Velocity, &Mass)>, mut row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>) {
        let Some((pos, _, _)) = bodies.get(self.body) else { return; };

        let relative = pos - self.anchor;
        let distance = relative.norm();

        let row_pos = self.body * 2;

        row_view[row_pos] = relative.x / distance;
        row_view[row_pos + 1] = relative.y / distance;
    }

    fn compute_j_dot_q_dot(&self, bodies: &View<(&Position, &Velocity, &Mass)>) -> f32 {
        let Some((pos, vel, _)) = bodies.get(self.body) else { return 0.0; };

        let x = pos.x - self.anchor.x;
        let y = pos.y - self.anchor.y;
        let vx = vel.x;
        let vy = vel.y;
        (x * vy - y * vx).powi(2) / (x * x + y * y).powf(3.0/2.0)
    }

    fn evaluate_c_dot(&self, bodies :&View<(&Position, &Velocity, &Mass)>) -> f32 {
        let Some((pos, vel, _)) = bodies.get(self.body) else { return 0.0; };

        let x = pos.x - self.anchor.x;
        let y = pos.y - self.anchor.y;
        let vx = vel.x;
        let vy = vel.y;
        (x * vx + y * vy) / (x * x + y * y).sqrt()
    }

    fn evaluate_c(&self, bodies: &View<(&Position, &Velocity, &Mass)>) -> f32 {
        let Some((pos, _, _)) = bodies.get(self.body) else { return 0.0; };

        let x =  pos.x - self.anchor.x;
        let y = pos.y - self.anchor.y;
        (x * x + y * y).sqrt() - self.distance
    }

    fn widget(&self) -> ConstraintWidget {
       ConstraintWidget::Anchor(0, self.anchor)
    }
}