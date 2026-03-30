use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::brain::{Brain, INPUT_SIZE};
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
    pub wander_t: f32,
    pub family_timer: f32,
    pub spawn_timer: f32,
    pub attack_cooldown: f32,
    pub hit_timer: f32,
    pub target_offset: f32,
}

impl Agent {
    pub fn new(id: u64, parent_id: Option<u64>, pos: Vec2, brain: Brain, color: Color, generation: u32) -> Self {
        Self {
            id,
            parent_id,
            pos,
            angle: gen_range(0.0, std::f32::consts::TAU),
            energy: INITIAL_ENERGY,
            brain,
            color,
            is_attacking: false,
            current_speed: 0.0,
            impulse_vec: Vec2::ZERO,
            generation,
            wander_t: gen_range(0.0, 100.0),
            family_timer: FAMILY_PROTECTION_TIME,
            spawn_timer: SPAWN_PROTECTION_TIME,
            attack_cooldown: 0.0,
            hit_timer: 0.0,
            target_offset: gen_range(-0.1, 0.1),
        }
    }

    fn wrap_angle(angle: f32) -> f32 {
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

        let mut inputs = [0.0; INPUT_SIZE];
        let mut target_dist = f32::MAX;
        let mut target_angle_err = 0.0;
        
        // Sensor de Comida
        if let Some(f) = closest_food {
            let dist = self.pos.distance(f);
            if dist < VISION_RADIUS {
                inputs[0] = 1.0 - (dist / VISION_RADIUS); 
                let t_angle = (f - self.pos).y.atan2((f - self.pos).x) + self.target_offset;
                inputs[1] = Self::wrap_angle(t_angle - self.angle) / std::f32::consts::PI;
                target_dist = dist;
                target_angle_err = inputs[1].abs();
            }
        }

        // Sensor de Inimigo
        if let Some(e) = closest_enemy {
            let dist = self.pos.distance(e);
            if dist < VISION_RADIUS {
                inputs[2] = 1.0 - (dist / VISION_RADIUS);
                let t_angle = (e - self.pos).y.atan2((e - self.pos).x) + self.target_offset;
                inputs[3] = Self::wrap_angle(t_angle - self.angle) / std::f32::consts::PI;
                if dist < target_dist {
                    target_dist = dist;
                    target_angle_err = inputs[3].abs();
                }
            }
        }

        // Sensores Internos e de Parede
        inputs[4] = (self.energy / (REPRODUCTION_THRESHOLD * 0.7)).clamp(0.0, 1.0);
        inputs[5] = (1.0 - (self.pos.x / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        inputs[6] = (1.0 - ((world_w - self.pos.x) / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        inputs[7] = (1.0 - (self.pos.y / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        inputs[8] = (1.0 - ((world_h - self.pos.y) / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        inputs[9] = 1.0; // Bias

        let (steer, speed_input, attack_input) = self.brain.think(&inputs);
        let has_target = inputs[0] > 0.0 || inputs[2] > 0.0;
        let is_near_wall = inputs[5] > 0.0 || inputs[6] > 0.0 || inputs[7] > 0.0 || inputs[8] > 0.0;

        // Lógica de Movimento
        if !has_target && !is_near_wall {
            self.wander_t += 0.05;
            self.angle += (self.wander_t.sin() * 0.04) + gen_range(-0.01, 0.01);
            self.current_speed = self.current_speed * FRICTION_LINEAR + WANDER_SPEED * (1.0 - FRICTION_LINEAR); 
        } else {
            let alignment_factor = (1.0 - target_angle_err).powi(2).clamp(0.4, 1.0);
            let mut target_multiplier = 0.6 + (alignment_factor * 1.4); 
            if has_target && target_dist < 30.0 { target_multiplier *= 0.7; }

            let turn_speed = if self.hit_timer > 0.0 { HIT_TURN_SPEED } else { BASE_TURN_SPEED };
            self.angle += steer * turn_speed; 
            
            let mut target_speed = if speed_input > -0.3 { ((speed_input + 0.3) / 1.3) * MAX_AGENT_SPEED } else { 0.0 };
            target_speed *= target_multiplier;

            // Decisão de Ataque (Bote)
            if attack_input > 0.4 && self.attack_cooldown <= 0.0 && target_angle_err < 0.2 {
                let dash_dir = vec2(self.angle.cos(), self.angle.sin());
                self.impulse_vec += dash_dir * ATTACK_DASH_IMPULSE;
                self.attack_cooldown = ATTACK_COOLDOWN_TIME;
                self.energy -= ATTACK_COST;
            }
            self.current_speed = self.current_speed * 0.85 + target_speed * 0.15;
        }

        // Aplicação de física
        let move_vec = vec2(self.angle.cos(), self.angle.sin()) * self.current_speed;
        self.pos += move_vec + self.impulse_vec;
        self.impulse_vec *= FRICTION_IMPULSE; 

        // Consumo de Energia
        self.energy -= ENERGY_LOSS_IDLE;
        if self.current_speed > 0.1 {
            let move_cost = if self.current_speed > 1.4 || self.impulse_vec.length() > 1.0 { ENERGY_LOSS_RUN } else { ENERGY_LOSS_MOVE };
            self.energy -= (self.current_speed / MAX_AGENT_SPEED) * move_cost;
        }
        self.energy -= steer.abs() * ENERGY_LOSS_ROTATION;
        if is_near_wall && self.current_speed > 0.4 { self.energy -= WALL_BUMP_PENALTY; }

        self.is_attacking = self.attack_cooldown > (ATTACK_COOLDOWN_TIME - ATTACK_VISUAL_DURATION) && self.spawn_timer <= 0.0; 

        self.pos.x = self.pos.x.clamp(8.0, world_w - 8.0);
        self.pos.y = self.pos.y.clamp(8.0, world_h - 8.0);
    }

    pub fn draw_body(&self) {
        let mut draw_color = if self.is_attacking { COLOR_ATTACK } else { self.color };
        
        if self.spawn_timer > 0.0 || self.family_timer > 0.0 {
            let timer = if self.spawn_timer > 0.0 { self.spawn_timer } else { self.family_timer };
            draw_color.a = 0.5 + (timer * 4.0).sin() * 0.2;
            draw_circle_lines(self.pos.x, self.pos.y, 11.0, 1.0, SKYBLUE);
        }

        if self.hit_timer > 0.0 { draw_color.a = 0.3; }
        draw_poly(self.pos.x, self.pos.y, 3, 8.0, self.angle.to_degrees(), draw_color);
        
        // Barra de Energia
        draw_rectangle(self.pos.x - 8.0, self.pos.y - 14.0, 16.0, 3.0, Color::new(0.2, 0.2, 0.2, 0.6));
        let energy_ratio = (self.energy / INITIAL_ENERGY).clamp(0.0, 1.0);
        draw_rectangle(self.pos.x - 8.0, self.pos.y - 14.0, 16.0 * energy_ratio, 3.0, if energy_ratio > 0.3 { GREEN } else { ORANGE });
    }

    pub fn draw_label(&self, camera: &Camera2D) {
        let screen_pos = camera.world_to_screen(self.pos);
        let gen_text = format!("G{}", self.generation);
        let text_dims = measure_text(&gen_text, None, 14, 1.0);
        draw_text(&gen_text, screen_pos.x - text_dims.width / 2.0, screen_pos.y + 30.0, 14.0, WHITE);
    }
}