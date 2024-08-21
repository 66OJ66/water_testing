use bevy::core_pipeline::experimental::taa::TemporalAntiAliasBundle;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::mesh::PlaneMeshBuilder;
use bevy_atmosphere::prelude::*;
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_water::material::*;
use bevy_water::*;

#[derive(Copy, Clone, Eq, PartialEq)]
enum Mode {
    /// Use bevy_water's default plugin so water spawns automatically. TAA enabled.
    WaterPlugin,
    /// Press 'E' to spawn water manually. TAA disabled.
    ManualSpawnNoTAA,
    /// Press 'E' to spawn water manually. TAA enabled.
    ManualSpawnTAA,
}

// Change this to experiment
const MODE: Mode = Mode::WaterPlugin;

fn main() {
    let mut app = App::new();

    // Plugins (always used regardless of MODE)
    app.add_plugins((DefaultPlugins, NoCameraPlayerPlugin, AtmospherePlugin));

    match MODE {
        Mode::WaterPlugin => {
            app.add_plugins(WaterPlugin);
        }
        Mode::ManualSpawnNoTAA | Mode::ManualSpawnTAA => {
            app.add_plugins(WaterMaterialPlugin)
                .add_systems(Update, spawn_water);
        }
    }

    app.add_systems(Startup, startup).run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut camera = commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 20.0, 10.0).looking_at(
                Vec3 {
                    x: 0.0,
                    y: 20.0,
                    z: 0.0,
                },
                Vec3::Y,
            ),
            ..Default::default()
        },
        AtmosphereCamera::default(),
        FlyCam,
    ));

    if MODE == Mode::WaterPlugin || MODE == Mode::ManualSpawnTAA {
        camera.insert(TemporalAntiAliasBundle::default());
    }

    // Cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_length(1.0)),
        material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
        transform: Transform::from_xyz(0.0, 20.0, 0.0),
        ..default()
    });

    // Sun
    commands.spawn((
        Name::from("Sun"),
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::srgba_u8(250, 255, 230, 0),
                shadows_enabled: true,
                ..default()
            },
            ..default()
        },
    ));
}

fn spawn_water(
    mut commands: Commands,
    // Resources
    keys: Res<ButtonInput<KeyCode>>,
    // Assets
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardWaterMaterial>>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        info!("Spawning water");

        setup_water(
            &mut commands,
            &WaterSettings::default(),
            &mut meshes,
            &mut materials,
        );
    }
}

fn setup_water(
    commands: &mut Commands,
    settings: &WaterSettings,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardWaterMaterial>,
) {
    let grid = match settings.spawn_tiles {
        Some(grid) => grid,
        None => {
            return;
        }
    };
    let water_height = settings.height;
    // Generate mesh for water.
    let mesh: Handle<Mesh> = meshes
        .add(PlaneMeshBuilder::from_length(WATER_SIZE as f32).subdivisions(WATER_SIZE as u32 / 4));

    commands
        .spawn(WaterBundle {
            name: Name::new("Water"),
            ..default()
        })
        .with_children(|parent| {
            let grid_center = (WATER_SIZE * WATER_GRID_SIZE) as f32 / 2.0;
            for x in 0..grid.x {
                for y in 0..grid.y {
                    let x = (x * WATER_SIZE) as f32 - grid_center;
                    let y = (y * WATER_SIZE) as f32 - grid_center;
                    // UV starts at (0,0) at the corner.
                    let coord_offset = Vec2::new(x, y);
                    // Water material.
                    let material = materials.add(StandardWaterMaterial {
                        base: StandardMaterial {
                            base_color: settings.base_color,
                            perceptual_roughness: 0.22,
                            ..default()
                        },
                        extension: WaterMaterial {
                            amplitude: settings.amplitude,
                            clarity: settings.clarity,
                            deep_color: settings.deep_color,
                            shallow_color: settings.shallow_color,
                            edge_color: settings.edge_color,
                            edge_scale: settings.edge_scale,
                            coord_offset,
                            coord_scale: Vec2::new(WATER_SIZE as f32, WATER_SIZE as f32),
                            ..default()
                        },
                    });

                    parent.spawn((
                        WaterTileBundle::new(mesh.clone(), material, water_height, coord_offset),
                        NotShadowCaster,
                    ));
                }
            }
        });
}
