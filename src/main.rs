use macroquad::prelude::*;

#[derive(Debug, Clone)]
struct Particles {
    position: Vec<Vec2>,
    last_position: Vec<Vec2>,
    velocity: Vec<Vec2>,
    last_velocity: Vec<Vec2>,
    mass: Vec<f32>,
}
impl Particles {
    fn len(&self) -> usize {
        self.position.len()
    }
}

#[derive(Debug, Clone, Copy)]
struct Constraint {
    colliders: (usize, usize),
    normal: Vec2,
}

#[derive(Debug, Clone, Default)]
struct Constraints {
    data: Vec<Constraint>,
    stiffness: Vec<f32>,
    dual_multiplier: Vec<f32>,
    dual_gradient: Vec<f32>,
}
impl Constraints {
    fn len(&self) -> usize {
        self.data.len()
    }
}

#[macroquad::main("main")]
async fn main() {
    let scaling = 50.0;

    let mut particles = Particles {
        position: vec![
            vec2(0.0, 0.0),
            vec2(5.0, 0.0),
            vec2(6.0, 0.0),
            vec2(7.0, 0.0),
        ],
        last_position: vec![vec2(0.0, 0.0); 4],
        velocity: vec![
            vec2(0.1, 0.0),
            vec2(0.0, 0.0),
            vec2(0.0, 0.0),
            vec2(0.0, 0.0),
        ],
        last_velocity: vec![vec2(0.0, 0.0); 4],
        mass: vec![1.0, 1.0, 1.0, 1.0],
    };

    let constraint_step = 1.0;

    loop {
        let offset = vec2(100.0, 100.0);
        clear_background(BLACK);

        if is_key_pressed(KeyCode::Period) || is_key_down(KeyCode::Space) {
            particles.last_position = particles.position.clone();
            particles.last_velocity = particles.velocity.clone();
            for i in 0..particles.len() {
                particles.position[i] += particles.velocity[i];
            }

            let mut constraints = Constraints::default();

            for i in 0..particles.len() {
                for j in i + 1..particles.len() {
                    let pi = particles.position[i];
                    let pj = particles.position[j];
                    if pi.distance(pj) <= 1.0 {
                        constraints.data.push(Constraint {
                            colliders: (i, j),
                            normal: (pi - pj).normalize(),
                        });
                        constraints.stiffness.push(1.0);
                        constraints.dual_multiplier.push(0.0);
                        constraints.dual_gradient.push(0.0);
                    }
                }
            }
            for _iter in 0..10 {
                // for i in 0..constraints.len() {
                //     constraints.dual_multiplier[i] = 0.0;
                // }
                // Clear dual multipliers?
                let mut preconditioner_diag = vec![0.0; constraints.len()];
                for i in 0..constraints.len() {
                    let Constraint {
                        colliders: (a, b),
                        normal,
                    } = constraints.data[i];
                    let constraint_value =
                        (particles.position[a] - particles.position[b]).dot(normal) - 1.0;
                    constraints.dual_gradient[i] = -constraint_value
                        - 1.0 / constraints.stiffness[i] * constraints.dual_multiplier[i];
                    preconditioner_diag[i] = 1.0
                        / (1.0 / particles.mass[a]
                            + 1.0 / particles.mass[b]
                            + 1.0 / constraints.stiffness[i]);
                }
                let delta_multiplier = (0..constraints.len())
                    .map(|i| {
                        constraint_step * preconditioner_diag[i] * constraints.dual_gradient[i]
                    })
                    .collect::<Vec<_>>();
                for i in 0..constraints.len() {
                    constraints.dual_multiplier[i] += delta_multiplier[i];
                }
                for i in 0..constraints.len() {
                    let Constraint {
                        colliders: (a, b),
                        normal,
                    } = constraints.data[i];
                    particles.velocity[a] += normal * delta_multiplier[i] / particles.mass[a];
                    particles.velocity[b] -= normal * delta_multiplier[i] / particles.mass[b];
                }
                for i in 0..particles.len() {
                    particles.position[i] = particles.velocity[i] + particles.last_position[i];
                }
            }
        }
        for i in 0..particles.len() {
            draw_circle(
                particles.position[i].x * scaling + offset.x,
                particles.position[i].y * scaling + offset.x,
                0.5 * scaling,
                RED,
            );
        }
        next_frame().await
    }
}
