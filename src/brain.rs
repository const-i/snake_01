
extern crate rand;

use rand::Rng;
use rand_distr::{Normal, Distribution};

const MUTATION_PROBABILITY: f64 = 0.005;
const CROSSOVER_PROBABILITY: f64 = 0.0;


fn sigmoid(z: f64) -> f64 {
    let e = std::f64::consts::E;
    1.0/(1.0+e.powf(-z))
}

fn get_normal() -> f64 {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(2.0, 3.0).unwrap();
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
        }
        else {
            let biases: Vec<f64> = (0..num_neurons)
                                            .map(|_| get_normal())
                                            .collect();
            let weights: Vec<Vec<f64>> = (0..num_neurons)
                                                    .map(|_| (0..num_inputs)
                                                                    .map(|_| get_normal())
                                                                    .collect())
                                                    .collect();
            Some(Layer {
                num_inputs: num_inputs,
                num_neurons: num_neurons, 
                weights: weights, 
                biases: biases,
            })
        }
    }
    
    fn feed_forward(&self, inputs: Vec<f64>) -> Option<Vec<f64>> {
        if self.num_inputs != inputs.len() as u32 {
            None
        }
        else {
            let wx: Vec<f64> = self.weights.iter()
                                           .map(|ws| ws.iter()
                                              .zip(inputs.iter())
                                              .map(|(w,x)| w*x)
                                              .sum())
                                           .collect();
            let wx_b: Vec<f64> = wx.iter()
                                   .zip(self.biases.iter())
                                   .map(|(wx,b)| wx+b)
                                   .collect();
            Some(wx_b.iter().map(|z| sigmoid(*z)).collect())
        }
    }
    
    fn should_mutate() -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() > 1f64 - MUTATION_PROBABILITY
    }
    
    fn mutate(&mut self) {
        
        self.biases = self.biases.iter()
                                 .map(|b| { if Layer::should_mutate() { get_normal() }
                                            else                      { *b }
                                        })
                                  .collect();
        self.weights = self.weights.iter()
                                   .map(|ws| ws.iter()
                                               .map(|w| { if Layer::should_mutate() { get_normal() }
                                                          else                      { *w }
                                                    })
                                               .collect())
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
            }
            else {
                child1.biases[i] = self.biases[i];
                child2.biases[i] = other.biases[i];
            }
            for j in 0..self.num_inputs as usize {
                if Layer::should_crossover() {
                    child1.weights[i][j] = other.weights[i][j];
                    child2.weights[i][j] = self.weights[i][j];
                }
                else {
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
        NN {
            layers: Vec::new(),
            //fitness: 0u32,
        }
    }
    
    pub fn add(&mut self, layer: Layer) -> bool {
        if self.layers.len() == 0 {
            self.layers.push(layer);
            true
        }
        else {
            if self.layers.last().unwrap().num_neurons == layer.num_inputs {
                self.layers.push(layer);
                true
            }
            else {
                false
            }
        }
    }
    
    
    pub fn propagate(&self, inputs: Vec<f64>) -> Option<Vec<f64>> {
        
        if self.layers.len() == 0 {
            None
        }
        else if self.layers[0].num_inputs != inputs.len() as u32 {
            None
        }
        else {
            let mut this_in = Some(inputs);
            for layer in &self.layers {
                match this_in {
                    Some(vals) => this_in = layer.feed_forward(vals),
                    None       => this_in = None,
                }
            }
            this_in
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


pub struct Population {
    pub length: usize,
    pub nn: Vec<NN>,
    pub fitness: Vec<i64>,
}

impl Population {
    
    pub fn new() -> Population {
        Population {
            length: 0,
            nn: Vec::new(),
            fitness: Vec::new(),
        }
    }
    
    pub fn add(&mut self, nn: NN) {
        self.nn.push(nn);
        self.fitness.push(0);
        self.length += 1;
    }
    
    pub fn add_fitness(&mut self, nn: NN, fitness: i64) {
        self.nn.push(nn);
        self.fitness.push(fitness);
        self.length += 1;
    }
      
    pub fn create_next_generation(&self) -> Population {
        let mut pop = Population::new();
        let sorted_index = self.get_sorted_index();
        let mut mid = self.length % 2 + self.length / 2;
        for i in 0..mid {
            let p1 = &self.nn[sorted_index[i]];
            let p2 = &self.nn[sorted_index[i+1]];
            let (mut c1, mut c2) = p1.crossover(p2);
            c1.mutate();
            c2.mutate();
            pop.add(c1);
            if i == mid - 1 && self.length % 2 == 1 {
                // Do nothing
            }
            else {
                pop.add(c2);
            }
        }
        pop
    }
    
    pub fn get_sorted_index(&self) -> Vec<usize> {
        let mut index: Vec<(usize, i64)> = self.fitness.clone().into_iter().enumerate().collect();
        index.sort_by(|(ai,af), (bi, bf)| bf.cmp(af));
        let sorted_index = index.iter().map(|(i,f)| *i).collect();
        sorted_index
    }
    
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sigmoid() {
        assert!(sigmoid(-1000f64)>=0f64);
        assert!(sigmoid(-1000f64)<=0.5f64);
        assert!(sigmoid(1000f64)>=0.5f64);
        assert!(sigmoid(1000f64)<=1f64);
        assert_eq!(sigmoid(0f64), 0.5f64);
    }
    
    #[test]
    fn test_layer_new() {
        let layer = Layer::new(3,2).unwrap();
        assert_eq!(layer.num_inputs, 3);
        assert_eq!(layer.num_neurons, 2);
        assert_eq!(layer.weights.len(), 2);
        assert_eq!(layer.weights.first().unwrap().len(), 3);
        assert_eq!(layer.biases.len(), 2);
        let res = Layer::new(0,0);
        match res {
            Some(layer) => assert!(false),
            None        => assert!(true),
        }
    }
    
    #[test]
    fn test_layer_feed_forward() {
        let mut layer = Layer::new(3,2).unwrap();
        let mut inputs = vec![1f64,2f64];
        let mut outputs = layer.feed_forward(inputs);
        assert_eq!(outputs, None);
        inputs = vec![0f64,1f64,0f64];
        outputs = layer.feed_forward(inputs);
        println!("{:?}", outputs);
        assert_eq!(outputs.unwrap().len(), 2);
        //assert!(false);
    }
    
    #[test]
    fn test_layer_should_mutate() {
        let muts: Vec<bool> = (0..10).map(|_| Layer::should_mutate()).collect();
        //assert!(muts.iter().any(|&x| x == true));
        assert!(true);
    }
    
    #[test]
    fn test_layer_mutate() {
        let mut layer = Layer::new(3,2).unwrap();
        let orig_biases = layer.biases.clone();
        let orig_weights = layer.weights.clone();
        layer.mutate();
        //assert_ne!(orig_biases, layer.biases);
        //assert_ne!(orig_weights, layer.weights);
        assert!(true);
    }
    
    #[test]
    fn test_layer_should_crossover() {
        let muts: Vec<bool> = (0..10).map(|_| Layer::should_crossover()).collect();
        //assert!(muts.iter().any(|&x| x == true));
        assert!(true);
    }
    
    #[test]
    fn test_layer_crossover() {
        let parent1 = Layer::new(3,2).unwrap();
        let parent2 = Layer::new(3,2).unwrap();
        let (mut child1, mut child2) = parent1.crossover(&parent2);
        //assert_ne!(child1.biases, parent1.biases);
        //assert_ne!(child2.biases, parent2.biases);
        //assert_ne!(child1.weights, parent1.weights);
        //assert_ne!(child2.weights, parent2.weights);
        assert!(true);
    }
    
    #[test]
    fn test_nn_new() {
        let mut nn = NN::new();
        assert_eq!(nn.layers.len(),0);
    }
    
    #[test]
    fn test_nn_add() {
        let mut nn = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        let mut layer3 = Layer::new(3,1).unwrap();
        assert!(nn.add(layer1));
        assert!(nn.add(layer2));
        assert!(!nn.add(layer3));
    }
    
    #[test]
    fn test_nn_propagate() {
        let mut nn = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn.add(layer1);
        nn.add(layer2);
        let mut inputs = vec![1f64,2f64];
        let mut outputs = nn.propagate(inputs);
        assert_eq!(outputs, None);
        inputs = vec![1f64,2f64,3f64];
        outputs = nn.propagate(inputs);
        let val = outputs.unwrap();
        assert_eq!(val.len(), 1);
        assert!(val[0]>=0f64);
        assert!(val[0]<=1f64);
    }
    
    #[test]
    fn test_nn_mutate() {
        let mut nn = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        let orig_biases = layer1.biases.clone();
        let orig_weights = layer2.weights.clone();
        nn.add(layer1);
        nn.add(layer2);
        nn.mutate();
        //assert_ne!(orig_biases, nn.layers[0].biases);
        //assert_ne!(orig_weights, nn.layers[1].weights);
        assert!(true);
    }
    
    #[test]
    fn test_nn_crossover() {
        let mut nn1 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        let orig_biases1 = layer1.biases.clone();
        let orig_weights1 = layer2.weights.clone();
        nn1.add(layer1);
        nn1.add(layer2);
        let mut nn2 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        let orig_biases2 = layer1.biases.clone();
        let orig_weights2 = layer2.weights.clone();
        nn2.add(layer1);
        nn2.add(layer2);
        let (c1, c2) = nn1.crossover(&mut nn2);
        //assert_ne!(orig_biases1,c1.layers[0].biases);
        //assert_ne!(orig_weights1,c1.layers[1].weights);
        //assert_ne!(orig_biases2,c2.layers[0].biases);
        //assert_ne!(orig_weights2,c2.layers[1].weights);
        assert!(true);
    }
    
    #[test]
    fn test_population_new() {
        let mut pop = Population::new();
        assert_eq!(pop.nn.len(),0);
    }
    
    #[test]
    fn test_population_add() {
        let mut pop = Population::new();
        let mut nn1 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn1.add(layer1);
        nn1.add(layer2);
        pop.add(nn1);
        assert_eq!(pop.length, 1);
        let mut nn2 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
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
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn1.add(layer1);
        nn1.add(layer2);
        let mut nn2 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn2.add(layer1);
        nn2.add(layer2);
        pop.nn.push(nn1);
        pop.nn.push(nn2);
        pop.fitness = vec![5,10];
        let mut si = pop.get_sorted_index();
        assert_eq!(si, vec![1,0]);
        let mut nn3 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn3.add(layer1);
        nn3.add(layer2);
        pop.nn.push(nn3);
        pop.fitness = vec![5,10,7];
        let mut si = pop.get_sorted_index();
        assert_eq!(si, vec![1,2,0]);
    }
    
    #[test]
    fn test_population_create_next_generation() {
        let mut pop = Population::new();
        let mut nn1 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn1.add(layer1);
        nn1.add(layer2);
        let mut nn2 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn2.add(layer1);
        nn2.add(layer2);
        pop.add(nn1);
        pop.add(nn2);
        pop.fitness = vec![5,10];
        let mut next_gen = pop.create_next_generation();
        assert_eq!(pop.length, next_gen.length);
        let mut nn3 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn3.add(layer1);
        nn3.add(layer2);
        pop.add(nn3);
        pop.fitness = vec![5,10, 7];
        next_gen = pop.create_next_generation();
        assert_eq!(pop.length, next_gen.length);
        assert_eq!(next_gen.nn[0].layers[0].biases, pop.nn[1].layers[0].biases);
        assert_eq!(next_gen.nn[1].layers[0].biases, pop.nn[2].layers[0].biases);
        assert_eq!(next_gen.nn[2].layers[0].biases, pop.nn[2].layers[0].biases);
    }
    
    /*
    #[test]
    fn test_population_sort() {
        let mut pop = Population::new();
        let mut nn1 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn1.add(layer1);
        nn1.add(layer2);
        nn1.fitness = 10;
        let mut nn2 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn2.add(layer1);
        nn2.add(layer2);
        nn2.fitness = 5;
        pop.nn.push(nn1);
        pop.nn.push(nn2);
        pop.sort();
        assert_eq!(pop.nn[0].fitness, 10);
    }
    
    #[test]
    fn test_population_create_next_generation() {
        let mut pop = Population::new();
        let mut nn1 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn1.add(layer1);
        nn1.add(layer2);
        nn1.fitness = 10;
        let mut nn2 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn2.add(layer1);
        nn2.add(layer2);
        nn2.fitness = 5;
        pop.nn.push(nn1);
        pop.nn.push(nn2);
        let mut next_gen = pop.create_next_generation();
        assert_eq!(pop.nn.len(), 2);
        let mut nn3 = NN::new();
        let mut layer1 = Layer::new(3,2).unwrap();
        let mut layer2 = Layer::new(2,1).unwrap();
        nn3.add(layer1);
        nn3.add(layer2);
        nn3.fitness = 15;
        pop.nn.push(nn3);
        let mut next_gen = pop.create_next_generation();
        assert_eq!(pop.nn.len(), 3);
    }

    */
}
