use macroquad::{prelude::*, rand::gen_range};

// Configurações globais
const FOOD_COUNT: usize = 60;
const INITIAL_AGENTS: usize = 20;
const MUTATION_RATE: f32 = 0.1; // 10% de chance de mudança drástica ou variação pequena
const ENERGY_TO_REPRODUCE: f32 = 200.0;

#[derive(Clone, Copy)]
struct DNA {
    speed: f32,
    vision: f32,
    color: Color,
}

struct Agent {
    pos: Vec2,
    vel: Vec2,
    energy: f32,
    dna: DNA,
}

struct Food {
    pos: Vec2,
}

impl Agent {
    fn new(pos: Vec2, dna: DNA) -> Self {
        Self {
            pos,
            vel: vec2(gen_range(-1.0, 1.0), gen_range(-1.0, 1.0)).normalize(),
            energy: 100.0,
            dna,
        }
    }

    // Cria um filho com DNA levemente mutado
    fn reproduce(&mut self) -> Agent {
        self.energy /= 2.0; // Divide a energia com o filho
        let mut child_dna = self.dna;

        // Mutação na velocidade
        child_dna.speed += gen_range(-0.2, 0.2);
        child_dna.speed = child_dna.speed.max(0.5);

        // Mutação na visão
        child_dna.vision += gen_range(-10.0, 10.0);
        child_dna.vision = child_dna.vision.max(10.0);

        // Mutação leve na cor para rastrear linhagens
        child_dna.color.r = (child_dna.color.r + gen_range(-0.05, 0.05)).clamp(0.0, 1.0);
        child_dna.color.g = (child_dna.color.g + gen_range(-0.05, 0.05)).clamp(0.0, 1.0);
        child_dna.color.b = (child_dna.color.b + gen_range(-0.05, 0.05)).clamp(0.0, 1.0);

        Agent::new(self.pos, child_dna)
    }

    fn update(&mut self, foods: &[Food]) {
        // Encontrar comida mais próxima dentro da visão
        let mut closest_dist = self.dna.vision;
        let mut target = None;

        for food in foods {
            let dist = self.pos.distance(food.pos);
            if dist < closest_dist {
                closest_dist = dist;
                target = Some(food.pos);
            }
        }

        // Se viu comida, move-se para lá. Se não, move-se conforme a velocidade atual.
        if let Some(target_pos) = target {
            let desired = (target_pos - self.pos).normalize() * self.dna.speed;
            // Steering: suave ajuste de direção
            let steer = (desired - self.vel) * 0.1;
            self.vel += steer;
        }

        // Limita a velocidade ao DNA
        self.vel = self.vel.normalize() * self.dna.speed;
        self.pos += self.vel;

        // Custo metabólico: velocidade e visão custam energia
        let metabolism = (self.dna.speed.powi(2) * 0.01) + (self.dna.vision * 0.0005) + 0.01;
        self.energy -= metabolism;

        // Borda do mundo
        if self.pos.x < 0.0 { self.pos.x = screen_width(); }
        if self.pos.x > screen_width() { self.pos.x = 0.0; }
        if self.pos.y < 0.0 { self.pos.y = screen_height(); }
        if self.pos.y > screen_height() { self.pos.y = 0.0; }
    }

    fn draw(&self) {
        // Desenha raio de visão (opcional para debug)
        // draw_circle_lines(self.pos.x, self.pos.y, self.dna.vision, 1.0, Color::new(1.0, 1.0, 1.0, 0.1));
        
        draw_circle(self.pos.x, self.pos.y, 4.0, self.dna.color);
    }
}

#[macroquad::main("Ecossistema Evolutivo")]
async fn main() {
    let mut agents: Vec<Agent> = (0..INITIAL_AGENTS)
        .map(|_| {
            Agent::new(
                vec2(gen_range(0.0, screen_width()), gen_range(0.0, screen_height())),
                DNA {
                    speed: gen_range(1.5, 3.0),
                    vision: gen_range(50.0, 150.0),
                    color: Color::new(gen_range(0.3, 1.0), gen_range(0.3, 1.0), gen_range(0.3, 1.0), 1.0),
                },
            )
        })
        .collect();

    let mut foods: Vec<Food> = (0..FOOD_COUNT)
        .map(|_| Food { pos: vec2(gen_range(0.0, screen_width()), gen_range(0.0, screen_height())) })
        .collect();

    loop {
        clear_background(BLACK);

        // Repovoar comida
        if foods.len() < FOOD_COUNT {
            if rand::gen_range(0, 10) == 0 {
                foods.push(Food { pos: vec2(gen_range(0.0, screen_width()), gen_range(0.0, screen_height())) });
            }
        }

        // Lógica de colisão com comida (Alimentação)
        for agent in agents.iter_mut() {
            let mut i = 0;
            while i < foods.len() {
                if agent.pos.distance(foods[i].pos) < 8.0 {
                    agent.energy += 30.0;
                    foods.remove(i);
                } else {
                    i += 1;
                }
            }
        }

        // Atualizar Agentes
        for agent in agents.iter_mut() {
            agent.update(&foods);
            agent.draw();
        }

        // Reprodução
        let mut offspring = Vec::new();
        for agent in agents.iter_mut() {
            if agent.energy > ENERGY_TO_REPRODUCE {
                offspring.push(agent.reproduce());
            }
        }
        agents.extend(offspring);

        // Morte
        agents.retain(|a| a.energy > 0.0);

        // Renderizar Comida
        for food in &foods {
            draw_circle(food.pos.x, food.pos.y, 2.0, GREEN);
        }

        // UI
        draw_text(&format!("População: {}", agents.len()), 10.0, 20.0, 20.0, WHITE);
        
        next_frame().await
    }
}