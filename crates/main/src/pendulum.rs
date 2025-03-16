use nalgebra::{Matrix1x2, Matrix2, Vector2};

/// this is my first attempt to implement an impulse solver as described by Erin Catto in 2014

pub struct Body {
    pub position: Vector2<f32>, // m
    velocity: Vector2<f32>, // m/s
    mass: f32, // kg
    inv_mass: f32, // kg^-1
}

impl Body {
    fn get_inv_mass_matrix(&self) -> Matrix2<f32> {
        Matrix2::from_diagonal_element(self.inv_mass)
    }

    fn get_effective_mass(&self, jacobian: Matrix1x2<f32>) -> f32 {
        let inv_mass = self.get_inv_mass_matrix();
        let jacobian_t = jacobian.transpose();

        // I'm confused about the use of the "effective mass" term, here, it simply returns the mass itself, (because the only jacobian is the normalized position vector)

        let result = jacobian * inv_mass * jacobian_t;
        1.0 / result.x
    }
}

/// Keep a body at a fixed distance from the origin
/// C = |p1| - L
/// dC/dt = (p1/|p1|) * v1
struct DistanceConstraint {
    length: f32,
}

impl DistanceConstraint {
    fn get_jacobian(&self, body: &Body) -> Matrix1x2<f32> {
        let p1 = body.position;
        let p1_norm = p1.normalize().transpose();
        p1_norm
    }

    fn constraint(&self, body: &Body) -> f32 {
        let p1 = body.position;
        let length = p1.norm();
        length - self.length
    }
}

pub struct PendulumSystem {
    pub body: Body,
    constraints: DistanceConstraint,
    gravity: Vector2<f32>,
    time_step: f32,
}

impl PendulumSystem {
    pub fn new(time_step: f32) -> Self {
        Self {
            body: Body {
                position: Vector2::new(2.0, -2.0),
                velocity: Vector2::new(0.0, 0.0),
                mass: 1.0,
                inv_mass: 1.0,
            },
            constraints: DistanceConstraint { length: 1.0 },
            gravity: Vector2::new(0.0, -9.8),
            time_step,
        }
    }

    // there is no broad-phase nor narrow-phase collision detection

    pub fn integrate(&mut self) {
        let gravity = self.gravity;
        let body = &mut self.body;

        // integrate position, BEFORE velocity
        body.position += body.velocity * self.time_step;

        // integrate velocity, the new velocity violates the constraints, therefore we need to correct it in the solve method
        body.velocity += gravity * self.time_step;
    }

    pub fn solve(&mut self) {
        // when we get here, the new velocity probably violates the constraints
        let bias = 1.0;
        let jacobian = self.constraints.get_jacobian(&self.body);
        let effective_mass = self.body.get_effective_mass(jacobian);
        let lagrange_multiplier = -effective_mass * ((jacobian * self.body.velocity).x + bias * self.constraints.constraint(&self.body) / self.time_step);
        let impulse = jacobian.transpose() * lagrange_multiplier;

        // then correct the body's velocity with the impulse
        self.body.velocity += impulse * self.body.inv_mass;
    }

}