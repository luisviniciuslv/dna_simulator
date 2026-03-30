use macroquad::{prelude::*, rand::gen_range};

// --- CONFIGURAÇÕES ---
const INITIAL_AGENTS: usize = 8;
const INITIAL_FOOD_COUNT: usize = 6;

// Custos balanceados
const ENERGY_LOSS_IDLE: f32 = 0.003;
const ENERGY_LOSS_MOVE: f32 = 0.08;
const ENERGY_LOSS_RUN: f32 = 0.2;
const ATTACK_COST: f32 = 0.3;

// --- ESTRUTURAS DE IA ---

#[derive(Clone)]
struct Brain {
    weights: Vec<f32>,
}

impl Brain {
    fn new_random() -> Self {
        let mut weights = vec![0.0; 18];
        weights[1] = 0.8;  // Instinto de virar para a comida
        weights[11] = 0.4; // Impulso de movimento
        weights[17] = -1.2; // Tendência a não atacar

        for w in weights.iter_mut() {
            *w += gen_range(-0.2, 0.2);
        }
        Self { weights }
    }

    fn mutate(&self) -> Self {
        let mut new_weights = self.weights.clone();
        for w in new_weights.iter_mut() {
            if gen_range(0.0, 1.0) < 0.2 {
                *w += gen_range(-0.15, 0.15);
            }
        }
        Self { weights: new_weights }
    }

    fn think(&self, inputs: &[f32; 6]) -> (f32, f32, f32) {
        let mut outputs = [0.0; 3];
        for i in 0..3 {
            let mut sum = 0.0;
            for j in 0..6 {
                sum += inputs[j] * self.weights[i * 6 + j];
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
    pos: Vec2,
    angle: f32,
    energy: f32,
    brain: Brain,
    color: Color,
    is_attacking: bool,
    current_speed: f32,
    generation: u32,
}

impl Agent {
    fn new(pos: Vec2, brain: Brain, color: Color, generation: u32) -> Self {
        Self {
            pos,
            angle: gen_range(0.0, std::f32::consts::TAU),
            energy: 120.0,
            brain,
            color,
            is_attacking: false,
            current_speed: 0.0,
            generation,
        }
    }

    fn update(&mut self, closest_food: Option<Vec2>, closest_enemy: Option<Vec2>, world_w: f32, world_h: f32) {
        let food_input_dist;
        let food_input_angle;
        if let Some(f) = closest_food {
            let rel = f - self.pos;
            food_input_dist = (1.0 - (rel.length() / world_w)).clamp(0.0, 1.0);
            let target_angle = rel.y.atan2(rel.x);
            food_input_angle = wrap_angle(target_angle - self.angle) / std::f32::consts::PI;
        } else {
            food_input_dist = 0.0;
            food_input_angle = 0.0;
        }

        let enemy_input_dist;
        let enemy_input_angle;
        if let Some(e) = closest_enemy {
            let rel = e - self.pos;
            enemy_input_dist = (1.0 - (rel.length() / world_w)).clamp(0.0, 1.0);
            let target_angle = rel.y.atan2(rel.x);
            enemy_input_angle = wrap_angle(target_angle - self.angle) / std::f32::consts::PI;
        } else {
            enemy_input_dist = 0.0;
            enemy_input_angle = 0.0;
        }

        let inputs = [
            food_input_dist,
            food_input_angle,
            enemy_input_dist,
            enemy_input_angle,
            (self.energy / 200.0).clamp(0.0, 1.0),
            1.0 
        ];

        let (steer, speed_input, attack_input) = self.brain.think(&inputs);

        self.angle += gen_range(-0.04, 0.04); 
        self.angle += steer * 0.08; 
        
        let target_speed = if speed_input > -0.4 {
            ((speed_input + 0.4) / 1.4) * 1.4
        } else {
            0.0
        };

        self.current_speed = self.current_speed * 0.9 + target_speed * 0.1;
        let velocity = vec2(self.angle.cos(), self.angle.sin()) * self.current_speed;
        self.pos += velocity;

        self.energy -= ENERGY_LOSS_IDLE;
        if self.current_speed > 0.1 {
            self.energy -= if self.current_speed > 1.0 { ENERGY_LOSS_RUN } else { ENERGY_LOSS_MOVE };
        }
        
        self.is_attacking = attack_input > 0.6;
        if self.is_attacking {
            self.energy -= ATTACK_COST;
        }

        self.pos.x = self.pos.x.clamp(10.0, world_w - 10.0);
        self.pos.y = self.pos.y.clamp(10.0, world_h - 10.0);
    }

    fn draw_body(&self) {
        let draw_color = if self.is_attacking { RED } else { self.color };
        
        // Desenha o triângulo (corpo)
        draw_poly(self.pos.x, self.pos.y, 3, 7.0, self.angle.to_degrees(), draw_color);
        
        // Barra de energia (seguindo o agente no mundo)
        draw_rectangle(self.pos.x - 6.0, self.pos.y - 12.0, 12.0, 2.0, Color::new(0.2, 0.2, 0.2, 0.5));
        let energy_ratio = (self.energy / 100.0).clamp(0.0, 1.0);
        draw_rectangle(self.pos.x - 6.0, self.pos.y - 12.0, 12.0 * energy_ratio, 2.0, GREEN);
    }

    // Função separada para desenhar o texto no espaço do ecrã (evita espelhamento)
    fn draw_label(&self, camera: &Camera2D) {
        let screen_pos = camera.world_to_screen(self.pos);
        let gen_text = format!("G{}", self.generation);
        let font_size = 14.0;
        let text_dims = measure_text(&gen_text, None, font_size as u16, 1.0);
        
        // Desenha no ecrã real, onde o texto não sofre inversão da câmara
        draw_text(
            &gen_text, 
            screen_pos.x - text_dims.width / 2.0, 
            screen_pos.y + 25.0 * (camera.zoom.y.abs() * screen_height() / 2.0).clamp(0.5, 1.5), 
            font_size, 
            WHITE
        );
    }
}

// --- MAIN ---

#[macroquad::main("Ecossistema: Fix de Texto Espelhado")]
async fn main() {
    let mut world_w = screen_width();
    let mut world_h = screen_height();

    let mut agents: Vec<Agent> = (0..INITIAL_AGENTS)
        .map(|_| Agent::new(
            vec2(gen_range(100.0, world_w-100.0), gen_range(100.0, world_h-100.0)),
            Brain::new_random(),
            Color::new(gen_range(0.3, 0.6), gen_range(0.4, 0.8), 1.0, 1.0),
            1
        ))
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

        // --- ZOOM FOCADO NO RATO ---
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

        // --- UI E BOTÕES ---
        if is_mouse_button_pressed(MouseButton::Left) {
            let m_pos = mouse_position();
            if m_pos.0 > world_w - 50.0 && m_pos.0 < world_w - 10.0 && m_pos.1 > 10.0 && m_pos.1 < 50.0 {
                max_food += 1;
            }
            if m_pos.0 > world_w - 100.0 && m_pos.0 < world_w - 60.0 && m_pos.1 > 10.0 && m_pos.1 < 50.0 {
                if max_food > 0 { max_food -= 1; }
            }
        }

        // --- LÓGICA DO MUNDO ---
        if foods.len() < max_food && gen_range(0, 20) == 0 {
            foods.push(vec2(gen_range(30.0, world_w-30.0), gen_range(30.0, world_h-30.0)));
        }
        if foods.len() > max_food { foods.pop(); }

        let mut new_agents = Vec::new();
        for i in 0..agents.len() {
            foods.retain(|&f| {
                if agents[i].pos.distance(f) < 18.0 {
                    agents[i].energy += 75.0; 
                    false
                } else { true }
            });

            if agents[i].is_attacking {
                for j in 0..agents.len() {
                    if i == j { continue; }
                    let dist = agents[i].pos.distance(agents[j].pos);
                    if dist < 22.0 {
                        agents[j].energy -= 8.0;
                        agents[i].energy += 4.0; 
                    }
                }
            }
        }

        for i in 0..agents.len() {
            let closest_f = foods.iter()
                .min_by(|a, b| agents[i].pos.distance(**a).partial_cmp(&agents[i].pos.distance(**b)).unwrap())
                .cloned();

            let mut closest_e = None;
            let mut min_dist_e = f32::MAX;
            for j in 0..agents.len() {
                if i == j { continue; }
                let d = agents[i].pos.distance(agents[j].pos);
                if d < min_dist_e {
                    min_dist_e = d;
                    closest_e = Some(agents[j].pos);
                }
            }

            let agent = &mut agents[i];
            agent.update(closest_f, closest_e, world_w, world_h);

            if agent.energy > 280.0 {
                agent.energy -= 140.0;
                new_agents.push(Agent::new(agent.pos, agent.brain.mutate(), agent.color, agent.generation + 1));
            }
        }
        agents.retain(|a| a.energy > 0.0);
        agents.extend(new_agents);

        // --- RENDERIZAÇÃO ---
        clear_background(Color::new(0.015, 0.015, 0.02, 1.0));

        let camera = Camera2D {
            target: camera_pos,
            zoom: vec2(zoom / world_w * 2.0, -zoom / world_h * 2.0),
            ..Default::default()
        };
        
        // Desenha objetos do mundo (com câmara ativa)
        set_camera(&camera);
        for food in &foods {
            draw_circle(food.x, food.y, 4.5, GREEN);
            draw_circle_lines(food.x, food.y, 6.0, 1.0, Color::new(0.0, 1.0, 0.0, 0.3));
        }
        for agent in &agents {
            agent.draw_body();
        }

        // Desenha UI e Rótulos (com câmara padrão/ecrã)
        set_default_camera();
        for agent in &agents {
            agent.draw_label(&camera);
        }

        draw_rectangle(5.0, 5.0, 280.0, 40.0, Color::new(0.0, 0.0, 0.0, 0.8));
        draw_text(&format!("Vivos: {} | Recursos: {}/{}", agents.len(), foods.len(), max_food), 15.0, 30.0, 20.0, WHITE);
        
        draw_rectangle(world_w - 100.0, 10.0, 40.0, 40.0, Color::new(0.2, 0.0, 0.0, 1.0));
        draw_text("-", world_w - 85.0, 40.0, 30.0, WHITE);
        
        draw_rectangle(world_w - 50.0, 10.0, 40.0, 40.0, Color::new(0.0, 0.2, 0.0, 1.0));
        draw_text("+", world_w - 35.0, 40.0, 30.0, WHITE);

        if agents.is_empty() {
            draw_text("EXTINÇÃO ATINGIDA", world_w / 2.0 - 120.0, world_h / 2.0, 30.0, RED);
        }

        next_frame().await
    }
}