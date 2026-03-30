use macroquad::{prelude::*, rand::gen_range};

// --- CONFIGURAÇÕES DO ECOSSISTEMA ---
const INITIAL_AGENTS: usize = 12;
const INITIAL_FOOD_COUNT: usize = 10;
const VISION_RADIUS: f32 = 160.0; 
const WALL_SENSE_DIST: f32 = 45.0; 

// Mecânicas de Sobrevivência e Combate
const FAMILY_PROTECTION_TIME: f32 = 10.0; // 10 segundos sem fogo amigo familiar
const SPAWN_PROTECTION_TIME: f32 = 5.0;  // Proteção geral inicial
const ATTACK_COOLDOWN_TIME: f32 = 0.8;    // Recarga do bote
const KNOCKBACK_FORCE: f32 = 12.0;       // Força de recuo suavizada (evita teleporte)

// Custos de energia
const ENERGY_LOSS_IDLE: f32 = 0.004;
const ENERGY_LOSS_MOVE: f32 = 0.015;
const ENERGY_LOSS_RUN: f32 = 0.08;
const ATTACK_COST: f32 = 0.6; 

// --- ESTRUTURAS DE IA ---

#[derive(Clone)]
struct Brain {
    // 10 entradas: [Vê_Comida, Ang_Comida, Vê_Inimigo, Ang_Inimigo, Energia, Parede_Esq, Parede_Dir, Parede_Topo, Parede_Fundo, Bias]
    // 3 saídas: [Giro, Velocidade, Ataque]
    weights: Vec<f32>,
}

impl Brain {
    fn new_random() -> Self {
        let mut weights = vec![0.0; 30]; 
        
        // Saída 0: Giro
        weights[1] = 1.4;   // Virar para comida
        weights[3] = 0.8;   // Virar para inimigo
        weights[5] = 1.0;   // Fugir da parede esquerda
        weights[6] = -1.0;  // Fugir da parede direita
        
        // Saída 1: Velocidade
        weights[19] = 0.4;  // Impulso base (Bias)
        weights[10] = 1.0;  // Acelerar se ver comida
        weights[14] = -0.5; // Desacelerar se energia estiver baixa (Input 4)
        
        // Saída 2: Ataque
        weights[22] = 2.0;  // Atacar se inimigo estiver perto
        weights[24] = 1.2;  // Atacar mais se tiver muita energia (Input 4)
        weights[29] = -0.8; // Inibição natural de ataque (Bias)

        for w in weights.iter_mut() {
            if *w == 0.0 {
                *w = gen_range(-0.5, 0.5);
            } else {
                *w += gen_range(-0.2, 0.2);
            }
        }
        Self { weights }
    }

    fn mutate(&self) -> Self {
        let mut new_weights = self.weights.clone();
        for w in new_weights.iter_mut() {
            if gen_range(0.0, 1.0) < 0.18 {
                *w += gen_range(-0.12, 0.12);
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
    generation: u32,
    wander_t: f32, 
    family_timer: f32,     // Proteção entre pais e filhos
    attack_cooldown: f32,  // Recarga do bote
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
            generation,
            wander_t: gen_range(0.0, 100.0),
            family_timer: FAMILY_PROTECTION_TIME,
            attack_cooldown: 0.0,
        }
    }

    fn update(&mut self, closest_food: Option<Vec2>, closest_enemy: Option<Vec2>, world_w: f32, world_h: f32, dt: f32) {
        // Atualizar timers
        if self.family_timer > 0.0 { self.family_timer -= dt; }
        if self.attack_cooldown > 0.0 { self.attack_cooldown -= dt; }

        let mut inputs = [0.0; 10];
        
        // [0, 1] Percepção de Comida
        if let Some(f) = closest_food {
            let dist = self.pos.distance(f);
            if dist < VISION_RADIUS {
                inputs[0] = 1.0 - (dist / VISION_RADIUS); 
                let rel = f - self.pos;
                let target_angle = rel.y.atan2(rel.x);
                inputs[1] = wrap_angle(target_angle - self.angle) / std::f32::consts::PI;
            }
        }

        // [2, 3] Percepção de Inimigo
        if let Some(e) = closest_enemy {
            let dist = self.pos.distance(e);
            if dist < VISION_RADIUS {
                inputs[2] = 1.0 - (dist / VISION_RADIUS);
                let rel = e - self.pos;
                let target_angle = rel.y.atan2(rel.x);
                inputs[3] = wrap_angle(target_angle - self.angle) / std::f32::consts::PI;
            }
        }

        inputs[4] = (self.energy / 200.0).clamp(0.0, 1.0);
        inputs[5] = (1.0 - (self.pos.x / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        inputs[6] = (1.0 - ((world_w - self.pos.x) / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        inputs[7] = (1.0 - (self.pos.y / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        inputs[8] = (1.0 - ((world_h - self.pos.y) / WALL_SENSE_DIST)).clamp(0.0, 1.0);
        inputs[9] = 1.0; 

        let (steer, speed_input, attack_input) = self.brain.think(&inputs);

        let is_searching = inputs[0] == 0.0 && inputs[2] == 0.0;
        let is_near_wall = inputs[5] > 0.0 || inputs[6] > 0.0 || inputs[7] > 0.0 || inputs[8] > 0.0;

        if is_searching && !is_near_wall {
            self.wander_t += 0.05;
            self.angle += (self.wander_t.sin() * 0.04) + gen_range(-0.01, 0.01);
            self.current_speed = self.current_speed * 0.96 + 0.35 * 0.04; 
        } else {
            self.angle += steer * 0.12; 
            let mut target_speed = if speed_input > -0.3 { ((speed_input + 0.3) / 1.3) * 1.3 } else { 0.0 };
            
            // "O Bote" suavizado - multiplicador menor para evitar teleporte visual
            if attack_input > 0.4 && self.attack_cooldown <= 0.0 {
                target_speed *= 1.6;
            }

            // Interpolação de velocidade (Inércia)
            self.current_speed = self.current_speed * 0.85 + target_speed * 0.15;
        }

        let velocity = vec2(self.angle.cos(), self.angle.sin()) * self.current_speed;
        self.pos += velocity;

        self.energy -= ENERGY_LOSS_IDLE;
        self.energy -= (self.current_speed / 1.3) * ENERGY_LOSS_RUN;
        
        if is_near_wall && self.current_speed > 0.4 {
            self.energy -= 0.03;
        }

        // Ativação do ataque - impede ataque em cooldown ou durante proteção familiar inicial
        self.is_attacking = attack_input > 0.4 && self.attack_cooldown <= 0.0; 
        if self.is_attacking {
            self.energy -= ATTACK_COST;
        }

        // Limites físicos
        self.pos.x = self.pos.x.clamp(8.0, world_w - 8.0);
        self.pos.y = self.pos.y.clamp(8.0, world_h - 8.0);
    }

    fn draw_body(&self) {
        let mut draw_color = if self.is_attacking { RED } else { self.color };
        
        // Efeito visual de proteção familiar (brilho suave)
        if self.family_timer > 0.0 {
            draw_color.a = 0.6 + (self.family_timer * 3.0).sin() * 0.2;
            draw_circle_lines(self.pos.x, self.pos.y, 10.0, 1.0, SKYBLUE);
        }

        draw_poly(self.pos.x, self.pos.y, 3, 8.0, self.angle.to_degrees(), draw_color);
        
        draw_rectangle(self.pos.x - 8.0, self.pos.y - 14.0, 16.0, 3.0, Color::new(0.2, 0.2, 0.2, 0.6));
        let energy_ratio = (self.energy / 120.0).clamp(0.0, 1.0);
        draw_rectangle(self.pos.x - 8.0, self.pos.y - 14.0, 16.0 * energy_ratio, 3.0, if energy_ratio > 0.3 { GREEN } else { ORANGE });
    }

    fn draw_label(&self, camera: &Camera2D) {
        let screen_pos = camera.world_to_screen(self.pos);
        let gen_text = format!("G{}", self.generation);
        let font_size = 14.0;
        let text_dims = measure_text(&gen_text, None, font_size as u16, 1.0);
        
        draw_text(
            &gen_text, 
            screen_pos.x - text_dims.width / 2.0, 
            screen_pos.y + 25.0, 
            font_size, 
            WHITE
        );
    }
}

// --- MAIN ---

#[macroquad::main("Ecossistema: Sobrevivência Familiar")]
async fn main() {
    let mut world_w = screen_width();
    let mut world_h = screen_height();
    let mut id_counter: u64 = 0;

    let mut agents: Vec<Agent> = (0..INITIAL_AGENTS)
        .map(|_| {
            id_counter += 1;
            Agent::new(
                id_counter,
                None,
                vec2(gen_range(100.0, world_w-100.0), gen_range(100.0, world_h-100.0)),
                Brain::new_random(),
                Color::new(gen_range(0.3, 0.6), gen_range(0.4, 0.8), 1.0, 1.0),
                1
            )
        })
        .collect();

    let mut foods: Vec<Vec2> = (0..INITIAL_FOOD_COUNT)
        .map(|_| vec2(gen_range(50.0, world_w-50.0), gen_range(50.0, world_h-50.0)))
        .collect();

    let mut max_food = INITIAL_FOOD_COUNT;
    let mut zoom: f32 = 1.0;
    let mut camera_pos = vec2(world_w / 2.0, world_h / 2.0);

    loop {
        world_w = screen_width();
        world_h = screen_height();
        let dt = get_frame_time();

        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            let mut camera = Camera2D {
                target: camera_pos,
                zoom: vec2(zoom / world_w * 2.0, -zoom / world_h * 2.0),
                ..Default::default()
            };
            let mouse_world_before = camera.screen_to_world(mouse_position().into());
            let zoom_speed: f32 = 1.1; 
            if wheel_y > 0.0 { zoom *= zoom_speed; } else { zoom /= zoom_speed; }
            zoom = zoom.clamp(1.0, 10.0);

            if zoom <= 1.001 {
                camera_pos = vec2(world_w / 2.0, world_h / 2.0);
            } else {
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

        if foods.len() < max_food && gen_range(0, 30) == 0 {
            foods.push(vec2(gen_range(30.0, world_w-30.0), gen_range(30.0, world_h-30.0)));
        }

        let mut new_agents = Vec::new();

        // --- LÓGICA DE INTERAÇÃO (ALIMENTAÇÃO E COMBATE) ---
        for i in 0..agents.len() {
            // Alimentação
            foods.retain(|&f| {
                if agents[i].pos.distance(f) < 16.0 {
                    agents[i].energy += 85.0; 
                    false
                } else { true }
            });

            // Combate com Proteção Familiar
            if agents[i].is_attacking && agents[i].attack_cooldown <= 0.0 {
                let mut hit_detected = false;
                for j in 0..agents.len() {
                    if i == j { continue; }
                    
                    // MECÂNICA GENIAL: Checagem de parentesco e tempo de proteção
                    let is_parent_child = agents[i].parent_id == Some(agents[j].id) || agents[j].parent_id == Some(agents[i].id);
                    let family_protected = is_parent_child && (agents[i].family_timer > 0.0 || agents[j].family_timer > 0.0);

                    if family_protected { continue; }

                    let dist = agents[i].pos.distance(agents[j].pos);
                    if dist < 22.0 {
                        // Dano e Recompensa
                        agents[j].energy -= 18.0;
                        agents[i].energy += 12.0; 
                        
                        // Recuo Suavizado (Knockback)
                        let push_dir = (agents[j].pos - agents[i].pos).normalize();
                        agents[j].pos += push_dir * KNOCKBACK_FORCE;
                        
                        hit_detected = true;
                    }
                }
                
                if hit_detected {
                    agents[i].attack_cooldown = ATTACK_COOLDOWN_TIME;
                }
            }
        }

        for i in 0..agents.len() {
            let closest_f = foods.iter()
                .filter(|&&f| agents[i].pos.distance(f) < VISION_RADIUS)
                .min_by(|a, b| agents[i].pos.distance(**a).partial_cmp(&agents[i].pos.distance(**b)).unwrap())
                .cloned();

            let mut closest_e = None;
            let mut min_dist_e = VISION_RADIUS;
            for j in 0..agents.len() {
                if i == j { continue; }
                let d = agents[i].pos.distance(agents[j].pos);
                if d < min_dist_e {
                    min_dist_e = d;
                    closest_e = Some(agents[j].pos);
                }
            }

            let agent = &mut agents[i];
            agent.update(closest_f, closest_e, world_w, world_h, dt);

            if agent.energy > 290.0 {
                agent.energy -= 145.0;
                id_counter += 1;
                new_agents.push(Agent::new(
                    id_counter, 
                    Some(agent.id), 
                    agent.pos, 
                    agent.brain.mutate(), 
                    agent.color, 
                    agent.generation + 1
                ));
            }
        }
        agents.retain(|a| a.energy > 0.0);
        agents.extend(new_agents);

        clear_background(Color::new(0.01, 0.01, 0.02, 1.0));

        let camera = Camera2D {
            target: camera_pos,
            zoom: vec2(zoom / world_w * 2.0, -zoom / world_h * 2.0),
            ..Default::default()
        };
        
        set_camera(&camera);
        draw_rectangle_lines(0.0, 0.0, world_w, world_h, 3.0, Color::new(0.3, 0.3, 0.7, 0.3));

        for food in &foods {
            draw_circle(food.x, food.y, 4.0, GREEN);
            draw_circle_lines(food.x, food.y, 6.0, 1.0, Color::new(0.0, 1.0, 0.0, 0.2));
        }
        for agent in &agents {
            agent.draw_body();
        }

        set_default_camera();
        for agent in &agents {
            agent.draw_label(&camera);
        }

        draw_rectangle(10.0, 10.0, 300.0, 50.0, Color::new(0.0, 0.0, 0.0, 0.8));
        draw_text(&format!("POPULAÇÃO: {} | RECURSOS: {}/{}", agents.len(), foods.len(), max_food), 20.0, 40.0, 20.0, WHITE);
        
        draw_rectangle(world_w - 110.0, 10.0, 45.0, 45.0, Color::new(0.3, 0.1, 0.1, 1.0));
        draw_text("-", world_w - 92.0, 42.0, 30.0, WHITE);
        draw_rectangle(world_w - 55.0, 10.0, 45.0, 45.0, Color::new(0.1, 0.3, 0.1, 1.0));
        draw_text("+", world_w - 40.0, 42.0, 30.0, WHITE);

        if agents.is_empty() {
            draw_text("EXTINÇÃO", world_w / 2.0 - 80.0, world_h / 2.0, 40.0, RED);
        }

        next_frame().await
    }
}