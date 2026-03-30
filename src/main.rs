use macroquad::{prelude::*, rand::gen_range};

// Configurações básicas da simulação
const AGENT_SIZE: f32 = 5.0;
const INITIAL_ENERGY: f32 = 100.0;
const ENERGY_LOSS_RATE: f32 = 0.05;

#[derive(Clone)]
struct DNA {
    max_speed: f32,
    color: Color,
}

struct Agent {
    pos: Vec2,
    vel: Vec2,
    energy: f32,
    dna: DNA,
}

impl Agent {
    fn new(x: f32, y: f32) -> Self {
        let speed = gen_range(1.0, 1.5);
        let angle = gen_range(0.0, std::f32::consts::TAU);
        
        Self {
            pos: vec2(x, y),
            vel: vec2(angle.cos() * speed, angle.sin() * speed),
            energy: INITIAL_ENERGY,
            dna: DNA {
                max_speed: speed,
                color: Color::new(gen_range(0.5, 1.0), gen_range(0.5, 1.0), gen_range(0.5, 1.0), 1.0),
            },
        }
    }

    fn update(&mut self) {
        // Movimento básico
        self.pos += self.vel;
        
        // Perda de energia constante por "viver"
        self.energy -= ENERGY_LOSS_RATE;

        // Lógica de borda (Teleporte para o outro lado)
        if self.pos.x < 0.0 { self.pos.x = screen_width(); }
        if self.pos.x > screen_width() { self.pos.x = 0.0; }
        if self.pos.y < 0.0 { self.pos.y = screen_height(); }
        if self.pos.y > screen_height() { self.pos.y = 0.0; }
    }

    fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, AGENT_SIZE, self.dna.color);
    }
}

#[macroquad::main("Simulador de Ecossistema - Milestone 1")]
async fn main() {
    let mut agents: Vec<Agent> = Vec::new();

    // Spawn inicial de 100 agentes
    for _ in 0..100 {
        agents.push(Agent::new(
            gen_range(0.0, screen_width()),
            gen_range(0.0, screen_height()),
        ));
    }

    loop {
        clear_background(BLACK);

        // Atualizar e remover agentes sem energia
        // Usamos retain para filtrar os vivos de forma eficiente
        agents.retain(|a| a.energy > 0.0);

        for agent in agents.iter_mut() {
            agent.update();
            agent.draw();
        }

        // Informações na tela
        draw_text(&format!("Agentes Vivos: {}", agents.len()), 10.0, 20.0, 20.0, WHITE);
        
        if agents.is_empty() {
            draw_text("EXTINÇÃO ATINGIDA", screen_width()/2.0 - 100.0, screen_height()/2.0, 30.0, RED);
        }

        next_frame().await
    }
}