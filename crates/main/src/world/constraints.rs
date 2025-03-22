use nalgebra::{Dyn, MatrixViewMut, Vector2, U1};
use crate::world::Body;

pub enum ConstraintWidget {
    None,
    Link(usize, usize),
    Anchor(usize, Vector2<f32>),
    Horizontal(f32),
}


pub trait Constraint : Send + Sync {
    fn set_partial_derivative(&self, bodies: &[Body], row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>) -> f32;
    fn widget(&self) -> ConstraintWidget { ConstraintWidget::None }
}

pub struct DistanceConstraint {
    pub body_a: usize,
    pub body_b: usize,
    pub distance: f32,
    pub bias: f32,
}

impl Constraint for DistanceConstraint {
    fn set_partial_derivative(&self, bodies: &[Body], mut row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>) -> f32 {
        let body1 = &bodies[self.body_a];
        let body2 = &bodies[self.body_b];

        let relative = body2.position - body1.position;
        let distance = relative.norm();

        let a_row_pos = self.body_a * 2;
        let b_row_pos = self.body_b * 2;

        row_view[a_row_pos] = relative.x / distance;
        row_view[a_row_pos + 1] = relative.y / distance;
        row_view[b_row_pos] = -relative.x / distance;
        row_view[b_row_pos + 1] = -relative.y / distance;
        self.bias * (distance - self.distance)
    }

    fn widget(&self) -> ConstraintWidget {
        ConstraintWidget::Link(self.body_a, self.body_b)
    }
}

pub struct AnchorConstraint {
    pub body: usize,
    pub anchor: Vector2<f32>,
    pub distance: f32,
    pub bias: f32,
}

impl Constraint for AnchorConstraint {
    fn set_partial_derivative(&self, bodies: &[Body], mut row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>) -> f32 {
        let body = &bodies[self.body];

        let relative = self.anchor - body.position;
        let distance = relative.norm();

        let row_pos = self.body * 2;

        row_view[row_pos] = relative.x / distance;
        row_view[row_pos + 1] = relative.y / distance;
        self.bias * (distance - self.distance)
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
    fn set_partial_derivative(&self, bodies: &[Body], mut row_view: MatrixViewMut<f32, U1, Dyn, U1, Dyn>) -> f32 {
        let body = &bodies[self.body];

        let distance = body.position.y - self.y_position;

        let row_pos = self.body * 2;

        row_view[row_pos + 1] = 1.0;
        -self.bias * distance
    }

    fn widget(&self) -> ConstraintWidget {
        ConstraintWidget::Horizontal(self.y_position)
    }
}