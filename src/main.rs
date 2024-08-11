use macroquad::prelude::*;

#[derive(Debug, Clone, Copy)]
struct Particle {
    position: Vec2,
    predicted_position: Vec2,
    displacement: Vec2,
    radius: f32,
    inv_mass: f32,
}

#[macroquad::main("main")]
async fn main() {
    let scaling = 50.0;

    let mut particles = vec![
        Particle {
            position: vec2(10.0, 0.0),
            predicted_position: vec2(10.0, 0.0),
            displacement: vec2(-0.1, 0.0),
            radius: 1.0,
            inv_mass: 0.01,
        },
        Particle {
            position: vec2(0.0, 0.0),
            predicted_position: vec2(0.0, 0.0),
            displacement: vec2(0.0, 0.0),
            radius: 0.5,
            inv_mass: 0.0,
        },
        Particle {
            position: vec2(1.0, 0.0),
            predicted_position: vec2(1.0, 0.0),
            displacement: vec2(0.0, 0.0),
            radius: 0.5,
            inv_mass: 1.0,
        },
    ];
    loop {
        let offset = vec2(100.0, 100.0);
        clear_background(BLACK);

        if is_key_pressed(KeyCode::Period) || is_key_down(KeyCode::Space) {
            for particle in &mut particles {
                let pos = particle.position + particle.displacement;
                particle.position = pos;
                particle.predicted_position = pos + particle.displacement;
            }
            for ix in 0..particles.len() {
                let particle = &particles[ix];
                let pos = particle.predicted_position;
                let mut displacement = vec2(0.0, 0.0);
                for other_particle in &particles {
                    if other_particle.position != particle.position {
                        let other_pos = other_particle.predicted_position;
                        let delta = pos - other_pos;
                        let dist = delta.length();
                        let penetration = particle.radius + other_particle.radius - dist;
                        if penetration > 0.0 {
                            let normal = delta / dist;
                            displacement += normal * penetration * particle.inv_mass
                                / (particle.inv_mass + other_particle.inv_mass);
                        }
                    }
                }
                particles[ix].displacement += displacement;
            }
        }
        for particle in &particles {
            draw_circle(
                particle.position.x * scaling + offset.x,
                particle.position.y * scaling + offset.x,
                particle.radius * scaling,
                RED,
            );
        }
        next_frame().await
    }
}
