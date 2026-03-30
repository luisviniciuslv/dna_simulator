use macroquad::{prelude::*, rand::gen_range};

// --- CONFIGURAÇÕES DO ECOSSISTEMA ---
const INITIAL_AGENTS: usize = 2;
const INITIAL_FOOD_COUNT: usize = 1;
const VISION_RADIUS: f32 = 220.0; 
const WALL_SENSE_DIST: f32 = 45.0; 

// Mecânicas de Sobrevivência e Combate
const FAMILY_PROTECTION_TIME: f32 = 10.0; 
const SPAWN_PROTECTION_TIME: f32 = 5.0;  
const ATTACK_COOLDOWN_TIME: f32 = 1.5;    
const HIT_STUN_TIME: f32 = 0.6;          
const KNOCKBACK_IMPULSE: f32 = 16.0;     

// --- METABOLISMO (Punir o giro inútil) ---
const ENERGY_LOSS_IDLE: f32 = 0.015;     
const ENERGY_LOSS_MOVE: f32 = 0.05;      
const ENERGY_LOSS_RUN: f32 = 0.30;       
const ENERGY_LOSS_ROTATION: f32 = 0.04;  // Aumentado para forçar eficiência
const ATTACK_COST: f32 = 1.2;            

// --- ESTRUTURAS DE IA ---

#[derive(Clone)]
struct Brain {
    weights: Vec<f32>,
}

impl Brain {
    fn new_random() -> Self {
        let mut weights = vec![0.0; 30]; 
        
        weights[1] = 2.5;   // Virar para comida
        weights[3] = 1.5;   // Virar para inimigo
        weights[5] = 1.2;   
        weights[6] = -1.2;  
        weights[19] = 0.6;  
        weights[10] = 1.8;  
        weights[22] = 3.0;  // Atacar agressivamente (Entrada 2 - Inimigo Perto)
        weights[29] = -0.8; 

        for w in weights.iter_mut() {
            if *w == 0.0 {
                *w = gen_range(-0.6, 0.6);
            } else {
                *w += gen_range(-0.2, 0.2);
            }
        }
        Self { weights }
    }

    fn mutate(&self) -> Self {
        let mut new_weights = self.weights.clone();
        for w in new_weights.iter_mut() {
            if gen_range(0.0, 1.0) < 0.15 {
                *w += gen_range(-0.1, 0.1);
            }
        }
        Self { weights: new_weights }
    }

    fn think(&self, inputs: &[f32; 10]) -> (f32, f32, f32) {
        let mut outputs = [0.0; 3];
        for i in 0..3 {
            let mut sum = 0.0;
            for j in 0..10 {
                sum += inputs[j] * self.weights[i * 10 + j];
            }
            outputs[i] = sum.tanh();
        }
        (outputs[0], outputs[1], outputs[2])
    }
}

fn wrap_angle(angle: f32) -> f32 {
    let mut a = angle;
    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; }
    a
}

// --- ENTIDADES ---

struct Agent {
    id: u64,
    parent_id: Option<u64>,
    pos: Vec2,
    angle: f32,
    energy: f32,
    brain: Brain,
    color: Color,
    is_attacking: bool,
    current_speed: f32,
    impulse_vec: Vec2,      
    generation: u32,
    wander_t: f32, 
    family_timer: f32,     
    spawn_timer: f32,      
    attack_cooldown: f32,  
    hit_timer: f32,
    target_offset: f32, // Offset aleatório para quebrar o giro infinito
}

impl Agent {
    fn new(id: u64, parent_id: Option<u64>, pos: Vec2, brain: Brain, color: Color, generation: u32) -> Self {
        Self {
            id,
            parent_id,
            pos,
            angle: gen_range(0.0, std::f32::consts::TAU),
            energy: 120.0,
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
            target_offset: gen_range(-0.1, 0.1), // Diferente para cada indivíduo
        }
    }

    fn update(&mut self, closest_food: Option<Vec2>, closest_enemy: Option<Vec2>, world_w: f32, world_h: f32, dt: f32) {
        if self.family_timer > 0.0 { self.family_timer -= dt; }
        if self.spawn_timer > 0.0 { self.spawn_timer -= dt; }
        if self.attack_cooldown > 0.0 { self.attack_cooldown -= dt; }
        if self.hit_timer > 0.0 { self.hit_timer -= dt; }

        let mut inputs = [0.0; 10];
        let mut target_dist = f32::MAX;
        let mut target_angle_err = 0.0;
        
        if let Some(f) = closest_food {
            let dist = self.pos.distance(f);
            if dist < VISION_RADIUS {
                inputs[0] = 1.0 - (dist / VISION_RADIUS); 
                let rel = f - self.pos;
                let t_angle = rel.y.atan2(rel.x) + self.target_offset; // Quebra de simetria
                inputs[1] = wrap_angle(t_angle - self.angle) / std::f32::consts::PI;
                target_dist = dist;
                target_angle_err = inputs[1].abs();
            }
        }

        if let Some(e) = closest_enemy {
            let dist = self.pos.distance(e);
            if dist < VISION_RADIUS {
                inputs[2] = 1.0 - (dist / VISION_RADIUS);
                let rel = e - self.pos;
                let t_angle = rel.y.atan2(rel.x) + self.target_offset; // Quebra de simetria
                inputs[3] = wrap_angle(t_angle - self.angle) / std::f32::consts::PI;
                if dist < target_dist {
                    target_dist = dist;
                    target_angle_err = inputs[3].abs();
                }
            }
        }

        inputs[4] = (self.energy / 200.0).clamp(0.0, 1.0);
        inputs[5] = (1.0 - (self.pos.x / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        inputs[6] = (1.0 - ((world_w - self.pos.x) / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        inputs[7] = (1.0 - (self.pos.y / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        inputs[8] = (1.0 - ((world_h - self.pos.y) / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        inputs[9] = 1.0; 

        let (steer, speed_input, attack_input) = self.brain.think(&inputs);

        let has_target = inputs[0] > 0.0 || inputs[2] > 0.0;
        let is_near_wall = inputs[5] > 0.0 || inputs[6] > 0.0 || inputs[7] > 0.0 || inputs[8] > 0.0;

        if !has_target && !is_near_wall {
            self.wander_t += 0.05;
            self.angle += (self.wander_t.sin() * 0.04) + gen_range(-0.01, 0.01);
            self.current_speed = self.current_speed * 0.94 + 0.6 * 0.06; 
        } else {
            // FLUIDEZ SEM ÓRBITA: Redução de velocidade baseada no erro, mas nunca para.
            let alignment_factor = (1.0 - target_angle_err).powi(2).clamp(0.4, 1.0);
            let mut target_multiplier = 0.6 + (alignment_factor * 1.4); 
            
            if has_target && target_dist < 30.0 { target_multiplier *= 0.7; }

            // Se estiver em cooldown de ataque ou foi atingido, o giro é mais lento (recuperação)
            let turn_speed = if self.hit_timer > 0.0 { 0.1 } else { 0.3 };
            self.angle += steer * turn_speed; 
            
            let mut target_speed = if speed_input > -0.3 { ((speed_input + 0.3) / 1.3) * 1.5 } else { 0.0 };
            target_speed *= target_multiplier;

            // Bote Decisivo: Se estiver minimamente alinhado e decidir atacar, ele VAI.
            if attack_input > 0.4 && self.attack_cooldown <= 0.0 && target_angle_err < 0.2 {
                let dash_dir = vec2(self.angle.cos(), self.angle.sin());
                self.impulse_vec += dash_dir * 7.0; // Bote mais forte para atravessar o alvo
                self.attack_cooldown = ATTACK_COOLDOWN_TIME;
                self.energy -= ATTACK_COST;
            }

            self.current_speed = self.current_speed * 0.85 + target_speed * 0.15;
        }

        let move_vec = vec2(self.angle.cos(), self.angle.sin()) * self.current_speed;
        self.pos += move_vec + self.impulse_vec;
        self.impulse_vec *= 0.82; 

        // Metabolismo
        self.energy -= ENERGY_LOSS_IDLE;
        if self.current_speed > 0.1 {
            let move_cost = if self.current_speed > 1.4 || self.impulse_vec.length() > 1.0 { ENERGY_LOSS_RUN } else { ENERGY_LOSS_MOVE };
            self.energy -= (self.current_speed / 1.5) * move_cost;
        }
        self.energy -= steer.abs() * ENERGY_LOSS_ROTATION;
        
        if is_near_wall && self.current_speed > 0.4 { self.energy -= 0.05; }

        // Estado visual do ataque: dura metade do cooldown para ser claro
        self.is_attacking = self.attack_cooldown > (ATTACK_COOLDOWN_TIME - 0.6) && self.spawn_timer <= 0.0; 

        self.pos.x = self.pos.x.clamp(8.0, world_w - 8.0);
        self.pos.y = self.pos.y.clamp(8.0, world_h - 8.0);
    }

    fn draw_body(&self) {
        let mut draw_color = if self.is_attacking { RED } else { self.color };
        
        if self.spawn_timer > 0.0 || self.family_timer > 0.0 {
            let timer = if self.spawn_timer > 0.0 { self.spawn_timer } else { self.family_timer };
            draw_color.a = 0.5 + (timer * 4.0).sin() * 0.2;
            draw_circle_lines(self.pos.x, self.pos.y, 11.0, 1.0, SKYBLUE);
        }

        if self.hit_timer > 0.0 { draw_color.a = 0.3; }
        draw_poly(self.pos.x, self.pos.y, 3, 8.0, self.angle.to_degrees(), draw_color);
        
        draw_rectangle(self.pos.x - 8.0, self.pos.y - 14.0, 16.0, 3.0, Color::new(0.2, 0.2, 0.2, 0.6));
        let energy_ratio = (self.energy / 120.0).clamp(0.0, 1.0);
        draw_rectangle(self.pos.x - 8.0, self.pos.y - 14.0, 16.0 * energy_ratio, 3.0, if energy_ratio > 0.3 { GREEN } else { ORANGE });
    }

    fn draw_label(&self, camera: &Camera2D) {
        let screen_pos = camera.world_to_screen(self.pos);
        let gen_text = format!("G{}", self.generation);
        let text_dims = measure_text(&gen_text, None, 14, 1.0);
        draw_text(&gen_text, screen_pos.x - text_dims.width / 2.0, screen_pos.y + 30.0, 14.0, WHITE);
    }
}

// --- MAIN ---

#[macroquad::main("Ecossistema: Fim do Giro Infinito")]
async fn main() {
    let mut world_w = screen_width();
    let mut world_h = screen_height();
    let mut id_generator: u64 = 0;

    let mut agents: Vec<Agent> = (0..INITIAL_AGENTS)
        .map(|_| {
            id_generator += 1;
            Agent::new(id_generator, None, vec2(gen_range(100.0, world_w-100.0), gen_range(100.0, world_h-100.0)), Brain::new_random(), Color::new(gen_range(0.3, 0.6), gen_range(0.4, 0.8), 1.0, 1.0), 1)
        })
        .collect();

    let mut foods: Vec<Vec2> = (0..INITIAL_FOOD_COUNT).map(|_| vec2(gen_range(50.0, world_w-50.0), gen_range(50.0, world_h-50.0))).collect();
    let mut max_food = INITIAL_FOOD_COUNT;
    let mut zoom: f32 = 1.0;
    let mut camera_pos = vec2(world_w / 2.0, world_h / 2.0);

    loop {
        world_w = screen_width();
        world_h = screen_height();
        let dt = get_frame_time();

        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            let mut camera = Camera2D { target: camera_pos, zoom: vec2(zoom / world_w * 2.0, -zoom / world_h * 2.0), ..Default::default() };
            let mouse_world_before = camera.screen_to_world(mouse_position().into());
            let zoom_speed: f32 = 1.1; 
            if wheel_y > 0.0 { zoom *= zoom_speed; } else { zoom /= zoom_speed; }
            zoom = zoom.clamp(1.0, 10.0);
            if zoom <= 1.001 { camera_pos = vec2(world_w / 2.0, world_h / 2.0); } 
            else {
                camera.zoom = vec2(zoom / world_w * 2.0, -zoom / world_h * 2.0);
                let mouse_world_after = camera.screen_to_world(mouse_position().into());
                camera_pos += mouse_world_before - mouse_world_after;
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let m_pos = mouse_position();
            if m_pos.0 > world_w - 50.0 && m_pos.0 < world_w - 10.0 && m_pos.1 > 10.0 && m_pos.1 < 50.0 { max_food += 1; }
            if m_pos.0 > world_w - 100.0 && m_pos.0 < world_w - 60.0 && m_pos.1 > 10.0 && m_pos.1 < 50.0 { if max_food > 0 { max_food -= 1; } }
        }

        if foods.len() < max_food && gen_range(0, 30) == 0 { foods.push(vec2(gen_range(30.0, world_w-30.0), gen_range(30.0, world_h-30.0))); }

        let mut new_agents = Vec::new();

        // --- LÓGICA DE INTERAÇÃO (DANO GARANTIDO) ---
        for i in 0..agents.len() {
            foods.retain(|&f| {
                if agents[i].pos.distance(f) < 20.0 {
                    agents[i].energy += 95.0; 
                    false
                } else { true }
            });

            if agents[i].is_attacking {
                for j in 0..agents.len() {
                    if i == j { continue; }
                    if agents[j].spawn_timer > 0.0 || agents[j].hit_timer > 0.0 { continue; }
                    
                    let are_family = agents[i].parent_id == Some(agents[j].id) || agents[j].parent_id == Some(agents[i].id);
                    if are_family && (agents[i].family_timer > 0.0 || agents[j].family_timer > 0.0) { continue; }

                    let dist = agents[i].pos.distance(agents[j].pos);
                    // Raio de acerto generoso durante o bote para garantir que a agressividade funcione
                    if dist < 28.0 {
                        agents[j].energy -= 40.0; 
                        agents[i].energy += 25.0; 
                        agents[j].hit_timer = HIT_STUN_TIME; 
                        let push_dir = (agents[j].pos - agents[i].pos).normalize_or_zero();
                        agents[j].impulse_vec += push_dir * KNOCKBACK_IMPULSE;
                        agents[i].impulse_vec -= push_dir * (KNOCKBACK_IMPULSE * 0.3); 
                    }
                }
            }
        }

        for i in 0..agents.len() {
            let closest_f = foods.iter().filter(|&&f| agents[i].pos.distance(f) < VISION_RADIUS).min_by(|a, b| agents[i].pos.distance(**a).partial_cmp(&agents[i].pos.distance(**b)).unwrap()).cloned();
            let mut closest_e = None;
            let mut min_dist_e = VISION_RADIUS;
            for j in 0..agents.len() {
                if i == j { continue; }
                let are_family = agents[i].parent_id == Some(agents[j].id) || agents[j].parent_id == Some(agents[i].id);
                if are_family && (agents[i].family_timer > 0.0 || agents[j].family_timer > 0.0) { continue; }
                let d = agents[i].pos.distance(agents[j].pos);
                if d < min_dist_e { min_dist_e = d; closest_e = Some(agents[j].pos); }
            }

            let agent = &mut agents[i];
            agent.update(closest_f, closest_e, world_w, world_h, dt);

            if agent.energy > 300.0 {
                agent.energy -= 150.0; id_generator += 1;
                new_agents.push(Agent::new(id_generator, Some(agent.id), agent.pos, agent.brain.mutate(), agent.color, agent.generation + 1));
            }
        }
        agents.retain(|a| a.energy > 0.0);
        agents.extend(new_agents);

        clear_background(Color::new(0.01, 0.01, 0.02, 1.0));
        let camera = Camera2D { target: camera_pos, zoom: vec2(zoom / world_w * 2.0, -zoom / world_h * 2.0), ..Default::default() };
        set_camera(&camera);
        draw_rectangle_lines(0.0, 0.0, world_w, world_h, 3.0, Color::new(0.3, 0.3, 0.7, 0.3));

        for food in &foods { draw_circle(food.x, food.y, 4.0, GREEN); draw_circle_lines(food.x, food.y, 6.0, 1.0, Color::new(0.0, 1.0, 0.0, 0.2)); }
        for agent in &agents { agent.draw_body(); }

        set_default_camera();
        for agent in &agents { agent.draw_label(&camera); }

        draw_rectangle(10.0, 10.0, 300.0, 50.0, Color::new(0.0, 0.0, 0.0, 0.8));
        draw_text(&format!("POPULAÇÃO: {} | RECURSOS: {}/{}", agents.len(), foods.len(), max_food), 20.0, 40.0, 20.0, WHITE);
        draw_rectangle(world_w - 110.0, 10.0, 45.0, 45.0, Color::new(0.3, 0.1, 0.1, 1.0)); draw_text("-", world_w - 92.0, 42.0, 30.0, WHITE);
        draw_rectangle(world_w - 55.0, 10.0, 45.0, 45.0, Color::new(0.1, 0.3, 0.1, 1.0)); draw_text("+", world_w - 40.0, 42.0, 30.0, WHITE);

        if agents.is_empty() { draw_text("EXTINÇÃO", world_w / 2.0 - 80.0, world_h / 2.0, 40.0, RED); }
        next_frame().await
    }
}