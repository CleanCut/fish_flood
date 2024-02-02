use std::f32::consts::FRAC_PI_2;

use bevy::{app::AppExit, prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1000.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(ClearColor(Color::rgb_u8(176, 234, 238)))
        .add_systems(Startup, setup)
        .add_systems(Update, (menu, spray, score, cart_spawner))
        .insert_resource(CartSpawner {
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        })
        .run();
}

#[derive(Resource)]
struct CartSpawner {
    timer: Timer,
}

fn cart_spawner(
    time: Res<Time>,
    mut cart_spawner: ResMut<CartSpawner>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if cart_spawner.timer.tick(time.delta()).just_finished() {
        spawn_cart(&mut commands, &asset_server);
    }
}

#[derive(Component)]
struct Score {
    score: usize,
}

#[derive(Component)]
struct Player {
    hose_timer: Timer,
    offsets: [f32; 4],
    offset_index: usize,
}

impl Player {
    fn new() -> Player {
        Self {
            hose_timer: Timer::from_seconds(0.04, TimerMode::Repeating),
            offsets: [-30.0, -10.0, 10.0, 30.0],
            offset_index: 0,
        }
    }
    fn next_offset(&mut self) -> f32 {
        self.offset_index = (self.offset_index + 1) % self.offsets.len();
        self.offsets[self.offset_index]
    }
}

#[derive(Component, Default)]
struct Cart {
    capacity: usize,
    count: usize,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Ground
    commands
        .spawn(Collider::cuboid(800.0, 20.0))
        .insert(RigidBody::Fixed)
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -337.0, 0.0)));

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

    // Player
    commands.spawn(Player::new()).insert(SpriteBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 300.0, 20.0)),
        texture: asset_server.load("sprite/big-hose.png"),
        ..Default::default()
    });

    // Score Text
    let font = asset_server.load("FiraMono-Medium.ttf");
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "Score: 0",
                TextStyle {
                    font: font.clone(),
                    font_size: 60.0,
                    color: Color::BLACK,
                },
            ),
            transform: Transform::from_translation(Vec3::new(-500.0, 320.0, 50.0)),
            ..Default::default()
        },
        Score { score: 0 },
    ));

    spawn_cart(&mut commands, &asset_server);
}

fn spawn_cart(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn(RigidBody::KinematicVelocityBased)
        .insert(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-650.0, -241.0, 1.0)),
            texture: asset_server.load("sprite/robot1.png"),
            ..Default::default()
        })
        .insert(Velocity {
            linvel: Vec2::new(100.0, 0.0),
            ..Default::default()
        })
        .insert(Cart {
            capacity: 15,
            count: 0,
        })
        .insert(Collider::compound(vec![(
            Vec2::new(0.0, 8.0),
            0.0,
            Collider::cuboid(80.0, 58.0),
        )]))
        .insert(Sensor)
        .with_children(|children| {
            children.spawn(Collider::compound(vec![
                (Vec2::new(-85.0, -2.0), 0.0, Collider::cuboid(5.0, 68.0)),
                (Vec2::new(85.0, -2.0), 0.0, Collider::cuboid(5.0, 68.0)),
                (Vec2::new(0.0, -60.0), 0.0, Collider::cuboid(85.0, 10.0)),
            ]));
        });
}

fn menu(keyboard_input: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn spray(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    time: Res<Time>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    // Get stuff for the viewport-to-world transformation
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };
    let mouse_pos = camera
        .viewport_to_world_2d(camera_transform, cursor_pos)
        .unwrap();

    // Get player stuff
    let Ok((mut player, mut player_transform)) = player_query.get_single_mut() else {
        return;
    };

    // Player follows mouse
    player_transform.translation.x = mouse_pos.x;

    // Spray Fishes
    if player.hose_timer.tick(time.delta()).just_finished()
        && mouse_button_input.pressed(MouseButton::Left)
    {
        commands
            .spawn(RigidBody::Dynamic)
            .insert(SpriteBundle {
                texture: asset_server.load("sprite/fish_orange.png"),
                ..Default::default()
            })
            .insert(Collider::capsule_x(10.0, 10.0))
            .insert(Restitution::coefficient(0.7))
            .insert(Friction::coefficient(0.08))
            .insert(TransformBundle::from_transform(Transform {
                translation: player_transform.translation
                    + Vec3::new(player.next_offset(), -28.0, 1.0),
                rotation: Quat::from_axis_angle(Vec3::Z, -FRAC_PI_2),
                ..Default::default()
            }))
            .insert(ExternalImpulse {
                impulse: Vec2::new(0.0, -0.25),
                torque_impulse: rand::thread_rng().gen_range(-0.000001..0.000001),
            })
            .insert(GravityScale(10.0));
    }
}

fn score(
    rapier_context: Res<RapierContext>,
    mut cart_query: Query<(Entity, &mut Cart, &Transform)>,
    mut score_query: Query<(&mut Score, &mut Text)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Ok((mut score, mut text)) = score_query.get_single_mut() else {
        return;
    };
    for (entity, mut cart, transform) in cart_query.iter_mut() {
        // How many fish are in the cart?
        let cart_count = rapier_context.intersections_with(entity).count();
        if cart_count > cart.capacity {
            // Cart breaks?
            commands.entity(entity).despawn_recursive();
            if score.score > cart_count {
                score.score -= cart_count;
            } else {
                score.score = 0;
            }
        } else if transform.translation.x > 750.0 {
            // Cart scores?
            commands.entity(entity).despawn_recursive();
            score.score += cart_count;
        }
        text.sections[0].value = format!("Score: {}", score.score);
        // for (collider1, collider2, intersecting) in rapier_context.intersections_with(entity) {
        //     if intersecting {
        //         println!(
        //             "The entities {:?} and {:?} have intersecting colliders!",
        //             collider1, collider2
        //         );
        //     }
        // }
    }
}
