use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::WindowResized;
use rand::Rng;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const PARTICLE_COUNT: usize = 100;

#[derive(Component)]
struct Particle {
    velocity: Vec2,
    mass: f32,
}

#[derive(Component)]
struct BlackHole {
    mass: f32,
    event_horizon: f32,
}

#[derive(Resource)]
struct SimulationState {
    paused: bool,
    selected_black_hole: usize,
    particle_size: f32,
    time_scale: f32,
}

#[derive(Component)]
struct GravitationalWave {
    lifetime: Timer,
    intensity: f32,
}

fn spawn_gravitational_wave(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec3,
    intensity: f32,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::new(intensity * 10.0)).into(),
            material: materials.add(ColorMaterial::from(Color::srgba(0.5, 0.5, 1.0, 0.5))),
            transform: Transform::from_translation(position),
            ..default()
        },
        GravitationalWave {
            lifetime: Timer::from_seconds(2.0, TimerMode::Once),
            intensity,
        },
    ));
}

fn update_gravitational_waves(
    mut commands: Commands,
    mut waves: Query<(
        Entity,
        &mut GravitationalWave,
        &mut Transform,
        &Handle<ColorMaterial>,
    )>,
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut wave, mut transform, material_handle) in waves.iter_mut() {
        wave.lifetime.tick(time.delta());
        if wave.lifetime.finished() {
            commands.entity(entity).despawn();
        } else {
            let scale = 1.0 + wave.lifetime.fraction() * 5.0;
            transform.scale = Vec3::splat(scale);
            let alpha = 1.0 - wave.lifetime.fraction();

            if let Some(material) = materials.get_mut(material_handle) {
                material.color.set_alpha(alpha);
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Black Hole Simulator".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(SimulationState {
            paused: false,
            selected_black_hole: 0,
            particle_size: 1.0,
            time_scale: 1.0,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_particles,
                update_black_holes,
                handle_input,
                update_ui,
                handle_window_resize,
                update_gravitational_waves,
                merge_black_holes,
                update_particle_color,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let width = window.width();
    let height = window.height();

    commands.spawn(Camera2dBundle::default());

    let mut rng = rand::thread_rng();

    // Spawn particles
    for _ in 0..PARTICLE_COUNT {
        let position = Vec2::new(
            rng.gen_range(0.0..width),
            height - rng.gen_range(0.0..height), // Invert Y
        );
        let velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
        let color = Color::srgb(
            rng.gen_range(0.5..1.0),
            rng.gen_range(0.5..1.0),
            rng.gen_range(0.5..1.0),
        );
        let mass = rng.gen_range(0.1..1.0);

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::new(mass)).into(),
                material: materials.add(ColorMaterial::from(color)),
                transform: Transform::from_translation(position.extend(0.0)),
                ..default()
            },
            Particle { velocity, mass },
        ));
    }

    // Spawn initial black hole
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::new(15.0)).into(),
            material: materials.add(ColorMaterial::from(Color::BLACK)),
            transform: Transform::from_translation(Vec3::new(width / 2.0, height / 2.0, 0.0)),
            ..default()
        },
        BlackHole {
            mass: 1000.0,
            event_horizon: 15.0,
        },
    ));
}

fn handle_window_resize(
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for event in resize_events.read() {
        // Update camera
        if let Ok(mut transform) = query.get_single_mut() {
            transform.translation.x = event.width / 2.0;
            transform.translation.y = event.height / 2.0;
        }

        // You can add more logic here to adjust other elements based on the new window size
    }
}

fn update_particle_color(
    mut particles: Query<(&Particle, &mut Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (particle, mut material_handle) in particles.iter_mut() {
        let speed = particle.velocity.length();
        let hue = (speed * 50.0) % 360.0; // Adjust this multiplier to change color variation
        let color = Color::hsl(hue, 1.0, 0.5);

        if let Some(material) = materials.get_mut(&*material_handle) {
            material.color = color;
        } else {
            *material_handle = materials.add(ColorMaterial::from(color));
        }
    }
}

fn update_particles(
    mut param_set: ParamSet<(
        Query<(&mut Transform, &mut Particle)>,
        Query<(&Transform, &BlackHole)>,
    )>,
    time: Res<Time>,
    simulation_state: Res<SimulationState>,
    windows: Query<&Window>,
) {
    if simulation_state.paused {
        return;
    }

    let window = windows.single();
    let width = window.width();
    let height = window.height();

    // Apply time scale to delta time
    let scaled_delta_time = time.delta_seconds() * simulation_state.time_scale;

    let black_holes: Vec<(Vec3, f32, f32)> = param_set
        .p1()
        .iter()
        .map(|(transform, black_hole)| {
            (
                transform.translation,
                black_hole.mass,
                black_hole.event_horizon,
            )
        })
        .collect();

    for (mut transform, mut particle) in param_set.p0().iter_mut() {
        for &(black_hole_position, black_hole_mass, event_horizon) in &black_holes {
            let direction = black_hole_position - transform.translation;
            let distance = direction.length();

            if distance < event_horizon {
                // Particle is consumed by black hole
                transform.translation = Vec3::new(
                    rand::random::<f32>() * width,
                    rand::random::<f32>() * height,
                    0.0,
                );
                particle.velocity = Vec2::new(
                    rand::random::<f32>() * 2.0 - 1.0,
                    rand::random::<f32>() * 2.0 - 1.0,
                );
            } else {
                let force = (black_hole_mass * particle.mass) / (distance * distance);
                // Use scaled_delta_time here
                particle.velocity += direction.normalize().truncate() * force * scaled_delta_time;
            }
        }

        // Use scaled_delta_time here as well
        transform.translation += particle.velocity.extend(0.0) * scaled_delta_time;

        // Wrap particles around screen edges
        transform.translation.x = (transform.translation.x + width) % width;
        transform.translation.y = (transform.translation.y + height) % height;
    }
}

fn merge_black_holes(
    mut commands: Commands,
    black_holes: Query<(Entity, &mut Transform, &mut BlackHole)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut to_merge = Vec::new();
    let black_hole_data: Vec<(Entity, Vec3, f32)> = black_holes
        .iter()
        .map(|(entity, transform, black_hole)| (entity, transform.translation, black_hole.mass))
        .collect();

    for i in 0..black_hole_data.len() {
        for j in i + 1..black_hole_data.len() {
            let (entity1, pos1, mass1) = black_hole_data[i];
            let (entity2, pos2, mass2) = black_hole_data[j];
            let distance = pos1.distance(pos2);

            if distance < 30.0 {
                // Adjust this threshold as needed
                to_merge.push((entity1, entity2, (pos1 + pos2) / 2.0, mass1 + mass2));
            }
        }
    }

    for (entity1, entity2, new_pos, new_mass) in to_merge {
        commands.entity(entity1).despawn();
        commands.entity(entity2).despawn();

        let new_event_horizon = (new_mass / 1000.0).sqrt() * 15.0;
        let new_size = new_event_horizon * 2.0;

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::new(new_size / 2.0)).into(),
                material: materials.add(ColorMaterial::from(Color::BLACK)),
                transform: Transform::from_translation(new_pos).with_scale(Vec3::splat(new_size)),
                ..default()
            },
            BlackHole {
                mass: new_mass,
                event_horizon: new_event_horizon,
            },
        ));

        spawn_gravitational_wave(
            &mut commands,
            &mut meshes,
            &mut materials,
            new_pos,
            new_mass / 1000.0,
        );
    }
}

fn update_black_holes(
    mut black_holes: Query<(&mut Transform, &mut BlackHole)>,
    simulation_state: Res<SimulationState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    if let Some(cursor_position) = window.cursor_position() {
        for (i, (mut transform, mut black_hole)) in black_holes.iter_mut().enumerate() {
            if i == simulation_state.selected_black_hole {
                if mouse_input.pressed(MouseButton::Left) {
                    // Invert the y-coordinate
                    let inverted_y = window.height() - cursor_position.y;
                    transform.translation = Vec3::new(cursor_position.x, inverted_y, 0.0);
                }

                if keyboard_input.pressed(KeyCode::ArrowUp) {
                    black_hole.mass += 1.0;
                }
                if keyboard_input.pressed(KeyCode::ArrowDown) {
                    black_hole.mass = (black_hole.mass - 10.0).max(1.0);
                }

                black_hole.event_horizon = (black_hole.mass / 1000.0).sqrt() * 15.0;

                // Update the black hole's size based on its mass
                let size = black_hole.event_horizon * 2.0; // Diameter
                transform.scale = Vec3::new(size, size, 1.0);
            }
        }
    }
}

fn handle_input(
    mut commands: Commands,
    mut simulation_state: ResMut<SimulationState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    black_holes: Query<Entity, With<BlackHole>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Query<&Window>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        simulation_state.paused = !simulation_state.paused;
    }

    if keyboard_input.just_pressed(KeyCode::KeyN) {
        let window = windows.single();
        let initial_mass = 1000.0;
        let initial_event_horizon = ((initial_mass / 1000.0) as f32).sqrt() * 15.0;
        let initial_size = initial_event_horizon * 2.0;
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::new(initial_size / 2.0)).into(),
                material: materials.add(ColorMaterial::from(Color::BLACK)),
                transform: Transform::from_xyz(
                    rand::random::<f32>() * window.width(),
                    window.height() - rand::random::<f32>() * window.height(), // Invert Y
                    0.0,
                )
                .with_scale(Vec3::new(initial_size, initial_size, 1.0)),
                ..default()
            },
            BlackHole {
                mass: initial_mass,
                event_horizon: initial_event_horizon,
            },
        ));
    }

    if keyboard_input.just_pressed(KeyCode::Tab) {
        let black_hole_count = black_holes.iter().count();
        if black_hole_count > 0 {
            simulation_state.selected_black_hole =
                (simulation_state.selected_black_hole + 1) % black_hole_count;
        }
    }

    if keyboard_input.just_pressed(KeyCode::Delete) {
        let black_hole_entities: Vec<Entity> = black_holes.iter().collect();
        if black_hole_entities.len() > 1 {
            commands
                .entity(black_hole_entities[simulation_state.selected_black_hole])
                .despawn();
            simulation_state.selected_black_hole %= black_hole_entities.len() - 1;
        }
    }

    if keyboard_input.pressed(KeyCode::Equal) {
        simulation_state.particle_size += 0.1;
    }
    if keyboard_input.pressed(KeyCode::Minus) {
        simulation_state.particle_size = (simulation_state.particle_size - 0.1).max(0.1);
    }

    if keyboard_input.pressed(KeyCode::BracketRight) {
        simulation_state.time_scale *= 1.1;
    }
    if keyboard_input.pressed(KeyCode::BracketLeft) {
        simulation_state.time_scale /= 1.1;
    }
}

fn update_ui(
    mut commands: Commands,
    query: Query<Entity, With<Text>>,
    simulation_state: Res<SimulationState>,
) {
    // Remove existing UI
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn new UI
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Black Hole Simulator\n",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                format!(
                    "Keybindings:\n\
                    Space: Pause/Resume ({})\n\
                    Left Click: Move selected black hole\n\
                    Up/Down Arrows: Adjust black hole mass\n\
                    N: Add new black hole\n\
                    Tab: Switch selected black hole\n\
                    Delete: Remove selected black hole\n\
                    +/-: Adjust particle size\n\
                    \n\
                    Black Holes: {}\n\
                    Selected Black Hole: {}\n\
                    Particle Size: {:.1} \n\
                     Time Scale: {:.2}x \n\
                    ",
                    if simulation_state.paused {
                        "Paused"
                    } else {
                        "Running"
                    },
                    query.iter().count(),
                    simulation_state.selected_black_hole,
                    simulation_state.particle_size,
                    simulation_state.time_scale
                ),
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}
