use macroquad::rand::gen_range;

pub const INPUT_SIZE: usize = 10;
pub const OUTPUT_SIZE: usize = 3;
pub const WEIGHTS_COUNT: usize = INPUT_SIZE * OUTPUT_SIZE;

#[derive(Clone)]
pub struct Brain {
    pub weights: Vec<f32>,
}

impl Brain {
    pub fn new_random() -> Self {
        let mut weights = vec![0.0; WEIGHTS_COUNT];
        
        // Genes instintivos (pré-configurados para evitar extinção inicial)
        weights[1] = 2.5;   // Virar para comida
        weights[3] = 1.5;   // Virar para inimigo
        weights[5] = 1.2;   // Fugir da parede esquerda
        weights[6] = -1.2;  // Fugir da parede direita
        weights[10] = 1.8;  
        weights[22] = 3.0;  // Atacar quando inimigo estiver perto
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

    pub fn mutate(&self) -> Self {
        let mut new_weights = self.weights.clone();
        let mutation_chance = 0.15;
        let mutation_strength = 0.1;

        for w in new_weights.iter_mut() {
            if gen_range(0.0, 1.0) < mutation_chance {
                *w += gen_range(-mutation_strength, mutation_strength);
            }
        }
        Self { weights: new_weights }
    }

    pub fn think(&self, inputs: &[f32; INPUT_SIZE]) -> (f32, f32, f32) {
        let mut outputs = [0.0; OUTPUT_SIZE];
        for i in 0..OUTPUT_SIZE {
            let mut sum = 0.0;
            for j in 0..INPUT_SIZE {
                sum += inputs[j] * self.weights[i * INPUT_SIZE + j];
            }
            outputs[i] = sum.tanh();
        }
        (outputs[0], outputs[1], outputs[2])
    }
}