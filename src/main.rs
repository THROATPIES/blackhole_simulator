use macroquad::prelude::*;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const PARTICLE_COUNT: usize = 5000;

struct Particle {
    position: Vec2,
    velocity: Vec2,
    color: Color,
    mass: f32,
}

struct BlackHole {
    position: Vec2,
    mass: f32,
    event_horizon: f32,
}

#[macroquad::main("Black Hole Simulator")]
async fn main() {
    let mut particles = vec![];
    let mut black_hole = BlackHole {
        position: Vec2::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0),
        mass: 1000.0,
        event_horizon: 15.0,
    };
    let mut paused = false;
    let mut show_trails = false;
    let mut trail_frame = Image::gen_image_color(WINDOW_WIDTH as u16, WINDOW_HEIGHT as u16, Color::new(0.1, 0.1, 0.1, 0.1));
    let mut trail_texture = Texture2D::from_image(&trail_frame);

    for _ in 0..PARTICLE_COUNT {
        particles.push(Particle {
            position: Vec2::new(rand::gen_range(0.0, WINDOW_WIDTH), rand::gen_range(0.0, WINDOW_HEIGHT)),
            velocity: Vec2::new(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)),
            color: Color::new(rand::gen_range(0.5, 1.0), rand::gen_range(0.5, 1.0), rand::gen_range(0.5, 1.0), 1.0),
            mass: rand::gen_range(0.1, 1.0),
        });
    }

    loop {
        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::T) {
            show_trails = !show_trails;
        }

        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));

        if show_trails {
            draw_texture(&trail_texture, 0.0, 0.0, WHITE);
        }

        // Update black hole position and mass
        if is_mouse_button_down(MouseButton::Left) {
            black_hole.position = Vec2::from(mouse_position());
        }
        black_hole.mass = black_hole.mass + (is_key_down(KeyCode::Up) as i32 - is_key_down(KeyCode::Down) as i32) as f32 * 10.0;
        black_hole.mass = black_hole.mass.max(100.0);
        black_hole.event_horizon = (black_hole.mass / 1000.0).sqrt() * 15.0;

        if !paused {
            // Update particles
            for particle in &mut particles {
                let direction = black_hole.position - particle.position;
                let distance = direction.length();

                if distance < black_hole.event_horizon {
                    // Particle is consumed by black hole
                    particle.position = Vec2::new(rand::gen_range(0.0, WINDOW_WIDTH), rand::gen_range(0.0, WINDOW_HEIGHT));
                    particle.velocity = Vec2::new(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0));
                } else {
                    let force = (black_hole.mass * particle.mass) / (distance * distance);
                    particle.velocity += direction.normalize() * force * get_frame_time();
                    particle.position += particle.velocity * get_frame_time();

                    // Wrap particles around screen edges
                    particle.position.x = (particle.position.x + WINDOW_WIDTH) % WINDOW_WIDTH;
                    particle.position.y = (particle.position.y + WINDOW_HEIGHT) % WINDOW_HEIGHT;
                }

                // Draw particle
                draw_circle(particle.position.x, particle.position.y, particle.mass, particle.color);

                if show_trails {
                    trail_frame.set_pixel(
                        particle.position.x as u32,
                        particle.position.y as u32,
                        Color::new(particle.color.r, particle.color.g, particle.color.b, 0.1),
                    );
                }
            }

            if show_trails {
                trail_texture.update(&trail_frame);
            }
        }

        // Draw black hole
        draw_circle(black_hole.position.x, black_hole.position.y, black_hole.event_horizon, BLACK);
        draw_circle_lines(black_hole.position.x, black_hole.position.y, black_hole.event_horizon + 2.0, 2.0, WHITE);

        // Draw UI
        draw_text("Left click: Move black hole", 10.0, 20.0, 20.0, WHITE);
        draw_text("Up/Down arrows: Adjust black hole mass", 10.0, 40.0, 20.0, WHITE);
        draw_text("Space: Pause/Resume", 10.0, 60.0, 20.0, WHITE);
        draw_text("T: Toggle trails", 10.0, 80.0, 20.0, WHITE);
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 100.0, 20.0, WHITE);
        draw_text(&format!("Black Hole Mass: {:.0}", black_hole.mass), 10.0, 120.0, 20.0, WHITE);
        draw_text(&format!("Paused: {}", paused), 10.0, 140.0, 20.0, WHITE);
        draw_text(&format!("Trails: {}", show_trails), 10.0, 160.0, 20.0, WHITE);

        next_frame().await
    }
}