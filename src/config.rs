use macroquad::prelude::*;

// --- CONFIGURAÇÕES DO MUNDO ---
pub const INITIAL_AGENTS: usize = 10;
pub const INITIAL_FOOD_COUNT: usize = 5;
pub const MAX_FOOD_SPAWN_CHANCE: i32 = 30; // 1 em 30 frames
pub const VISION_RADIUS: f32 = 220.0;
pub const WALL_SENSE_DIST: f32 = 55.0;

// --- METABOLISMO ---
pub const INITIAL_ENERGY: f32 = 120.0;
pub const REPRODUCTION_THRESHOLD: f32 = 300.0;
pub const REPRODUCTION_COST: f32 = 150.0;
pub const ENERGY_LOSS_IDLE: f32 = 0.015;
pub const ENERGY_LOSS_MOVE: f32 = 0.05;
pub const ENERGY_LOSS_RUN: f32 = 0.30;
pub const ENERGY_LOSS_ROTATION: f32 = 0.04;
pub const ATTACK_COST: f32 = 1.2;
pub const WALL_BUMP_PENALTY: f32 = 0.05;
pub const FOOD_ENERGY_GAIN: f32 = 95.0;

// --- COMBATE E SOBREVIVÊNCIA ---
pub const FAMILY_PROTECTION_TIME: f32 = 10.0;
pub const SPAWN_PROTECTION_TIME: f32 = 5.0;
pub const ATTACK_COOLDOWN_TIME: f32 = 1.5;
pub const ATTACK_VISUAL_DURATION: f32 = 0.6;
pub const HIT_STUN_TIME: f32 = 0.6;
pub const KNOCKBACK_IMPULSE: f32 = 16.0;
pub const KNOCKBACK_RECOIL_FACTOR: f32 = 0.3;
pub const ATTACK_DAMAGE: f32 = 40.0;
pub const ATTACK_LIFESTEAL: f32 = 25.0;
pub const ATTACK_RADIUS: f32 = 28.0;

// --- FÍSICA E MOVIMENTO ---
pub const FRICTION_LINEAR: f32 = 0.94;
pub const FRICTION_IMPULSE: f32 = 0.82;
pub const WANDER_SPEED: f32 = 0.6;
pub const BASE_TURN_SPEED: f32 = 0.3;
pub const HIT_TURN_SPEED: f32 = 0.1;
pub const MAX_AGENT_SPEED: f32 = 1.5;
pub const ATTACK_DASH_IMPULSE: f32 = 7.0;

// --- CORES E UI ---
pub const COLOR_BACKGROUND: Color = Color::new(0.01, 0.01, 0.02, 1.0);
pub const COLOR_WALL: Color = Color::new(0.3, 0.3, 0.7, 0.3);
pub const COLOR_FOOD: Color = GREEN;
pub const COLOR_ATTACK: Color = RED;