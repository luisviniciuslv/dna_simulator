mod config;
mod brain;
mod agent;

use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::agent::Agent;
use crate::brain::Brain;
use crate::config::*;

#[macroquad::main("DNA Simulator: Refatorado")]
async fn main() {
    let mut world_w = screen_width();
    let mut world_h = screen_height();
    let mut id_generator: u64 = 0;

    // Inicialização da População
    let mut agents: Vec<Agent> = (0..INITIAL_AGENTS)
        .map(|_| {
            id_generator += 1;
            Agent::new(
                id_generator, 
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

        // --- ENTRADA E CONTROLE DE CÂMERA ---
        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            let mut camera = Camera2D { target: camera_pos, zoom: vec2(zoom / world_w * 2.0, -zoom / world_h * 2.0), ..Default::default() };
            let mouse_world_before = camera.screen_to_world(mouse_position().into());
            if wheel_y > 0.0 { zoom *= 1.1; } else { zoom /= 1.1; }
            zoom = zoom.clamp(1.0, 10.0);
            if zoom > 1.001 {
                camera.zoom = vec2(zoom / world_w * 2.0, -zoom / world_h * 2.0);
                let mouse_world_after = camera.screen_to_world(mouse_position().into());
                camera_pos += mouse_world_before - mouse_world_after;
            } else {
                camera_pos = vec2(world_w / 2.0, world_h / 2.0);
            }
        }

        // Interface: Controle de Recursos
        if is_mouse_button_pressed(MouseButton::Left) {
            let m = mouse_position();
            if m.0 > world_w - 50.0 && m.0 < world_w - 10.0 && m.1 > 10.0 && m.1 < 50.0 { max_food += 1; }
            if m.0 > world_w - 100.0 && m.0 < world_w - 60.0 && m.1 > 10.0 && m.1 < 50.0 { if max_food > 0 { max_food -= 1; } }
        }

        // Spawn de Comida
        if foods.len() < max_food && gen_range(0, MAX_FOOD_SPAWN_CHANCE) == 0 { 
            foods.push(vec2(gen_range(30.0, world_w-30.0), gen_range(30.0, world_h-30.0))); 
        }

        let mut next_generation = Vec::new();

        // --- INTERAÇÕES E COMBATE ---
        for i in 0..agents.len() {
            // Coleta de Comida
            foods.retain(|&f| {
                if agents[i].pos.distance(f) < 20.0 {
                    agents[i].energy += FOOD_ENERGY_GAIN; 
                    false
                } else { true }
            });

            // Ataques
            if agents[i].is_attacking {
                for j in 0..agents.len() {
                    if i == j { continue; }
                    if agents[j].spawn_timer > 0.0 || agents[j].hit_timer > 0.0 { continue; }
                    
                    let are_family = agents[i].parent_id == Some(agents[j].id) || agents[j].parent_id == Some(agents[i].id);
                    if are_family && (agents[i].family_timer > 0.0 || agents[j].family_timer > 0.0) { continue; }

                    if agents[i].pos.distance(agents[j].pos) < ATTACK_RADIUS {
                        agents[j].energy -= ATTACK_DAMAGE; 
                        agents[i].energy += ATTACK_LIFESTEAL; 
                        agents[j].hit_timer = HIT_STUN_TIME; 
                        let push_dir = (agents[j].pos - agents[i].pos).normalize_or_zero();
                        agents[j].impulse_vec += push_dir * KNOCKBACK_IMPULSE;
                        agents[i].impulse_vec -= push_dir * (KNOCKBACK_IMPULSE * KNOCKBACK_RECOIL_FACTOR); 
                    }
                }
            }
        }

        // --- ATUALIZAÇÃO E REPRODUÇÃO ---
        for i in 0..agents.len() {
            let pos_i = agents[i].pos;
            let id_i = agents[i].id;
            let parent_i = agents[i].parent_id;
            let fam_i = agents[i].family_timer;

            // Busca de alvos para a IA
            let closest_f = foods.iter().filter(|&&f| pos_i.distance(f) < VISION_RADIUS).min_by(|a, b| pos_i.distance(**a).partial_cmp(&pos_i.distance(**b)).unwrap()).cloned();
            let mut closest_e = None;
            let mut min_dist_e = VISION_RADIUS;
            
            for j in 0..agents.len() {
                if i == j { continue; }
                let are_family = parent_i == Some(agents[j].id) || agents[j].parent_id == Some(id_i);
                if are_family && (fam_i > 0.0 || agents[j].family_timer > 0.0) { continue; }
                let d = pos_i.distance(agents[j].pos);
                if d < min_dist_e { min_dist_e = d; closest_e = Some(agents[j].pos); }
            }

            let agent = &mut agents[i];
            agent.update(closest_f, closest_e, world_w, world_h, dt);

            if agent.energy > REPRODUCTION_THRESHOLD {
                agent.energy -= REPRODUCTION_COST; 
                id_generator += 1;
                next_generation.push(Agent::new(id_generator, Some(agent.id), agent.pos, agent.brain.mutate(), agent.color, agent.generation + 1));
            }
        }

        agents.retain(|a| a.energy > 0.0);
        agents.extend(next_generation);

        // --- RENDERIZAÇÃO ---
        clear_background(COLOR_BACKGROUND);
        let camera = Camera2D { target: camera_pos, zoom: vec2(zoom / world_w * 2.0, -zoom / world_h * 2.0), ..Default::default() };
        set_camera(&camera);
        
        draw_rectangle_lines(0.0, 0.0, world_w, world_h, 3.0, COLOR_WALL);

        for food in &foods { 
            draw_circle(food.x, food.y, 4.0, COLOR_FOOD); 
            draw_circle_lines(food.x, food.y, 6.0, 1.0, Color::new(0.0, 1.0, 0.0, 0.2)); 
        }
        for agent in &agents { agent.draw_body(); }

        // Elementos de UI (Câmera Padrão)
        set_default_camera();
        for agent in &agents { agent.draw_label(&camera); }

        draw_rectangle(10.0, 10.0, 320.0, 50.0, Color::new(0.0, 0.0, 0.0, 0.8));
        draw_text(&format!("POPULAÇÃO: {} | RECURSOS: {}/{}", agents.len(), foods.len(), max_food), 20.0, 40.0, 20.0, WHITE);
        
        // Botões
        draw_rectangle(world_w - 110.0, 10.0, 45.0, 45.0, Color::new(0.3, 0.1, 0.1, 1.0)); draw_text("-", world_w - 92.0, 42.0, 30.0, WHITE);
        draw_rectangle(world_w - 55.0, 10.0, 45.0, 45.0, Color::new(0.1, 0.3, 0.1, 1.0)); draw_text("+", world_w - 40.0, 42.0, 30.0, WHITE);

        if agents.is_empty() { draw_text("EXTINÇÃO", world_w / 2.0 - 80.0, world_h / 2.0, 40.0, RED); }
        next_frame().await
    }
}