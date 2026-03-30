mod config;
mod brain;
mod agent;

use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::agent::Agent;
use crate::brain::Brain;
use crate::config::*;

#[macroquad::main("Simulador DNA: Inteligência e Interface")]
async fn main() {
    let mut world_w = screen_width();
    let mut world_h = screen_height();
    let mut id_gen: u64 = 0;

    // Inicialização da População
    let mut agents: Vec<Agent> = (0..INITIAL_AGENTS).map(|_| {
        id_gen += 1;
        Agent::new(
            id_gen, 
            None, 
            vec2(gen_range(100.0, world_w-100.0), gen_range(100.0, world_h-100.0)), 
            Brain::new_random(), 
            Color::new(gen_range(0.4, 1.0), gen_range(0.4, 1.0), 1.0, 1.0), 
            1
        )
    }).collect();

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

        // --- LÓGICA DE ZOOM E CÂMERA (EXATAMENTE COMO ORIGINAL) ---
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

        // --- BOTÕES DE RECURSOS (EXATAMENTE COMO ORIGINAL) ---
        if is_mouse_button_pressed(MouseButton::Left) {
            let m_pos = mouse_position();
            // Botão "+" no HUD (espaço da tela)
            if m_pos.0 > world_w - 55.0 && m_pos.0 < world_w - 10.0 && m_pos.1 > 10.0 && m_pos.1 < 55.0 { 
                max_food += 1; 
            }
            // Botão "-" no HUD (espaço da tela)
            if m_pos.0 > world_w - 110.0 && m_pos.0 < world_w - 65.0 && m_pos.1 > 10.0 && m_pos.1 < 55.0 { 
                if max_food > 0 { max_food -= 1; } 
            }
        }

        // Spawn de comida
        if foods.len() < max_food && gen_range(0, MAX_FOOD_SPAWN_CHANCE) == 0 {
            foods.push(vec2(gen_range(30.0, world_w-30.0), gen_range(30.0, world_h-30.0)));
        }

        let mut offspring = Vec::new();

        // --- LÓGICA DE INTERAÇÃO COLETIVA ---
        for i in 0..agents.len() {
            // Comer
            foods.retain(|&f| {
                if agents[i].pos.distance(f) < 20.0 {
                    agents[i].energy += FOOD_ENERGY_GAIN;
                    false
                } else { true }
            });

            // Combate Ativo
            if agents[i].is_attacking {
                for j in 0..agents.len() {
                    if i == j { continue; }
                    
                    let a_pos = agents[i].pos;
                    let a_id = agents[i].id;
                    let a_parent = agents[i].parent_id;
                    let a_fam_t = agents[i].family_timer;
                    
                    let b = &mut agents[j];
                    if b.spawn_timer > 0.0 { continue; } 
                    
                    let are_fam = a_parent == Some(b.id) || b.parent_id == Some(a_id);
                    if are_fam && (a_fam_t > 0.0 || b.family_timer > 0.0) { continue; }

                    if a_pos.distance(b.pos) < ATTACK_RADIUS {
                        let push = (b.pos - a_pos).normalize_or_zero();
                        b.energy -= ATTACK_DAMAGE;
                        b.hit_timer = HIT_STUN_TIME;
                        b.impulse_vec += push * 16.0; 
                        agents[i].energy += ATTACK_LIFESTEAL;
                    }
                }
            }
        }

        // --- ATUALIZAÇÃO E SENSORES ---
        for i in 0..agents.len() {
            let pos = agents[i].pos;
            let id = agents[i].id;
            let parent = agents[i].parent_id;
            let fam_t = agents[i].family_timer;
            
            let closest_f = foods.iter()
                .filter(|&&f| pos.distance(f) < VISION_RADIUS)
                .min_by(|a, b| pos.distance(**a).partial_cmp(&pos.distance(**b)).unwrap())
                .cloned();

            let closest_e = agents.iter().enumerate()
                .filter(|(idx, other)| {
                    if *idx == i { return false; }
                    if pos.distance(other.pos) >= VISION_RADIUS { return false; }
                    let are_fam = parent == Some(other.id) || other.parent_id == Some(id);
                    if are_fam && (fam_t > 0.0 || other.family_timer > 0.0) { return false; }
                    true
                })
                .min_by(|(_, a), (_, b)| pos.distance(a.pos).partial_cmp(&pos.distance(b.pos)).unwrap())
                .map(|(_, a)| a.pos);

            let agent = &mut agents[i];
            agent.update(closest_f, closest_e, world_w, world_h, dt);

            // Reprodução
            if agent.energy > REPRODUCTION_THRESHOLD && agent.repro_timer <= 0.0 {
                agent.energy -= REPRODUCTION_COST;
                agent.repro_timer = REPRO_COOLDOWN; 
                id_gen += 1;
                offspring.push(Agent::new(id_gen, Some(agent.id), agent.pos, agent.brain.mutate(), agent.color, agent.generation + 1));
            }
        }

        agents.retain(|a| a.energy > 0.0);
        agents.extend(offspring);

        // --- RENDER ---
        clear_background(Color::new(0.01, 0.01, 0.02, 1.0));
        
        let camera = Camera2D { 
            target: camera_pos, 
            zoom: vec2(zoom / world_w * 2.0, -zoom / world_h * 2.0), 
            ..Default::default() 
        };
        set_camera(&camera);
        
        // Bordas do Mundo
        draw_rectangle_lines(0.0, 0.0, world_w, world_h, 3.0, Color::new(0.3, 0.3, 0.7, 0.3));

        for f in &foods { 
            draw_circle(f.x, f.y, 4.0, GREEN); 
            draw_circle_lines(f.x, f.y, 7.0, 1.0, Color::new(0.0, 1.0, 0.0, 0.2));
        }
        for a in &agents { a.draw(); }

        // --- UI e HUD (Câmera Default / Espaço da Tela) ---
        set_default_camera();
        for a in &agents { a.draw_label(&camera); }

        // Painel de Informações
        draw_rectangle(10.0, 10.0, 300.0, 50.0, Color::new(0.0, 0.0, 0.0, 0.8));
        draw_text(&format!("POPULAÇÃO: {} | RECURSOS: {}/{}", agents.len(), foods.len(), max_food), 20.0, 40.0, 20.0, WHITE);
        
        // Desenho dos Botões (HUD)
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