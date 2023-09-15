use bevy::{app::AppExit, prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1000.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(ClearColor(Color::rgb_u8(176, 234, 238)))
        .add_systems(Startup, setup)
        .add_systems(Update, spray)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Ground
    commands
        .spawn(Collider::cuboid(800.0, 20.0))
        .insert(RigidBody::Fixed)
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -337.0, 0.0)));

    // Cart
    commands
        .spawn(Collider::compound(vec![
            (Vec2::new(-85.0, -2.0), 0.0, Collider::cuboid(5.0, 68.0)),
            (Vec2::new(85.0, -2.0), 0.0, Collider::cuboid(5.0, 68.0)),
            (Vec2::new(0.0, -60.0), 0.0, Collider::cuboid(85.0, 10.0)),
        ]))
        .insert(RigidBody::KinematicVelocityBased)
        .insert(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-400.0, -241.0, 1.0)),
            texture: asset_server.load("sprite/robot1.png"),
            ..Default::default()
        })
        .insert(Velocity {
            linvel: Vec2::new(100.0, 0.0),
            ..Default::default()
        });

    // Ground
    for i in 0..10 {
        commands.spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                -576.0 + 128.0 * (i as f32),
                -380.0,
                0.0,
            )),
            texture: asset_server.load("sprite/dirt.png"),
            ..Default::default()
        });
    }
}

fn spray(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
    mut exit: EventWriter<AppExit>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    let Some(mouse_pos) = window.cursor_position() else {
        return;
    };
    // Get stuff for the viewport-to-world transformation
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };
    let mouse_world_space = camera
        .viewport_to_world_2d(camera_transform, mouse_pos)
        .unwrap();

    if mouse_button_input.pressed(MouseButton::Left) {
        commands
            .spawn(RigidBody::Dynamic)
            .insert(SpriteBundle {
                texture: asset_server.load("sprite/fish_orange.png"),
                ..Default::default()
            })
            .insert(Collider::capsule_x(10.0, 10.0))
            .insert(Restitution::coefficient(0.7))
            .insert(Friction::coefficient(0.08))
            .insert(TransformBundle::from(Transform::from_translation(
                mouse_world_space.extend(10.0),
            )))
            .insert(GravityScale(8.0));
    }
}
