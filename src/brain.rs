use macroquad::rand::gen_range;

#[repr(usize)]
pub enum Sensor {
    FoodDist = 0,
    FoodAngle = 1,
    EnemyDist = 2,
    EnemyAngle = 3,
    WallAhead = 4,      
    WallLeft = 5,
    WallRight = 6,
    EnergyLevel = 7,
    DistFromCenter = 8, 
    Bias = 9,
}

pub const INPUT_SIZE: usize = 10;
pub const OUTPUT_SIZE: usize = 3; 

#[derive(Clone)]
pub struct Brain {
    pub weights: Vec<f32>,
}

impl Brain {
    pub fn new_random() -> Self {
        let mut weights = vec![0.0; INPUT_SIZE * OUTPUT_SIZE];
        
        weights[Sensor::FoodAngle as usize] = 3.5;       
        weights[Sensor::WallLeft as usize] = 5.0;        
        weights[Sensor::WallRight as usize] = -5.0;      
        weights[Sensor::DistFromCenter as usize] = 1.0;  

        weights[Sensor::Bias as usize + INPUT_SIZE] = 0.6;
        weights[Sensor::FoodDist as usize + INPUT_SIZE] = 1.2;
        weights[Sensor::WallAhead as usize + INPUT_SIZE] = -2.5; 

        weights[Sensor::EnemyDist as usize + INPUT_SIZE * 2] = 2.0;

        for w in weights.iter_mut() {
            *w += gen_range(-0.3, 0.3);
        }

        Self { weights }
    }

    pub fn mutate(&self) -> Self {
        let mut new_weights = self.weights.clone();
        for w in new_weights.iter_mut() {
            if gen_range(0.0, 1.0) < 0.15 {
                *w += gen_range(-0.1, 0.1);
            }
        }
        Self { weights: new_weights }
    }

    pub fn think(&self, sensors: &[f32; INPUT_SIZE]) -> (f32, f32, f32) {
        let mut outputs = [0.0; OUTPUT_SIZE];
        for i in 0..OUTPUT_SIZE {
            let mut sum = 0.0;
            for j in 0..INPUT_SIZE {
                sum += sensors[j] * self.weights[i * INPUT_SIZE + j];
            }
            outputs[i] = sum.tanh();
        }
        (outputs[0], outputs[1], outputs[2])
    }
}