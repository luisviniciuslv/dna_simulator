use macroquad::{prelude::*, rand::gen_range};

// --- CONFIGURAÇÕES ---
const GRID_SIZE: f32 = 50.0;
const INITIAL_AGENTS: usize = 30;
const FOOD_COUNT: usize = 50;
const ENERGY_LOSS_WALK: f32 = 0.02;
const ENERGY_LOSS_RUN: f32 = 0.1;
const ATTACK_COST: f32 = 0.5;

// --- ESTRUTURAS DE IA ---

#[derive(Clone)]
struct Brain {
    // Pesos da Rede Neural (Entradas -> Saídas)
    // 6 entradas: [Dist_Comida, Ang_Comida, Dist_Inimigo, Ang_Inimigo, Energia, Constante]
    // 3 saídas: [Giro, Velocidade, Intenção_Ataque]
    weights: Vec<f32>, 
}

impl Brain {
    fn new_random() -> Self {
        let weights = (0..18).map(|_| gen_range(-1.0, 1.0)).collect();
        Self { weights }
    }

    fn mutate(&self) -> Self {
        let mut new_weights = self.weights.clone();
        for w in new_weights.iter_mut() {
            if gen_range(0.0, 1.0) < 0.1 { // 10% de chance de mutação por peso
                *w += gen_range(-0.2, 0.2);
            }
        }
        Self { weights: new_weights }
    }

    // Processa entradas e retorna (giro, velocidade, ataque)
    fn think(&self, inputs: &[f32; 6]) -> (f32, f32, f32) {
        let mut outputs = [0.0; 3];
        for i in 0..3 {
            let mut sum = 0.0;
            for j in 0..6 {
                sum += inputs[j] * self.weights[i * 6 + j];
            }
            outputs[i] = sum.tanh(); // Ativação para manter entre -1 e 1
        }
        (outputs[0], outputs[1], outputs[2])
    }
}

// --- ENTIDADES ---

struct Agent {
    pos: Vec2,
    angle: f32,
    energy: f32,
    brain: Brain,
    color: Color,
    is_attacking: bool,
}

impl Agent {
    fn new(pos: Vec2, brain: Brain, color: Color) -> Self {
        Self {
            pos,
            angle: gen_range(0.0, std::f32::consts::TAU),
            energy: 100.0,
            brain,
            color,
            is_attacking: false,
        }
    }

    fn update(&mut self, closest_food: Option<Vec2>, closest_enemy: Option<Vec2>) {
        // Preparar entradas para o cérebro
        let food_rel = closest_food.map(|f| (f - self.pos)).unwrap_or(Vec2::ZERO);
        let enemy_rel = closest_enemy.map(|e| (e - self.pos)).unwrap_or(Vec2::ZERO);

        let inputs = [
            food_rel.length() / screen_width(),
            food_rel.y.atan2(food_rel.x) - self.angle,
            enemy_rel.length() / screen_width(),
            enemy_rel.y.atan2(enemy_rel.x) - self.angle,
            self.energy / 200.0,
            1.0 // Bias
        ];

        let (steer, speed_input, attack_input) = self.brain.think(&inputs);

        // Aplicar resultados
        self.angle += steer * 0.1;
        
        // Velocidade: -1 a 1 mapeado para 0.5 (devagar) a 4.0 (correndo)
        let speed = ((speed_input + 1.0) / 2.0) * 3.5 + 0.5;
        let velocity = vec2(self.angle.cos(), self.angle.sin()) * speed;
        self.pos += velocity;

        // Custos de energia
        self.energy -= if speed > 2.0 { ENERGY_LOSS_RUN } else { ENERGY_LOSS_WALK };
        
        self.is_attacking = attack_input > 0.5;
        if self.is_attacking {
            self.energy -= ATTACK_COST;
        }

        // Limites do mapa (Não atravessa)
        self.pos.x = self.pos.x.clamp(5.0, screen_width() - 5.0);
        self.pos.y = self.pos.y.clamp(5.0, screen_height() - 5.0);
    }

    fn draw(&self) {
        let draw_color = if self.is_attacking { RED } else { self.color };
        draw_poly(self.pos.x, self.pos.y, 3, 6.0, self.angle.to_degrees(), draw_color);
        
        // Barra de energia pequena acima
        draw_rectangle(self.pos.x - 5.0, self.pos.y - 10.0, (self.energy / 10.0).max(0.0), 2.0, GREEN);
    }
}

// --- MAIN LOOP ---

#[macroquad::main("Evolução Milenar: Milestone 4")]
async fn main() {
    let mut agents: Vec<Agent> = (0..INITIAL_AGENTS)
        .map(|_| Agent::new(
            vec2(gen_range(0.0, screen_width()), gen_range(0.0, screen_height())),
            Brain::new_random(),
            Color::new(gen_range(0.4, 1.0), gen_range(0.4, 1.0), 1.0, 1.0)
        ))
        .collect();

    let mut foods: Vec<Vec2> = (0..FOOD_COUNT)
        .map(|_| vec2(gen_range(0.0, screen_width()), gen_range(0.0, screen_height())))
        .collect();

    loop {
        clear_background(BLACK);

        // Repovoar comida
        if foods.len() < FOOD_COUNT && gen_range(0, 5) == 0 {
            foods.push(vec2(gen_range(10.0, screen_width()-10.0), gen_range(10.0, screen_height()-10.0)));
        }

        // --- Lógica de Interação ---
        // (Simulando o Grid: Para este exemplo simplificado, usamos busca direta, 
        // mas as entradas da IA já limitam o comportamento)
        
        let mut new_agents = Vec::new();

        // 1. Alimentação e Combate
        for i in 0..agents.len() {
            // Comer comida
            foods.retain(|&f| {
                if agents[i].pos.distance(f) < 10.0 {
                    agents[i].energy += 40.0;
                    false
                } else { true }
            });

            // Ataque entre agentes
            if agents[i].is_attacking {
                for j in 0..agents.len() {
                    if i == j { continue; }
                    let dist = agents[i].pos.distance(agents[j].pos);
                    if dist < 15.0 {
                        agents[j].energy -= 2.0; // Dano ao alvo
                        agents[i].energy += 1.0; // Atacante ganha parte (predação)
                    }
                }
            }
        }

        // 2. Pensar e Mover
        for agent in agents.iter_mut() {
            let closest_f = foods.iter().min_by(|a, b| 
                agent.pos.distance(**a).partial_cmp(&agent.pos.distance(**b)).unwrap_or(std::cmp::Ordering::Equal)
            ).cloned();

            // Simular "inimigo" próximo para o cérebro
            let closest_e = None; // Pode ser implementado similar à comida

            agent.update(closest_f, closest_e);
            agent.draw();

            // Reprodução
            if agent.energy > 200.0 {
                agent.energy -= 100.0;
                new_agents.push(Agent::new(agent.pos, agent.brain.mutate(), agent.color));
            }
        }

        // 3. Limpeza
        agents.retain(|a| a.energy > 0.0);
        agents.extend(new_agents);

        // Desenhar Comida
        for food in &foods {
            draw_circle(food.x, food.y, 2.5, GREEN);
        }

        draw_text(&format!("População: {} | Comida: {}", agents.len(), foods.len()), 10.0, 20.0, 20.0, WHITE);
        
        next_frame().await
    }
}