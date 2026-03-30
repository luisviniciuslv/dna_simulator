use macroquad::{prelude::*, rand::gen_range};

// --- CONFIGURAÇÕES AJUSTADAS PARA MOVIMENTO ORGÂNICO ---
const INITIAL_AGENTS: usize = 8; // Aumentado levemente para melhor observação
const FOOD_COUNT: usize = 6;

// Custos balanceados para movimento mais lento
const ENERGY_LOSS_IDLE: f32 = 0.005;  
const ENERGY_LOSS_MOVE: f32 = 0.015;   
const ENERGY_LOSS_RUN: f32 = 0.1;    
const ATTACK_COST: f32 = 0.3;        

// --- ESTRUTURAS DE IA ---

#[derive(Clone)]
struct Brain {
    // Pesos da Rede Neural (6 entradas -> 3 saídas)
    weights: Vec<f32>, 
}

impl Brain {
    fn new_random() -> Self {
        let mut weights = vec![0.0; 18];

        // Instinto 1: Virar para a comida (mais suave que antes)
        weights[1] = 0.8; 

        // Instinto 2: Impulso de exploração (Bias -> Velocidade moderada)
        weights[11] = 0.4; 

        // Instinto 3: Tendência a não atacar por padrão
        weights[17] = -1.2; 

        // Ruído genético inicial
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

    fn update(&mut self, closest_food: Option<Vec2>, closest_enemy: Option<Vec2>) {
        // --- INPUTS ---
        let food_input_dist;
        let food_input_angle;
        if let Some(f) = closest_food {
            let rel = f - self.pos;
            food_input_dist = (1.0 - (rel.length() / screen_width())).clamp(0.0, 1.0);
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
            enemy_input_dist = (1.0 - (rel.length() / screen_width())).clamp(0.0, 1.0);
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

        // --- COMPORTAMENTO ORGÂNICO ---
        
        // 1. Ruído de Exploração: Pequeno desvio aleatório constante para evitar linhas retas perfeitas
        self.angle += gen_range(-0.04, 0.04);

        // 2. Giro Suavizado
        self.angle += steer * 0.08; 
        
        // 3. Velocidade Reduzida: Máximo de 1.4 em vez de 2.2
        let target_speed = if speed_input > -0.4 {
            ((speed_input + 0.4) / 1.4) * 1.4
        } else {
            0.0
        };

        // Interpolação de velocidade (Inércia leve)
        self.current_speed = self.current_speed * 0.9 + target_speed * 0.1;

        let velocity = vec2(self.angle.cos(), self.angle.sin()) * self.current_speed;
        self.pos += velocity;

        // --- CUSTOS ---
        self.energy -= ENERGY_LOSS_IDLE;
        if self.current_speed > 0.1 {
            self.energy -= if self.current_speed > 1.0 { ENERGY_LOSS_RUN } else { ENERGY_LOSS_MOVE };
        }
        
        self.is_attacking = attack_input > 0.6;
        if self.is_attacking {
            self.energy -= ATTACK_COST;
        }

        // Colisões com a borda
        self.pos.x = self.pos.x.clamp(10.0, screen_width() - 10.0);
        self.pos.y = self.pos.y.clamp(10.0, screen_height() - 10.0);
    }

    fn draw(&self) {
        let draw_color = if self.is_attacking { RED } else { self.color };
        draw_poly(self.pos.x, self.pos.y, 3, 7.0, self.angle.to_degrees(), draw_color);
        
        // Indicador de Energia
        draw_rectangle(self.pos.x - 6.0, self.pos.y - 12.0, 12.0, 2.0, Color::new(0.2, 0.2, 0.2, 0.5));
        let energy_ratio = (self.energy / 100.0).clamp(0.0, 1.0);
        draw_rectangle(self.pos.x - 6.0, self.pos.y - 12.0, 12.0 * energy_ratio, 2.0, GREEN);

        // Geração Texto
        let gen_text = format!("G{}", self.generation);
        let font_size = 14.0;
        let text_size = measure_text(&gen_text, None, font_size as u16, 1.0);
        draw_text(
            &gen_text, 
            self.pos.x - text_size.width / 2.0, 
            self.pos.y + 18.0, 
            font_size, 
            WHITE
        );
    }
}

// --- MAIN ---

#[macroquad::main("Ecossistema Neural: Comportamento Orgânico")]
async fn main() {
    let mut agents: Vec<Agent> = (0..INITIAL_AGENTS)
        .map(|_| Agent::new(
            vec2(gen_range(100.0, screen_width()-100.0), gen_range(100.0, screen_height()-100.0)),
            Brain::new_random(),
            Color::new(gen_range(0.3, 0.6), gen_range(0.4, 0.8), 1.0, 1.0),
            1
        ))
        .collect();

    let mut foods: Vec<Vec2> = (0..FOOD_COUNT)
        .map(|_| vec2(gen_range(50.0, screen_width()-50.0), gen_range(50.0, screen_height()-50.0)))
        .collect();

    loop {
        clear_background(Color::new(0.015, 0.015, 0.02, 1.0));

        if foods.len() < FOOD_COUNT && gen_range(0, 20) == 0 {
            foods.push(vec2(gen_range(30.0, screen_width()-30.0), gen_range(30.0, screen_height()-30.0)));
        }

        let mut new_agents = Vec::new();

        // 1. Colisões e Alimentação
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

        // 2. IA e Movimento
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
            agent.update(closest_f, closest_e);
            agent.draw();

            if agent.energy > 280.0 {
                agent.energy -= 140.0;
                new_agents.push(Agent::new(agent.pos, agent.brain.mutate(), agent.color, agent.generation + 1));
            }
        }

        agents.retain(|a| a.energy > 0.0);
        agents.extend(new_agents);

        // Comida
        for food in &foods {
            draw_circle(food.x, food.y, 4.5, GREEN);
            draw_circle_lines(food.x, food.y, 6.0, 1.0, Color::new(0.0, 1.0, 0.0, 0.3));
        }

        // UI
        draw_rectangle(5.0, 5.0, 260.0, 35.0, Color::new(0.0, 0.0, 0.0, 0.8));
        draw_text(&format!("Vivos: {} | Recursos: {}", agents.len(), foods.len()), 15.0, 28.0, 20.0, WHITE);
        
        if agents.is_empty() {
            draw_text("EXTINÇÃO ATINGIDA", screen_width()/2.0 - 120.0, screen_height()/2.0, 30.0, RED);
        }

        next_frame().await
    }
}