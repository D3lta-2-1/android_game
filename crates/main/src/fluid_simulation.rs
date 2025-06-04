use crate::fluid_simulation::grid::SquareMask;
use nalgebra::Vector2;

mod grid;

pub struct FluidSimulation {
    densities: Vec<f32>,
    //pressures: Vec<f32>,
    velocities: Vec<Vector2<f32>>,
    is_wall: Vec<bool>,
    access_mask: SquareMask,
    cell_size: f32,
    time_step: f32,
}

impl FluidSimulation {
    pub fn new(size: usize, time_step: f32) -> Self {
        let access_mask = SquareMask::new(size);
        Self {
            densities: access_mask.create_grid(Default::default()),
            //pressures: access_mask.create_grid(Default::default()),
            velocities: access_mask.create_grid(Default::default()),
            is_wall: access_mask.create_grid(Default::default()),
            access_mask,
            cell_size: 0.1, // Default cell size, can be adjusted
            time_step,
        }
    }

    const NEIGHBOURS: [Vector2<isize>; 4] = [
        Vector2::new(1, 0),  // Right
        Vector2::new(0, 1),  // Down
        Vector2::new(-1, 0), // Left
        Vector2::new(0, -1), // Up
    ];

    pub fn divergence_step(&mut self) {
        for i in self.access_mask.index_range() {
            let neighbours = Self::NEIGHBOURS
                .iter()
                .map(|neighbour| {
                    self.access_mask.get(
                        &self.is_wall,
                        self.access_mask.index_to_pos(i) + neighbour,
                        true,
                    )
                })
                .map(|b| if b { 0i32 } else { 1i32 })
                .sum::<i32>() as f32;

            let mut divergence = 0.0;
            let vel = self.velocities[i];
            divergence -= vel.x;
            divergence -= vel.y;
            divergence += self
                .access_mask
                .get(
                    &self.velocities,
                    self.access_mask.index_to_pos(i) + Vector2::new(1, 0),
                    Vector2::new(0.0, 0.0),
                )
                .x;
            divergence += self
                .access_mask
                .get(
                    &self.velocities,
                    self.access_mask.index_to_pos(i) + Vector2::new(0, 1),
                    Vector2::new(0.0, 0.0),
                )
                .y;
            self.velocities[i] += Vector2::new(divergence / neighbours, divergence / neighbours);
            self.access_mask.set(
                &mut self.velocities,
                self.access_mask.index_to_pos(i) + Vector2::new(1, 0),
                |vel| {
                    vel.x -= divergence / neighbours;
                },
            );
            self.access_mask.set(
                &mut self.velocities,
                self.access_mask.index_to_pos(i) + Vector2::new(0, 1),
                |vel| {
                    vel.y -= divergence / neighbours;
                },
            );
        }
    }

    pub fn interpolate_vec(&self, grid: &[Vector2<f32>], pos: Vector2<f32>) -> Vector2<f32> {
        let tx = pos.x.fract();
        let ty = pos.y.fract();
        let pos = Vector2::new(pos.x.floor() as isize, pos.y.floor() as isize);
        let v00 = self.access_mask.get(grid, pos, Vector2::new(0.0, 0.0));
        let v10 = self
            .access_mask
            .get(grid, pos + Vector2::new(1, 0), Vector2::new(0.0, 0.0));
        let v01 = self
            .access_mask
            .get(grid, pos + Vector2::new(0, 1), Vector2::new(0.0, 0.0));
        let v11 = self
            .access_mask
            .get(grid, pos + Vector2::new(1, 1), Vector2::new(0.0, 0.0));
        (1.0 - ty) * (v00 * (1.0 - tx) + v10 * tx) + ty * (v01 * (1.0 - tx) + v11 * tx)
    }

    pub fn interpolate(&self, grid: &[f32], pos: Vector2<f32>) -> f32 {
        let tx = pos.x.fract();
        let ty = pos.y.fract();
        let pos = Vector2::new(pos.x.floor() as isize, pos.y.floor() as isize);
        let v00 = self.access_mask.get(grid, pos, 0.0);
        let v10 = self.access_mask.get(grid, pos + Vector2::new(1, 0), 0.0);
        let v01 = self.access_mask.get(grid, pos + Vector2::new(0, 1), 0.0);
        let v11 = self.access_mask.get(grid, pos + Vector2::new(1, 1), 0.0);
        (1.0 - ty) * (v00 * (1.0 - tx) + v10 * tx) + ty * (v01 * (1.0 - tx) + v11 * tx)
    }

    pub fn advect(&mut self) {
        let old_velocities = self.velocities.clone();
        let old_densities = self.densities.clone();
        for i in self.access_mask.index_range() {
            let pos = self.access_mask.index_to_pos(i);
            let float_pos = Vector2::new(pos.x as f32, pos.y as f32);
            let vel = old_velocities[i];
            let backtracked_vel = self.interpolate_vec(
                &old_velocities,
                float_pos - vel * self.time_step * self.cell_size,
            );
            let backtracked_density =
                self.interpolate(&old_densities, float_pos - vel * self.time_step);
            self.velocities[i] = backtracked_vel;
            self.densities[i] = backtracked_density
        }
    }

    pub fn tick(&mut self) {
        self.access_mask
            .set(&mut self.velocities, Vector2::new(0, 0), |vel| {
                *vel = Vector2::new(1.0, 0.0); // Reset velocities at the origin
            });
        self.access_mask
            .set(&mut self.densities, Vector2::new(0, 0), |density| {
                *density = 1.0; // Reset densities at the origin
            });

        for _ in 0..20 {
            self.divergence_step();
        }
        self.advect();

        // Implement the simulation step logic here
    }

    pub fn take_snapshot(&self) -> FluidSnapshot {
        FluidSnapshot {
            access_mask: self.access_mask.clone(),
            cell_size: self.cell_size,
            densities: self.densities.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FluidSnapshot {
    pub access_mask: SquareMask,
    pub cell_size: f32,
    pub densities: Vec<f32>,
}
