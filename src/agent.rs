use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::brain::{Brain, Sensor, INPUT_SIZE};
use crate::config::*;

pub struct Agent {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub pos: Vec2,
    pub angle: f32,
    pub energy: f32,
    pub brain: Brain,
    pub color: Color,
    pub is_attacking: bool,
    pub current_speed: f32,
    pub impulse_vec: Vec2,
    pub generation: u32,
    pub family_timer: f32,
    pub spawn_timer: f32,
    pub attack_cooldown: f32,
    pub hit_timer: f32,
    pub repro_timer: f32,
}

impl Agent {
    pub fn new(id: u64, parent_id: Option<u64>, pos: Vec2, brain: Brain, color: Color, generation: u32) -> Self {
        Self {
            id, parent_id, pos, brain, color, generation,
            angle: gen_range(0.0, std::f32::consts::TAU),
            energy: INITIAL_ENERGY,
            is_attacking: false,
            current_speed: 0.0,
            impulse_vec: Vec2::ZERO,
            family_timer: FAMILY_PROTECTION_TIME,
            spawn_timer: SPAWN_PROTECTION_TIME,
            attack_cooldown: 0.0,
            hit_timer: 0.0,
            repro_timer: 0.0,
        }
    }

    fn normalize_angle(angle: f32) -> f32 {
        let mut a = angle;
        while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
        while a < -std::f32::consts::PI { a += std::f32::consts::TAU; }
        a
    }

    pub fn update(&mut self, closest_food: Option<Vec2>, closest_enemy: Option<Vec2>, world_w: f32, world_h: f32, dt: f32) {
        if self.family_timer > 0.0 { self.family_timer -= dt; }
        if self.spawn_timer > 0.0 { self.spawn_timer -= dt; }
        if self.attack_cooldown > 0.0 { self.attack_cooldown -= dt; }
        if self.hit_timer > 0.0 { self.hit_timer -= dt; }
        if self.repro_timer > 0.0 { self.repro_timer -= dt; }

        let mut s = [0.0; INPUT_SIZE];
        let center = vec2(world_w / 2.0, world_h / 2.0);

        if let Some(f) = closest_food {
            let rel = f - self.pos;
            s[Sensor::FoodDist as usize] = (1.0 - (rel.length() / VISION_RADIUS)).clamp(0.0, 1.0);
            s[Sensor::FoodAngle as usize] = Self::normalize_angle(rel.y.atan2(rel.x) - self.angle) / std::f32::consts::PI;
        }

        if let Some(e) = closest_enemy {
            let rel = e - self.pos;
            s[Sensor::EnemyDist as usize] = (1.0 - (rel.length() / VISION_RADIUS)).clamp(0.0, 1.0);
            s[Sensor::EnemyAngle as usize] = Self::normalize_angle(rel.y.atan2(rel.x) - self.angle) / std::f32::consts::PI;
        }

        let dist_left = self.pos.x;
        let dist_right = world_w - self.pos.x;
        let dist_up = self.pos.y;
        let dist_down = world_h - self.pos.y;

        s[Sensor::WallLeft as usize] = (1.0 - (dist_left / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        s[Sensor::WallRight as usize] = (1.0 - (dist_right / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        
        let dir = vec2(self.angle.cos(), self.angle.sin());
        let lookahead = self.pos + dir * WALL_SENSE_DIST;
        s[Sensor::WallAhead as usize] = if lookahead.x < 0.0 || lookahead.x > world_w || lookahead.y < 0.0 || lookahead.y > world_h { 1.0 } else { 0.0 };

        s[Sensor::EnergyLevel as usize] = (self.energy / REPRODUCTION_THRESHOLD).clamp(0.0, 1.0);
        s[Sensor::DistFromCenter as usize] = (self.pos.distance(center) / (world_w / 2.0)).clamp(0.0, 1.0);
        s[Sensor::Bias as usize] = 1.0;

        let (brain_steer, brain_speed, attack_dec) = self.brain.think(&s);
        let mut final_steer = brain_steer;
        
        let danger_level = s[Sensor::WallLeft as usize]
            .max(s[Sensor::WallRight as usize])
            .max(s[Sensor::WallAhead as usize])
            .max(1.0 - (dist_up / WALL_SENSE_DIST))
            .max(1.0 - (dist_down / WALL_SENSE_DIST));

        if danger_level > 0.25 {
            let to_center = (center - self.pos).normalize_or_zero();
            let angle_to_center = to_center.y.atan2(to_center.x);
            let diff_to_center = Self::normalize_angle(angle_to_center - self.angle) / std::f32::consts::PI;
            final_steer = (brain_steer * (1.0 - danger_level)) + (diff_to_center * danger_level * WALL_REPULSION_FORCE);
        }

        let control = if self.hit_timer > 0.0 { 0.1 } else { 1.0 };
        self.angle += final_steer.clamp(-1.0, 1.0) * BASE_TURN_SPEED * control;
        
        let target_speed = (brain_speed.max(0.0) * MAX_AGENT_SPEED) * control;
        self.current_speed = self.current_speed * FRICTION_LINEAR + target_speed * (1.0 - FRICTION_LINEAR);

        if attack_dec > 0.5 && self.attack_cooldown <= 0.0 && self.energy > 40.0 {
            let dash_dir = vec2(self.angle.cos(), self.angle.sin());
            self.impulse_vec += dash_dir * ATTACK_DASH_IMPULSE;
            self.attack_cooldown = ATTACK_COOLDOWN_TIME;
            self.energy -= ATTACK_COST;
        }

        let velocity = vec2(self.angle.cos(), self.angle.sin()) * self.current_speed;
        self.pos += (velocity + self.impulse_vec) * (dt * 60.0);
        self.impulse_vec *= FRICTION_IMPULSE;

        if self.pos.x < 15.0 { self.impulse_vec.x += WALL_BOUNCE_ELASTICITY; }
        if self.pos.x > world_w - 15.0 { self.impulse_vec.x -= WALL_BOUNCE_ELASTICITY; }
        if self.pos.y < 15.0 { self.impulse_vec.y += WALL_BOUNCE_ELASTICITY; }
        if self.pos.y > world_h - 15.0 { self.impulse_vec.y -= WALL_BOUNCE_ELASTICITY; }

        self.energy -= ENERGY_LOSS_IDLE;
        if self.current_speed > 0.1 || self.impulse_vec.length() > 0.5 {
            let is_running = self.current_speed > MAX_AGENT_SPEED * 0.8 || self.impulse_vec.length() > 2.0;
            let move_cost = if is_running { ENERGY_LOSS_RUN } else { ENERGY_LOSS_MOVE };
            self.energy -= (self.current_speed / MAX_AGENT_SPEED) * move_cost;
        }
        self.energy -= final_steer.abs() * ENERGY_LOSS_ROTATION;

        self.is_attacking = self.attack_cooldown > (ATTACK_COOLDOWN_TIME - 0.4);
        self.pos.x = self.pos.x.clamp(10.0, world_w - 10.0);
        self.pos.y = self.pos.y.clamp(10.0, world_h - 10.0);
    }

    pub fn draw(&self) {
        let mut color = if self.is_attacking { RED } else if self.hit_timer > 0.0 { WHITE } else { self.color };
        if self.spawn_timer > 0.0 { color.a = 0.5; }
        draw_poly(self.pos.x, self.pos.y, 3, 9.0, self.angle.to_degrees(), color);
        
        let e_ratio = (self.energy / INITIAL_ENERGY).clamp(0.0, 1.0);
        draw_rectangle(self.pos.x - 8.0, self.pos.y - 18.0, 16.0, 2.5, BLACK);
        draw_rectangle(self.pos.x - 8.0, self.pos.y - 18.0, 16.0 * e_ratio, 2.5, if e_ratio > 0.3 { GREEN } else { ORANGE });
    }

    pub fn draw_label(&self, camera: &Camera2D) {
        let screen_pos = camera.world_to_screen(self.pos);
        let gen_text = format!("G{}", self.generation);
        let text_dims = measure_text(&gen_text, None, 14, 1.0);
        draw_text(&gen_text, screen_pos.x - text_dims.width / 2.0, screen_pos.y + 25.0, 14.0, WHITE);
    }
}