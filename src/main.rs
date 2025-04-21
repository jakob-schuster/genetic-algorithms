use std::{ops::Index, str::from_utf8};

use itertools::Itertools;
use rand::{
    Rng, random_bool,
    rngs::ThreadRng,
    seq::{IndexedRandom, IteratorRandom, SliceRandom},
};

type Element = String;

/// Generate a new element totally randomly
fn generate(length: usize, rng: &mut ThreadRng) -> Element {
    let mut a = ('a'..='z').choose_multiple(rng, length);
    a.shuffle(rng);
    a.iter().collect()
}

/// Calculate the fitness of an element, against a target
fn absolute_fitness(target: &Element, guess: &Element) -> f32 {
    // hamming distance
    let mut matching = 0;
    for i in 0..target.len() {
        if target.as_bytes()[i] == guess.as_bytes()[i] {
            matching += 1;
        }
    }

    matching as f32 / target.len() as f32
}

/// Two elements reproduce to make a new element
fn reproduce(rng: &mut ThreadRng, e1: &Element, e2: &Element) -> Element {
    let mut new_element = vec![];
    for i in 0..e1.len() {
        new_element.push(*[e1.as_bytes()[i], e2.as_bytes()[i]].choose(rng).unwrap());
    }
    String::from_utf8(new_element).unwrap()
}

/// Random mutations are applied to an element
fn mutate(rng: &mut ThreadRng, element: &Element, mutation_rate: f32) -> Element {
    let mut new_element = vec![];
    for c in element.chars() {
        new_element.push(if random_bool(mutation_rate as f64) {
            // pick a random letter
            ('a'..='z').choose(rng).unwrap()
        } else {
            c
        });
    }

    new_element.iter().collect()
}

fn normalise_fitnesses(population: &[(Element, f32)]) -> Vec<(Element, f32)> {
    let sum = population
        .iter()
        .fold(0.0, |acc, (_, fitness)| acc + fitness);

    population
        .iter()
        .map(|(element, fitness)| (element.clone(), *fitness / sum))
        .collect()
}

fn generate_new_population(
    rng: &mut ThreadRng,
    population: &[(Element, f32)],
    mating_pool_size: usize,
    population_size: usize,
    mutation_rate: f32,
) -> Vec<Element> {
    let mating_pool = population
        .iter()
        // .inspect(|(element, fitness)| {
        //     println!(
        //         "repeating {} with fitness {} {} times",
        //         element,
        //         fitness,
        //         (fitness * mating_pool_size as f32).ceil() as usize
        //     )
        // })
        .flat_map(|(element, fitness)| {
            [element].repeat((fitness * mating_pool_size as f32).ceil() as usize)
        })
        .collect::<Vec<_>>();

    println!("mating pool size {}", mating_pool.len());

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

fn average_absolute_fitness(target: &Element, population: &[Element]) -> f32 {
    population
        .iter()
        .fold(0.0, |acc, guess| acc + absolute_fitness(target, guess))
        / population.len() as f32
}

fn top(target: &Element, population: &[Element], n: usize) -> Vec<Element> {
    let mut sorted_population = population.to_vec();
    sorted_population
        .sort_by_key(|element| -(absolute_fitness(target, element) * 1000.0).round() as i32);

    sorted_population.iter().unique().take(n).cloned().collect()
}

fn main() {
    let mut rng = rand::rng();

    let target: Element = Element::from("beepbopskeepskop");

    let population_size = 100;
    let mating_pool_size = 100;
    let mutation_rate = 0.001;

    let mut population = (0..population_size)
        .map(|_| generate(target.len(), &mut rng))
        .collect::<Vec<_>>();

    for i in 0..1000 {
        println!(
            "generation {}. average fitness {}",
            i,
            average_absolute_fitness(&target, &population)
        );

        for element in top(&target, &population, 10) {
            println!("{}", element);
        }

        let population_with_fitnesses = population
            .iter()
            .map(|element| (element.clone(), absolute_fitness(&target, element)))
            .collect::<Vec<_>>();
        // let population_with_relative_fitnesses = normalise_fitnesses(&population_with_fitnesses);

        population = generate_new_population(
            &mut rng,
            &population_with_fitnesses,
            mating_pool_size,
            population_size,
            mutation_rate,
        );
    }

    println!("final answer {}", population.choose(&mut rng).unwrap());
}
