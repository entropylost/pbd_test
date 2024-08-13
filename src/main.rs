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
struct Contact {
    colliders: (usize, usize),
    normal: Vec2,
    stiffness: f32,
}

#[derive(Debug, Clone, Default)]
struct Constraints {
    data: Vec<Contact>,
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
            vec2(4.0, 0.0),
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

    let constraint_step = 0.1;
    let primal = false;

    loop {
        let offset = vec2(100.0, 100.0);
        clear_background(BLACK);

        if is_key_pressed(KeyCode::Period) || is_key_down(KeyCode::Space) {
            particles.last_position = particles.position.clone();
            particles.last_velocity = particles.velocity.clone();
            for i in 0..particles.len() {
                particles.position[i] += particles.velocity[i];
            }

            let mut contacts = vec![];

            for i in 0..particles.len() {
                for j in i + 1..particles.len() {
                    let pi = particles.position[i];
                    let pj = particles.position[j];
                    if pi.distance(pj) <= 1.0 {
                        contacts.push(Contact {
                            colliders: (i, j),
                            normal: (pi - pj).normalize(),
                            stiffness: 10000.0,
                        });
                    }
                }
            }
            if primal {
                for _iter in 0..100 {
                    let mut forces = vec![Vec2::ZERO; particles.len()];
                    let mut jacobian_diag = vec![Vec2::ZERO; particles.len()];

                    for &Contact {
                        colliders: (i, j),
                        normal,
                        stiffness,
                    } in &contacts
                    {
                        let pi = particles.position[i];
                        let pj = particles.position[j];
                        let constraint = (pi - pj).dot(normal) - 1.0;
                        let force = normal * stiffness * constraint.min(0.0);
                        forces[i] -= force;
                        forces[j] += force;
                        jacobian_diag[i] += normal * normal * stiffness;
                        jacobian_diag[j] += normal * normal * stiffness;
                    }
                    let mut preconditioner_diag = vec![Vec2::ZERO; particles.len()];
                    for i in 0..particles.len() {
                        preconditioner_diag[i] = 1.0 / (particles.mass[i] + jacobian_diag[i])
                    }
                    let gradient = (0..particles.len())
                        .map(|i| {
                            particles.mass[i] * (particles.velocity[i] - particles.last_velocity[i])
                                - forces[i]
                        })
                        .collect::<Vec<_>>();
                    for i in 0..particles.len() {
                        particles.velocity[i] -=
                            constraint_step * preconditioner_diag[i] * gradient[i];
                    }
                    for i in 0..particles.len() {
                        particles.position[i] = particles.velocity[i] + particles.last_position[i];
                    }
                }
            } else {
                let mut dual_multiplier = vec![0.0; contacts.len()];
                for _iter in 0..100 {
                    let mut preconditioner_diag = vec![0.0; contacts.len()];
                    let mut dual_gradient = vec![0.0; contacts.len()];
                    for i in 0..contacts.len() {
                        let Contact {
                            colliders: (a, b),
                            normal,
                            stiffness,
                        } = contacts[i];
                        let constraint =
                            (particles.position[a] - particles.position[b]).dot(normal) - 1.0;
                        // let constraint = constraint.min(0.0); // Shouldn't be here probably.
                        dual_gradient[i] = -constraint - 1.0 / stiffness * dual_multiplier[i];
                        preconditioner_diag[i] = 1.0
                            / (1.0 / particles.mass[a] + 1.0 / particles.mass[b] + 1.0 / stiffness);
                    }
                    let mut delta_multiplier = vec![0.0; contacts.len()];
                    for i in 0..contacts.len() {
                        let dm = dual_multiplier[i];
                        dual_multiplier[i] +=
                            constraint_step * preconditioner_diag[i] * dual_gradient[i];
                        dual_multiplier[i] = dual_multiplier[i].max(0.0); // ?
                        delta_multiplier[i] = dual_multiplier[i] - dm;
                    }
                    for i in 0..contacts.len() {
                        let Contact {
                            colliders: (a, b),
                            normal,
                            ..
                        } = contacts[i];
                        particles.velocity[a] += normal * delta_multiplier[i] / particles.mass[a];
                        particles.velocity[b] -= normal * delta_multiplier[i] / particles.mass[b];
                    }
                    for i in 0..particles.len() {
                        particles.position[i] = particles.velocity[i] + particles.last_position[i];
                    }
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
