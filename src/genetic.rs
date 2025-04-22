use itertools::Itertools;
use rand::{
    random_bool,
    rngs::ThreadRng,
    seq::{IndexedRandom, IteratorRandom, SliceRandom},
};

/// The type of the basic element to be evolved
type Element = String;

/// Generate a new element totally randomly
fn generate(length: usize, rng: &mut ThreadRng) -> Element {
    let mut a = (' '..='~').choose_multiple(rng, length);
    a.shuffle(rng);
    a.iter().collect()
}

/// Calculate the fitness of an element, against a target
fn absolute_fitness(target: &Element, guess: &Element) -> f32 {
    // just naively calculate Hamming distance
    let mut matching = 0;
    for i in 0..target.len() {
        if target.as_bytes()[i] == guess.as_bytes()[i] {
            matching += 1;
        }
    }

    matching as f32 / target.len() as f32
}

/// Two elements reproduce to make a new element.
fn reproduce(rng: &mut ThreadRng, e1: &Element, e2: &Element) -> Element {
    // each character is selected randomly from each element
    let mut new_element = vec![];
    for i in 0..e1.len() {
        new_element.push(*[e1.as_bytes()[i], e2.as_bytes()[i]].choose(rng).unwrap());
    }
    String::from_utf8(new_element).unwrap()
}

/// Random mutations are applied to an element, based on the mutation rate
fn mutate(rng: &mut ThreadRng, element: &Element, mutation_rate: f32) -> Element {
    let mut new_element = vec![];
    for c in element.chars() {
        new_element.push(if random_bool(mutation_rate as f64) {
            // pick a random character
            (' '..='~').choose(rng).unwrap()
        } else {
            c
        });
    }

    new_element.iter().collect()
}

/// Normalise the fitness across the population
fn normalise_fitnesses(population: &[(Element, f32)]) -> Vec<(Element, f32)> {
    let sum = population
        .iter()
        .fold(0.0, |acc, (_, fitness)| acc + fitness);

    population
        .iter()
        .map(|(element, fitness)| (element.clone(), *fitness / sum))
        .collect()
}

/// Generate a new population,
/// by producing a mating pool proportionate to the fitness of each element
/// and then randomly selecting elements from the mating pool to reproduce
/// and applying mutations to each child.
fn generate_new_population(
    rng: &mut ThreadRng,
    population: &[(Element, f32)],
    mating_pool_size: usize,
    population_size: usize,
    mutation_rate: f32,
) -> Vec<Element> {
    // generate the mating pool proportionate to the fitness levels
    let mating_pool = population
        .iter()
        .flat_map(|(element, fitness)| {
            [element].repeat((fitness * mating_pool_size as f32).ceil() as usize)
        })
        .collect::<Vec<_>>();

    // mate until the population limit is reached
    let mut new_population = vec![];
    for _ in 0..population_size {
        let a = mating_pool.choose(rng).unwrap();
        let mut b = mating_pool.choose(rng).unwrap();
        while b == a {
            b = mating_pool.choose(rng).unwrap();
        }

        let c = reproduce(rng, a, b);
        new_population.push(mutate(rng, &c, mutation_rate));
    }

    new_population
}

/// Calculate the average absolute fitness in a population
fn average_absolute_fitness(target: &Element, population: &[Element]) -> f32 {
    population
        .iter()
        .fold(0.0, |acc, guess| acc + absolute_fitness(target, guess))
        / population.len() as f32
}

/// Return the top n fittest elements in a population
fn top(target: &Element, population: &[Element], n: usize) -> Vec<Element> {
    let mut sorted_population = population.to_vec();
    sorted_population
        .sort_by_key(|element| -(absolute_fitness(target, element) * 1000.0).round() as i32);

    sorted_population.iter().unique().take(n).cloned().collect()
}

/// The internal state of the program
pub struct State {
    pub target: Element,
    pub population: Vec<Element>,
    pub mating_pool_size: usize,
    pub mutation_rate: f32,
    pub generation: usize,
    pub rng: ThreadRng,
}

impl State {
    /// Generate a new state, with a randomly generated population
    pub fn new(
        target: &Element,
        population_size: usize,
        mating_pool_size: usize,
        mutation_rate: f32,
    ) -> State {
        let mut rng = rand::rng();

        State {
            target: target.clone(),
            population: (0..population_size)
                .map(|_| generate(target.len(), &mut rng))
                .collect::<Vec<_>>(),
            mating_pool_size,
            mutation_rate,
            generation: 0,
            rng,
        }
    }

    /// Calculate the fitnesses of the population,
    /// and then reproduce to generate a new population,
    /// resulting in the next state
    pub fn update(&mut self) -> State {
        let population_with_fitnesses = self
            .population
            .iter()
            .map(|element| (element.clone(), absolute_fitness(&self.target, element)))
            .collect::<Vec<_>>();

        let new_population = generate_new_population(
            &mut self.rng,
            &population_with_fitnesses,
            self.mating_pool_size,
            self.population.len(),
            self.mutation_rate,
        );

        State {
            target: self.target.clone(),
            population: new_population,
            mating_pool_size: self.mating_pool_size,
            mutation_rate: self.mutation_rate,
            generation: self.generation + 1,
            rng: self.rng.clone(),
        }
    }

    /// Convert the state to include only the information necessary for rendering
    pub fn get_render_state(&self) -> RenderState {
        RenderState {
            top_word: self
                .population
                .iter()
                .sorted_by_key(|element| (-absolute_fitness(&self.target, element) * 100.0) as i32)
                .next()
                .unwrap()
                .clone(),
            generation: self.generation,
            average_fitness: average_absolute_fitness(&self.target, &self.population),
            total_population: self.population.len(),
            mutation_rate: self.mutation_rate,
            top_n: top(&self.target, &self.population, 10),
        }
    }
}

/// A state which contains only the information necessary for rendering
pub struct RenderState {
    pub top_word: String,
    pub generation: usize,
    pub average_fitness: f32,
    pub total_population: usize,
    pub mutation_rate: f32,
    pub top_n: Vec<String>,
}
