use macroquad::prelude::*;

// --- CONFIGURAÇÕES DO MUNDO ---
pub const INITIAL_AGENTS: usize = 4;
pub const INITIAL_FOOD_COUNT: usize = 2;
pub const MAX_FOOD_SPAWN_CHANCE: i32 = 40; 
pub const VISION_RADIUS: f32 = 280.0;
pub const WALL_SENSE_DIST: f32 = 80.0; 

// --- METABOLISMO ---
pub const INITIAL_ENERGY: f32 = 120.0;
pub const REPRODUCTION_THRESHOLD: f32 = 280.0;
pub const REPRODUCTION_COST: f32 = 150.0;
pub const REPRO_COOLDOWN: f32 = 5.0;
pub const ENERGY_LOSS_IDLE: f32 = 0.01;
pub const ENERGY_LOSS_MOVE: f32 = 0.03;
pub const ENERGY_LOSS_RUN: f32 = 0.20;
pub const ENERGY_LOSS_ROTATION: f32 = 0.08; 
pub const FOOD_ENERGY_GAIN: f32 = 90.0;

// --- COMBATE ---
pub const ATTACK_COOLDOWN_TIME: f32 = 2.0;
pub const ATTACK_COST: f32 = 5.0;
pub const ATTACK_DASH_IMPULSE: f32 = 14.0;
pub const ATTACK_DAMAGE: f32 = 45.0;
pub const ATTACK_LIFESTEAL: f32 = 20.0;
pub const ATTACK_RADIUS: f32 = 40.0;
pub const HIT_STUN_TIME: f32 = 0.7;
pub const FAMILY_PROTECTION_TIME: f32 = 10.0;
pub const SPAWN_PROTECTION_TIME: f32 = 4.0;

// --- FÍSICA E NAVEGAÇÃO ---
pub const FRICTION_LINEAR: f32 = 0.90;
pub const FRICTION_IMPULSE: f32 = 0.80;
pub const BASE_TURN_SPEED: f32 = 0.40;   
pub const MAX_AGENT_SPEED: f32 = 1.6;
pub const WALL_REPULSION_FORCE: f32 = 3.0; 
pub const WALL_BOUNCE_ELASTICITY: f32 = 0.6;