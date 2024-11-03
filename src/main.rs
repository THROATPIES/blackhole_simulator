use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};
use rand::Rng;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const PARTICLE_COUNT: usize = 5000;

#[derive(Resource)]
struct WindowSize {
    width: f32,
    height: f32,
}

#[derive(Component)]
struct Particle {
    velocity: Vec2,
    mass: f32,
}

#[derive(Resource)]
struct BlackHole {
    position: Vec2,
    mass: f32,
    event_horizon: f32,
}

#[derive(Resource)]
struct SimulationState {
    paused: bool,
    show_trails: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Black Hole Simulator".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(BlackHole {
            position: Vec2::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0),
            mass: 1000.0,
            event_horizon: 15.0,
        })
        .insert_resource(SimulationState {
            paused: false,
            show_trails: false,
        })
        .insert_resource(WindowSize {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                input_handling,
                update_particles,
                update_black_hole,
                draw_ui,
                handle_window_resize,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, window_size: Res<WindowSize>) {
    commands.spawn(Camera2dBundle::default());

    let mut rng = rand::thread_rng();
    for _ in 0..PARTICLE_COUNT {
        commands.spawn((
            Particle {
                velocity: Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)),
                mass: rng.gen_range(0.1..1.0),
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(
                        rng.gen_range(0.5..1.0),
                        rng.gen_range(0.5..1.0),
                        rng.gen_range(0.5..1.0),
                    ),
                    custom_size: Some(Vec2::splat(2.0)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    rng.gen_range(0.0..window_size.width),
                    rng.gen_range(0.0..window_size.height),
                    0.0,
                ),
                ..default()
            },
        ));
    }
}

fn input_handling(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut simulation_state: ResMut<SimulationState>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        simulation_state.paused = !simulation_state.paused;
    }
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        simulation_state.show_trails = !simulation_state.show_trails;
    }
}

fn update_particles(
    mut particles: Query<(&mut Particle, &mut Transform)>,
    black_hole: Res<BlackHole>,
    simulation_state: Res<SimulationState>,
    window_size: Res<WindowSize>,
    time: Res<Time>,
) {
    if simulation_state.paused {
        return;
    }

    for (mut particle, mut transform) in particles.iter_mut() {
        let direction = black_hole.position - transform.translation.truncate();
        let distance = direction.length();

        if distance < black_hole.event_horizon {
            // Particle is consumed by black hole
            let mut rng = rand::thread_rng();
            transform.translation.x = rng.gen_range(0.0..WINDOW_WIDTH);
            transform.translation.y = rng.gen_range(0.0..WINDOW_HEIGHT);
            particle.velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
        } else {
            let force = (black_hole.mass * particle.mass) / (distance * distance);
            particle.velocity += direction.normalize() * force * time.delta_seconds();
            transform.translation += (particle.velocity * time.delta_seconds()).extend(0.0);

            // Wrap particles around screen edges
            transform.translation.x =
                (transform.translation.x + window_size.width) % window_size.width;
            transform.translation.y =
                (transform.translation.y + window_size.height) % window_size.height;
        }
    }
}

fn update_black_hole(
    mut black_hole: ResMut<BlackHole>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    let window = windows.single();

    if mouse_button.pressed(MouseButton::Left) {
        if let Some(position) = window.cursor_position() {
            black_hole.position = position;
        }
    }

    let mass_change = if keyboard_input.pressed(KeyCode::ArrowUp) {
        10.0
    } else if keyboard_input.pressed(KeyCode::ArrowDown) {
        -10.0
    } else {
        0.0
    };

    black_hole.mass = (black_hole.mass + mass_change).max(100.0);
    black_hole.event_horizon = (black_hole.mass / 1000.0).sqrt() * 15.0;
}

fn draw_ui(mut commands: Commands) {
    commands.spawn(TextBundle::from_section(
        "Left click: Move black hole\nUp/Down arrows: Adjust black hole mass\nSpace: Pause/Resume\nT: Toggle trails",
        TextStyle {
            font_size: 20.0,
            color: Color::WHITE,
            ..default()
        },
    ).with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(10.0),
        left: Val::Px(10.0),
        ..default()
    }));
}

fn handle_window_resize(
    mut window_size: ResMut<WindowSize>,
    mut resize_reader: EventReader<WindowResized>,
    mut black_hole: ResMut<BlackHole>,
    mut particles: Query<&mut Transform, With<Particle>>,
) {
    for event in resize_reader.read() {
        window_size.width = event.width;
        window_size.height = event.height;

        // Update black hole position
        black_hole.position = Vec2::new(window_size.width / 2.0, window_size.height / 2.0);

        // Redistribute particles
        let mut rng = rand::thread_rng();
        for mut transform in particles.iter_mut() {
            transform.translation.x = rng.gen_range(0.0..window_size.width);
            transform.translation.y = rng.gen_range(0.0..window_size.height);
        }
    }
}
