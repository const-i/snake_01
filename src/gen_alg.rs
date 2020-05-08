extern crate rand;

use rand::Rng;
use rand_distr::{Distribution, Normal};

use crate::constants::*;
use crate::game::Brain;

fn sigmoid(z: f64) -> f64 {
    let e = std::f64::consts::E;
    1.0 / (1.0 + e.powf(-z))
}

fn get_normal() -> f64 {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, 1.0).unwrap();
    normal.sample(&mut rng)
}

pub struct Layer {
    pub num_inputs: u32,
    pub num_neurons: u32,
    pub weights: Vec<Vec<f64>>,
    pub biases: Vec<f64>,
}

impl Layer {
    pub fn new(num_inputs: u32, num_neurons: u32) -> Option<Layer> {
        if num_inputs == 0 || num_neurons == 0 {
            None
        } else {
            let biases: Vec<f64> = (0..num_neurons).map(|_| get_normal()).collect();
            let weights: Vec<Vec<f64>> = (0..num_neurons)
                .map(|_| (0..num_inputs).map(|_| get_normal()).collect())
                .collect();
            Some(Layer {
                num_inputs,
                num_neurons,
                weights,
                biases,
            })
        }
    }

    fn feed_forward(&self, inputs: &Vec<f64>) -> Option<Vec<f64>> {
        if self.num_inputs != inputs.len() as u32 {
            None
        } else {
            let wx: Vec<f64> = self
                .weights
                .iter()
                .map(|ws| ws.iter().zip(inputs.iter()).map(|(w, x)| w * x).sum())
                .collect();
            let wx_b: Vec<f64> = wx.iter().zip(self.biases.iter()).map(|(wx, b)| wx + b).collect();
            Some(wx_b.iter().map(|z| sigmoid(*z)).collect())
        }
    }

    fn should_mutate() -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() > 1f64 - MUTATION_PROBABILITY
    }

    fn mutate(&mut self) {
        self.biases = self
            .biases
            .iter()
            .map(|b| if Layer::should_mutate() { get_normal() } else { *b })
            .collect();
        self.weights = self
            .weights
            .iter()
            .map(|ws| {
                ws.iter()
                    .map(|w| if Layer::should_mutate() { get_normal() } else { *w })
                    .collect()
            })
            .collect();
    }

    fn should_crossover() -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() > 1f64 - CROSSOVER_PROBABILITY
    }

    fn crossover(&self, other: &Layer) -> (Layer, Layer) {
        let mut child1 = Layer::new(self.num_inputs, self.num_neurons).unwrap();
        let mut child2 = Layer::new(self.num_inputs, self.num_neurons).unwrap();

        // Cannot iter() over here since destructuring assignments are not allowed
        // https://github.com/rust-lang/rfcs/issues/372

        for i in 0..self.num_neurons as usize {
            if Layer::should_crossover() {
                child1.biases[i] = other.biases[i];
                child2.biases[i] = self.biases[i];
            } else {
                child1.biases[i] = self.biases[i];
                child2.biases[i] = other.biases[i];
            }
            for j in 0..self.num_inputs as usize {
                if Layer::should_crossover() {
                    child1.weights[i][j] = other.weights[i][j];
                    child2.weights[i][j] = self.weights[i][j];
                } else {
                    child1.weights[i][j] = self.weights[i][j];
                    child2.weights[i][j] = other.weights[i][j];
                }
            }
        }

        (child1, child2)
    }
}

pub struct NN {
    pub layers: Vec<Layer>,
}

impl NN {
    pub fn new() -> NN {
        NN { layers: Vec::new() }
    }

    pub fn new_defined(layer_def: &[[usize; 2]]) -> NN {
        let mut nn = NN::new();
        for layer in layer_def {
            nn.add(Layer::new(layer[0] as u32, layer[1] as u32).unwrap());
        }

        nn
    }

    pub fn add(&mut self, layer: Layer) -> bool {
        if self.layers.is_empty() || self.layers.last().unwrap().num_neurons == layer.num_inputs {
            self.layers.push(layer);
            true
        } else {
            false
        }
    }

    pub fn propagate(&self, inputs: &Vec<f64>) -> Option<Vec<f64>> {
        if self.layers.is_empty() || self.layers[0].num_inputs != inputs.len() as u32 {
            None
        } else {
            let mut this_in = inputs;
            let mut this_out: Vec<f64> = Vec::new();
            for layer in &self.layers {
                let temp = layer.feed_forward(&this_in);
                match temp {
                    Some(vals) => this_out = vals,
                    None => return None,
                }
                //this_out = layer.feed_forward(&this_in).unwrap();
                this_in = &this_out;
            }
            Some(this_out)
        }
    }

    fn mutate(&mut self) {
        for layer in &mut self.layers {
            layer.mutate();
        }
    }

    fn crossover(&self, other: &NN) -> (NN, NN) {
        let mut child1 = NN::new();
        let mut child2 = NN::new();
        for (p1, p2) in self.layers.iter().zip(other.layers.iter()) {
            let (c1, c2) = p1.crossover(p2);
            child1.add(c1);
            child2.add(c2);
        }
        (child1, child2)
    }
}

impl Brain for NN {
    fn get_action(&mut self, inputs: &Vec<f64>) -> Option<usize> {
        let output = self.propagate(inputs);
        match output {
            Some(vals) => get_index_max_float(&vals),
            None => None,
        }
    }

    fn train(
        &mut self,
        _state_initial: &Vec<f64>,
        _action: usize,
        _reward: f64,
        _state_final: &Vec<f64>,
    ) -> Option<bool> {
        Some(true)
    }
}

pub struct Population {
    pub length: usize,
    pub nn: Vec<NN>,
    pub fitness: Vec<f64>,
}

impl Population {
    pub fn new() -> Population {
        Population {
            length: 0,
            nn: Vec::new(),
            fitness: Vec::new(),
        }
    }

    pub fn new_defined(num_nn: u32, layer_def: &[[usize; 2]]) -> Population {
        let mut pop = Population::new();
        for _ in 0..num_nn {
            pop.add(NN::new_defined(layer_def));
        }
        pop
    }

    pub fn add(&mut self, nn: NN) {
        self.nn.push(nn);
        self.fitness.push(0f64);
        self.length += 1;
    }

    pub fn create_next_generation(&self) -> Population {
        let mut pop = Population::new();
        let sorted_index = self.get_sorted_index();
        let mid = self.length % 2 + self.length / 2;
        for i in 0..mid {
            let p1 = &self.nn[sorted_index[i]];
            let p2 = &self.nn[sorted_index[i + 1]];
            let (mut c1, mut c2) = p1.crossover(p2);
            c1.mutate();
            c2.mutate();
            pop.add(c1);
            if i == mid - 1 && self.length % 2 == 1 {
                // Do nothing
            } else {
                pop.add(c2);
            }
        }
        pop
    }

    pub fn get_sorted_index(&self) -> Vec<usize> {
        let mut index: Vec<(usize, f64)> = self.fitness.clone().into_iter().enumerate().collect();
        index.sort_by(|(_, af), (_, bf)| bf.partial_cmp(af).unwrap_or(std::cmp::Ordering::Equal));
        index.iter().map(|(i, _)| *i).collect()
    }
}

fn get_index_max_float(input: &Vec<f64>) -> Option<usize> {
    input
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigmoid() {
        assert!(sigmoid(-1000f64) >= 0f64);
        assert!(sigmoid(-1000f64) <= 0.5f64);
        assert!(sigmoid(1000f64) >= 0.5f64);
        assert!(sigmoid(1000f64) <= 1f64);
        assert_eq!(sigmoid(0f64), 0.5f64);
    }

    #[test]
    fn test_layer_new() {
        let layer = Layer::new(3, 2).unwrap();
        assert_eq!(layer.num_inputs, 3);
        assert_eq!(layer.num_neurons, 2);
        assert_eq!(layer.weights.len(), 2);
        assert_eq!(layer.weights.first().unwrap().len(), 3);
        assert_eq!(layer.biases.len(), 2);
        let res = Layer::new(0, 0);
        match res {
            Some(_) => assert!(false),
            None => assert!(true),
        }
    }

    #[test]
    fn test_layer_feed_forward() {
        let layer = Layer::new(3, 2).unwrap();
        let mut inputs = vec![1f64, 2f64];
        let mut outputs = layer.feed_forward(&inputs);
        assert_eq!(outputs, None);
        inputs = vec![0f64, 1f64, 0f64];
        outputs = layer.feed_forward(&inputs);
        println!("{:?}", outputs);
        assert_eq!(outputs.unwrap().len(), 2);
        //assert!(false);
    }

    #[test]
    fn test_nn_new() {
        let nn = NN::new();
        assert_eq!(nn.layers.len(), 0);
    }

    #[test]
    fn test_nn_new_defined() {
        let nn = NN::new_defined(&[[4, 3], [3, 2], [2, 1]]);
        assert_eq!(nn.layers.len(), 3);
        assert_eq!(nn.layers[0].num_inputs, 4);
        assert_eq!(nn.layers[0].num_neurons, 3);
        assert_eq!(nn.layers[2].num_inputs, 2);
        assert_eq!(nn.layers[2].num_neurons, 1);
    }

    #[test]
    fn test_nn_add() {
        let mut nn = NN::new();
        let layer1 = Layer::new(3, 2).unwrap();
        let layer2 = Layer::new(2, 1).unwrap();
        let layer3 = Layer::new(3, 1).unwrap();
        assert!(nn.add(layer1));
        assert!(nn.add(layer2));
        assert!(!nn.add(layer3));
    }

    #[test]
    fn test_nn_propagate() {
        let nn = NN::new_defined(&[[3, 2], [2, 1]]);
        let inputs = vec![0.0_f64, 1.0_f64];
        let outputs = nn.propagate(&inputs);
        assert_eq!(outputs, None);
        let inputs = vec![0.0_f64, 1.0_f64, 0.0_f64];
        let outputs = nn.propagate(&inputs);
        assert_ne!(outputs, None);
        let vals = outputs.unwrap();
        assert_eq!(vals.len(), 1);
        assert!(vals[0] >= 0f64);
        assert!(vals[0] <= 1f64);
    }

    #[test]
    fn test_population_new() {
        let pop = Population::new();
        assert_eq!(pop.nn.len(), 0);
    }

    #[test]
    fn test_population_new_defined() {
        let pop = Population::new_defined(10, &[[4, 3], [3, 2], [2, 1]]);
        assert_eq!(pop.length, 10);
        assert_eq!(pop.nn[0].layers.len(), 3);
        assert_eq!(pop.nn[0].layers[0].num_inputs, 4);
        assert_eq!(pop.nn[0].layers[0].num_neurons, 3);
        assert_eq!(pop.nn[9].layers[2].num_inputs, 2);
        assert_eq!(pop.nn[9].layers[2].num_neurons, 1);
    }

    #[test]
    fn test_population_add() {
        let mut pop = Population::new();
        let mut nn1 = NN::new();
        let layer1 = Layer::new(3, 2).unwrap();
        let layer2 = Layer::new(2, 1).unwrap();
        nn1.add(layer1);
        nn1.add(layer2);
        pop.add(nn1);
        assert_eq!(pop.length, 1);
        let mut nn2 = NN::new();
        let layer1 = Layer::new(3, 2).unwrap();
        let layer2 = Layer::new(2, 1).unwrap();
        nn2.add(layer1);
        nn2.add(layer2);
        pop.add(nn2);
        assert_eq!(pop.length, 2);
        assert_eq!(pop.nn.len(), 2);
        assert_eq!(pop.fitness.len(), 2);
    }

    #[test]
    fn test_population_sort() {
        let mut pop = Population::new();
        let mut nn1 = NN::new();
        let layer1 = Layer::new(3, 2).unwrap();
        let layer2 = Layer::new(2, 1).unwrap();
        nn1.add(layer1);
        nn1.add(layer2);
        let mut nn2 = NN::new();
        let layer1 = Layer::new(3, 2).unwrap();
        let layer2 = Layer::new(2, 1).unwrap();
        nn2.add(layer1);
        nn2.add(layer2);
        pop.nn.push(nn1);
        pop.nn.push(nn2);
        pop.fitness = vec![5.0, 10.0];
        let si = pop.get_sorted_index();
        assert_eq!(si, vec![1, 0]);
        let mut nn3 = NN::new();
        let layer1 = Layer::new(3, 2).unwrap();
        let layer2 = Layer::new(2, 1).unwrap();
        nn3.add(layer1);
        nn3.add(layer2);
        pop.nn.push(nn3);
        pop.fitness = vec![5.0, 10.0, 7.0];
        let si = pop.get_sorted_index();
        assert_eq!(si, vec![1, 2, 0]);
    }

    #[test]
    fn test_population_create_next_generation() {
        let mut pop = Population::new();
        let mut nn1 = NN::new();
        let layer1 = Layer::new(3, 2).unwrap();
        let layer2 = Layer::new(2, 1).unwrap();
        nn1.add(layer1);
        nn1.add(layer2);
        let mut nn2 = NN::new();
        let layer1 = Layer::new(3, 2).unwrap();
        let layer2 = Layer::new(2, 1).unwrap();
        nn2.add(layer1);
        nn2.add(layer2);
        pop.add(nn1);
        pop.add(nn2);
        pop.fitness = vec![5.0, 10.0];
        let next_gen = pop.create_next_generation();
        assert_eq!(pop.length, next_gen.length);
        let mut nn3 = NN::new();
        let layer1 = Layer::new(3, 2).unwrap();
        let layer2 = Layer::new(2, 1).unwrap();
        nn3.add(layer1);
        nn3.add(layer2);
        pop.add(nn3);
        pop.fitness = vec![5.0, 10.0, 7.0];
        let next_gen = pop.create_next_generation();
        assert_eq!(pop.length, next_gen.length);
    }
}
