const NUM_PLATFORMS: u32 = 5;
const GAME_WIDTH: f32 = 540.;
const GAME_HEIGHT: f32 = 960.;
const PLAYER_TOP_MARGIN: f32 = 10.0;
const PLAYER_HORIZ_SPEED: f32 = 1.0;
const PLAYER_JUMP_SPEED: f32 = 3.0;
const FIXED_DT: Option<f32> = None;
const FIXED_RNG_SEED: Option<u64> = None;

const PLAYER_WIDTH: f32 = 0.5;
const PLAYER_HEIGHT: f32 = 1.0;

const PLATFORM_WIDTH: f32 = 1.0;
const PLATFORM_HEIGHT: f32 = 0.5;

use std::{ops::{Deref, DerefMut}, time::{SystemTime, UNIX_EPOCH}};

use bevy::prelude::*;
use rand::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Player;

#[derive(Component, Debug, Default)]
pub struct Velocity(pub Vec2);

#[derive(Resource, Debug, Clone)]
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
            collider: RectCollider(Vec2 { x: PLAYER_WIDTH, y: PLAYER_HEIGHT }),
            facing: FacingDirection::Right,
        }
    }
}

#[derive(Resource, Debug)]
pub struct ScrollHeight(pub f32);

impl Default for ScrollHeight {
    fn default() -> Self {
        // assuming player at center
        Self(PLAYER_TOP_MARGIN + PLAYER_HEIGHT / 2.0)
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
            collider: RectCollider(Vec2 { x: PLATFORM_WIDTH, y: PLATFORM_HEIGHT }),
            transform: Transform::default(),
        }
    }
}

#[derive(Event, Debug, Default)]
pub struct PlayerJumpEvent;

#[derive(Resource, Debug, Clone)]
pub struct DoodlJumpSettings {
    pub platform_count: u32,
    pub game_width: f32,
    pub game_height: f32,
    pub player_top_margin: f32,
    pub player_horiz_speed: f32,
    pub player_jump_speed: f32,
    pub fixed_dt: Option<f32>,
    pub fixed_rng_seed: Option<u64>,
    pub num_platforms: u32,
}

impl Default for DoodlJumpSettings {
    fn default() -> Self {
        Self {
            platform_count: NUM_PLATFORMS,
            game_width: GAME_WIDTH,
            game_height: GAME_HEIGHT,
            player_top_margin: PLAYER_TOP_MARGIN,
            player_horiz_speed: PLAYER_HORIZ_SPEED,
            player_jump_speed: PLAYER_JUMP_SPEED,
            fixed_dt: FIXED_DT,
            fixed_rng_seed: FIXED_RNG_SEED,
            num_platforms: NUM_PLATFORMS,
        }
    }
}

impl DoodlJumpSettings {
    pub fn dt(&self, time: &Time) -> f32 {
        self.fixed_dt.unwrap_or_else(|| time.delta_secs())
    }

    pub fn non_platform_space(&self) -> f32 {
        self.game_height - (self.num_platforms as f32 * PLATFORM_HEIGHT)
    }

    pub fn space_between_platforms(&self) -> f32 {
        self.non_platform_space() / self.num_platforms as f32
    }

    pub fn max_initial_platform_y(&self) -> f32 {
        self.game_height / 2.0 - PLATFORM_HEIGHT / 2.0 - self.space_between_platforms() / 2.0
    }

    pub fn min_initial_platform_y(&self) -> f32 {
        -self.game_height / 2.0 + PLATFORM_HEIGHT / 2.0 + self.space_between_platforms() / 2.0
    }

    pub fn min_platform_x(&self) -> f32 {
        -self.game_width / 2.0 + PLATFORM_WIDTH / 2.0
    }
    
    pub fn max_platform_x(&self) -> f32 {
        self.game_width / 2.0 - PLATFORM_WIDTH / 2.0
    }
}

#[derive(Resource, Debug)]
pub struct RngSource(pub StdRng);

impl Deref for RngSource {
    type Target = StdRng;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RngSource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default, Debug)]
pub struct DoodlJumpPlugin {
    pub settings: DoodlJumpSettings,
    pub gravity: Gravity,
}

impl Plugin for DoodlJumpPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ScrollHeight>()
            .insert_resource(self.gravity.clone())
            .insert_resource(self.settings.clone())
            .insert_resource(RngSource(StdRng::seed_from_u64(self.settings.fixed_rng_seed.unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()))))
            .add_systems(Startup, setup_doodl_jump)
            .add_systems(
                Update,
                (
                    update_scroll,
                    (
                        apply_gravity,
                        apply_velocity,
                        wrap_around_walls,
                        collide_with_platforms,
                        check_death,
                    ).chain(),
                ),
            );
        // TODO restrict window on render feature
    }
}

pub fn setup_doodl_jump(mut commands: Commands, mut rng: ResMut<RngSource>, settings: Res<DoodlJumpSettings>,) {
    // player
    commands.spawn((
        PlayerBundle::default(),
        // TODO sprite bundle on render feature
    ));

    // TODO spawn camera on render feature
    
    let top = settings.max_initial_platform_y();
    let bottom = settings.min_initial_platform_y();
    let left = settings.min_platform_x();
    let right = settings.max_platform_x();

    let step = (top - bottom) / settings.num_platforms as f32;

    for i in 0..5 {
        let y = top - i as f32 * step;
        let x = rng.random_range(left..right);

        commands.spawn((
            PlatformBundle {
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            // TODO SpriteBundle on render feature
        ));
    }
}

pub fn update_scroll(
    player_q: Query<(&Transform, &RectCollider), With<Player>>,
    platforms_q: Query<(Entity, &Transform, &RectCollider), With<Platform>>,
    settings: Res<DoodlJumpSettings>,
    mut rng: ResMut<RngSource>,
    mut commands: Commands,
    mut scroll: ResMut<ScrollHeight>,
) {
    let (player_transform, player_collider) = player_q.single().unwrap();
    let player_top = player_transform.translation.y + player_collider.0.y / 2.0;
    let top_with_margin = player_top + settings.player_top_margin;

    if top_with_margin > scroll.0 {
        scroll.0 = top_with_margin;

        for (platform, platform_transform, platform_collider) in platforms_q.iter() {
            let platform_top = platform_transform.translation.y + platform_collider.0.y / 2.0;
            if platform_top < scroll.0 - settings.game_height {
                // off bottom of screen, despawn
                commands.entity(platform).despawn();

                let y = settings.max_initial_platform_y();
                let left = settings.min_platform_x();
                let right = settings.max_platform_x();

                let x = rng.random_range(left..right);
                commands.spawn((
                    PlatformBundle {
                        transform: Transform::from_xyz(x, y, 0.0),
                        ..default()
                    },
                    // TODO SpriteBundle on render feature
                ));
            }
        }

        // TODO move camera on render feature
    }
}

// register death and allow downstream crates to figure out what to do when the player dies
// ring: end game, record high score as fitness, maybe test a few extra times for consistency.
// display: show a game over menu that allows the player to restart.
pub fn check_death(
    player_q: Query<&Transform, With<Player>>,
    scroll: Res<ScrollHeight>,
    settings: Res<DoodlJumpSettings>,
    mut commands: Commands,
) {
    let player_transform = player_q.single().unwrap();
    let player_bottom = player_transform.translation.y - player_transform.scale.y / 2.0;
    if player_bottom < scroll.0 - settings.game_height {
        commands.trigger(DeathEvent {
            high_score: scroll.0,
        });
    }
}

pub fn apply_gravity(mut velocities: Query<&mut Velocity>, settings: Res<DoodlJumpSettings>, gravity: Res<Gravity>, time: Res<Time>) {
    let dt = settings.dt(&time);
    for mut velocity in velocities.iter_mut() {
        velocity.0 += gravity.0 * dt;
    }
}

pub fn apply_velocity(mut transforms: Query<(&mut Transform, &Velocity)>, settings: Res<DoodlJumpSettings>, time: Res<Time>) {
    let dt = settings.dt(&time);
    for (mut transform, velocity) in transforms.iter_mut() {
        transform.translation += (velocity.0 * dt).extend(0.0);
    }
}

// in the display crate, these will get passed automatically by `DefaultPlugins`,
// but for any AI usage, we have to trigger the ButtonInput events ourselves.
pub fn handle_inputs(
    event: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<DoodlJumpSettings>,
    mut transform: Query<(&mut Transform, &mut FacingDirection), With<Player>>,
) {
    let (mut player_transform, mut player_facing) = transform.single_mut().unwrap();
    let dt = settings.dt(&time);

    if event.pressed(KeyCode::ArrowRight) {
        player_transform.translation.x += settings.player_horiz_speed * dt;
        *player_facing = FacingDirection::Right;
    }

    if event.pressed(KeyCode::ArrowLeft) {
        player_transform.translation.x -= settings.player_horiz_speed * dt;
        *player_facing = FacingDirection::Left;
    }
}

pub fn collide_with_platforms(
    mut player_q: Query<(&mut Transform, &mut Velocity, &RectCollider), With<Player>>,
    mut commands: Commands,
    platforms_q: Query<(&Transform, &RectCollider), With<Platform>>,
    settings: Res<DoodlJumpSettings>,
) {
    let (mut player_transform, mut player_velocity, player_collider) = player_q.single_mut().unwrap();
    if player_velocity.0.y >= 0.0 {
        // only collide with platforms when falling
        return;
    }

    for (platform_transform, platform_collider) in platforms_q.iter() {
        if platform_transform.translation.y >= player_transform.translation.y {
            // player must at least be above the center of the platform
            continue;
        }

        if platform_collider.is_colliding(platform_transform.translation.xy(), player_collider, player_transform.translation.xy()) {
            // player is colliding, moving downward, and above the center, so we should jump.
            player_transform.translation.y = platform_transform.translation.y + platform_collider.0.y / 2.0;
            player_velocity.0.y = settings.player_jump_speed;

            // pass jump event for audio and such
            commands.trigger(PlayerJumpEvent);

            // max 1 jump per tick
            return;
        }
    }
}

pub fn wrap_around_walls(
    mut player_q: Query<(&mut Transform, &RectCollider), With<Player>>,
    settings: Res<DoodlJumpSettings>,
) {
    let (mut player_transform, player_collider) = player_q.single_mut().unwrap();
    let player_right = player_transform.translation.x + player_collider.0.x / 2.0;
    let player_left = player_transform.translation.x - player_collider.0.x / 2.0;

    if player_left > settings.game_width / 2.0 {
        // disappeared off right edge, loop to left.
        player_transform.translation.x = -(settings.game_width / 2.0) - player_collider.0.x / 2.0;
        return; // impossible to loop twice per tick anyway
    }

    if player_right < -(settings.game_width / 2.0) {
        // disappeared off left edge, loop to right.
        player_transform.translation.x = settings.game_width / 2.0 + player_collider.0.x / 2.0;
    }
}