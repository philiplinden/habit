use std::time::Duration;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_xpbd_2d::{math::*, prelude::*};

#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

const TETHER_LENGTH: f32 = 100.0;
const TETHER_THICKNESS: f32 = 3.0;
const TETHER_LINK_RATIO: f32 = 4.0;
const TETHER_COMPLIANCE: f32 = 0.00001;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            UiControlsPlugin,
            #[cfg(feature = "debug")]
            WorldInspectorPlugin::new(),
        ))
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .insert_resource(SubstepCount(50))
        .insert_resource(Gravity::default())
        .add_systems(Startup, setup_tether)
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum AppState {
    Paused,
    #[default]
    Running,
}

fn pause_button(
    current_state: ResMut<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        let new_state = match current_state.get() {
            AppState::Paused => AppState::Running,
            AppState::Running => AppState::Paused,
        };
        next_state.0 = Some(new_state);
    }
}

fn step_button(mut time: ResMut<Time<Physics>>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Enter) {
        time.advance_by(Duration::from_secs_f64(1.0 / 60.0));
    }
}

#[derive(Component)]
struct FpsText;

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "FPS: ",
            TextStyle {
                font: default(),
                font_size: 20.0,
                color: Color::TOMATO,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        FpsText,
    ));
}

fn update_fps_text(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[0].value = format!("FPS: {value:.2}");
            }
        }
    }
}

#[derive(Default)]
pub struct UiControlsPlugin;

impl Plugin for UiControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsPlugins::default(),
            FrameTimeDiagnosticsPlugin,
            #[cfg(feature = "debug")]
            PhysicsDebugPlugin::default(),
        ))
        .init_state::<AppState>()
        .add_systems(Startup, setup_ui)
        .add_systems(
            OnEnter(AppState::Paused),
            |mut time: ResMut<Time<Physics>>| time.pause(),
        )
        .add_systems(
            OnExit(AppState::Paused),
            |mut time: ResMut<Time<Physics>>| time.unpause(),
        )
        .add_systems(Update, update_fps_text)
        .add_systems(Update, pause_button)
        .add_systems(Update, step_button.run_if(in_state(AppState::Paused)));
    }
}

fn setup_tether(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2dBundle::default());

    let particle_radius = TETHER_THICKNESS;
    let particle_length = TETHER_THICKNESS * TETHER_LINK_RATIO;
    let particle_mesh: Mesh2dHandle = meshes
        .add(Capsule2d::new(
            particle_radius,
            particle_length,
        ))
        .into();
    let particle_material = materials.add(Color::rgb(0.9, 0.7, 0.2));

    // Spawn kinematic particle that can follow the mouse
    let mut previous_particle = commands
        .spawn((
            RigidBody::Kinematic,
            MaterialMesh2dBundle {
                mesh: particle_mesh.clone(),
                material: particle_material.clone(),
                ..default()
            },
        ))
        .id();

    // Spawn the rest of the particles, connecting each one to the previous one with joints
    let num_joints = TETHER_LENGTH / particle_length;
    for i in 1..num_joints as i32 {
        let current_particle = commands
            .spawn((
                RigidBody::Dynamic,
                MassPropertiesBundle::new_computed(&Collider::capsule(particle_length, particle_radius), 1.0),
                MaterialMesh2dBundle {
                    mesh: particle_mesh.clone(),
                    material: particle_material.clone(),
                    transform: Transform::from_xyz(
                        0.0,
                        -i as f32 * (particle_length + 1.0),
                        0.0,
                    ),
                    ..default()
                },
            ))
            .id();

        commands.spawn(
            RevoluteJoint::new(previous_particle, current_particle)
                .with_local_anchor_1(Vector::Y * (particle_length * -0.5))
                .with_local_anchor_2(Vector::Y * (particle_length * 0.5))
                .with_compliance(TETHER_COMPLIANCE),
        );

        previous_particle = current_particle;
    }
}
