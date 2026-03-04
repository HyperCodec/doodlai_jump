pub const PLATFORM_COUNT: u32 = 5;
pub const GAME_WIDTH: f32 = 540.;
pub const GAME_HEIGHT: f32 = 960.;
pub const PLAYER_TOP_MARGIN: f32 = 10.0;
pub const PLAYER_SPEED: f32 = 1.0;

use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Player;

#[derive(Component, Debug, Default)]
pub struct Velocity(pub Vec2);

#[derive(Resource, Debug)]
pub struct Gravity(pub Vec2);

impl Default for Gravity {
    fn default() -> Self {
        Self(Vec2 { x: 0.0, y: -9.8 })
    }
}

#[derive(Component, Debug)]
pub struct RectCollider(pub Vec2);

impl RectCollider {
    pub fn corners(&self, center: Vec2) -> [Vec2; 4] {
        let half = self.0 / 2.0;
        [
            center
                + Vec2 {
                    x: -half.x,
                    y: -half.y,
                },
            center
                + Vec2 {
                    x: half.x,
                    y: -half.y,
                },
            center
                + Vec2 {
                    x: -half.x,
                    y: half.y,
                },
            center
                + Vec2 {
                    x: half.x,
                    y: half.y,
                },
        ]
    }

    pub fn contains(&self, center: Vec2, point: Vec2) -> bool {
        let top = center.y + self.0.y / 2.0;
        let bottom = center.y - self.0.y / 2.0;
        let left = center.x - self.0.x / 2.0;
        let right = center.x + self.0.x / 2.0;
        point.x >= left && point.x <= right && point.y >= bottom && point.y <= top
    }

    pub fn is_colliding(&self, center: Vec2, other: &RectCollider, other_center: Vec2) -> bool {
        if self.contains(center, other_center) || other.contains(other_center, center) {
            return true;
        }
        for corner in other.corners(other_center) {
            if self.contains(center, corner) {
                return true;
            }
        }
        for corner in self.corners(center) {
            if other.contains(other_center, corner) {
                return true;
            }
        }
        false
    }
}

// TODO write tests for colliders

#[derive(Component, Debug, Clone, Default, Copy)]
pub enum FacingDirection {
    Left,

    #[default]
    Right,
}

#[derive(Bundle, Debug)]
pub struct PlayerBundle {
    pub marker: Player,
    pub velocity: Velocity,
    pub transform: Transform,
    pub collider: RectCollider,
    pub facing: FacingDirection,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            marker: Player::default(),
            velocity: Velocity::default(),
            transform: Transform::default(),
            collider: RectCollider(Vec2 { x: 0.5, y: 1.0 }),
            facing: FacingDirection::Right,
        }
    }
}

#[derive(Resource, Debug)]
pub struct ScrollHeight(pub f32);

impl Default for ScrollHeight {
    fn default() -> Self {
        Self(PLAYER_TOP_MARGIN + 0.5)
    }
}

#[derive(Event, Debug, Clone)]
pub struct DeathEvent {
    pub high_score: f32,
}

#[derive(Component, Debug, Default)]
pub struct Platform;

#[derive(Bundle, Debug)]
pub struct PlatformBundle {
    pub marker: Platform,
    pub collider: RectCollider,
    pub transform: Transform,
}

impl Default for PlatformBundle {
    fn default() -> Self {
        Self {
            marker: Platform,
            collider: RectCollider(Vec2 { x: 0.5, y: 1.0 }),
            transform: Transform::default(),
        }
    }
}

pub fn doodl_jump_plugin(app: &mut App) {
    app.init_resource::<ScrollHeight>()
        .init_resource::<Gravity>()
        .add_systems(Startup, setup_doodl_jump)
        .add_systems(
            Update,
            (
                update_scroll,
                (apply_gravity, apply_velocity, check_death).chain(),
            ),
        );
}

pub fn setup_doodl_jump(mut commands: Commands) {
    // player
    commands.spawn((
        PlayerBundle::default(),
        // TODO sprite bundle on render feature
    ));
}

pub fn update_scroll(
    player_q: Query<(&Transform, &RectCollider), With<Player>>,
    platforms_q: Query<(Entity, &Transform, &RectCollider), With<Platform>>,
    mut commands: Commands,
    mut scroll: ResMut<ScrollHeight>,
) {
    let (player_transform, player_collider) = player_q.single().unwrap();
    let player_top = player_transform.translation.y + player_collider.0.y / 2.0;
    let top_with_margin = player_top + PLAYER_TOP_MARGIN;

    if top_with_margin > scroll.0 {
        scroll.0 = top_with_margin;

        for (platform, platform_transform, platform_collider) in platforms_q.iter() {
            let platform_top = platform_transform.translation.y + platform_collider.0.y / 2.0;
            if platform_top < scroll.0 - GAME_HEIGHT {
                // off bottom of screen, despawn
                commands.entity(platform).despawn();

                // TODO spawn new random platform to replace it and keep
                // the total platforms on screen consistent.
            }
        }

        // TODO camera on render feature
    }
}

// register death and allow downstream crates to figure out what to do when the player dies
// ring: end game, record high score as fitness, maybe test a few extra times for consistency.
// display: show a game over menu that allows the player to restart.
pub fn check_death(
    player_q: Query<&Transform, With<Player>>,
    scroll: Res<ScrollHeight>,
    mut commands: Commands,
) {
    let player_transform = player_q.single().unwrap();
    let player_bottom = player_transform.translation.y - player_transform.scale.y / 2.0;
    if player_bottom < scroll.0 - GAME_HEIGHT {
        commands.trigger(DeathEvent {
            high_score: scroll.0,
        });
    }
}

pub fn apply_gravity(mut velocities: Query<&mut Velocity>, gravity: Res<Gravity>, time: Res<Time>) {
    let dt = time.delta_secs();
    for mut velocity in velocities.iter_mut() {
        velocity.0 += gravity.0 * dt;
    }
}

pub fn apply_velocity(mut transforms: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in transforms.iter_mut() {
        transform.translation += (velocity.0 * dt).extend(0.0);
    }
}

// in the display crate, these will get passed automatically by `DefaultPlugins`,
// but for any AI usage, we have to trigger the ButtonInput events ourselves.
pub fn handle_inputs(
    event: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut transform: Query<(&mut Transform, &mut FacingDirection), With<Player>>,
) {
    let (mut player_transform, mut player_facing) = transform.single_mut().unwrap();
    let dt = time.delta_secs();

    if event.pressed(KeyCode::ArrowRight) {
        player_transform.translation.x += PLAYER_SPEED * dt;
        *player_facing = FacingDirection::Right;
    }

    if event.pressed(KeyCode::ArrowLeft) {
        player_transform.translation.x -= PLAYER_SPEED * dt;
        *player_facing = FacingDirection::Left;
    }
}

// TODO platform and wall collision
