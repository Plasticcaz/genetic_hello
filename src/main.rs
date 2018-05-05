extern crate rand;

use rand::Rng;
use rand::distributions::{Range, IndependentSample};

const GOAL: &[u8] = br#"
1 Corinthians 13 King James Version (KJV)
13 Though I speak with the tongues of men and of angels, and have not charity, I am become as sounding brass, or a tinkling cymbal.

2 And though I have the gift of prophecy, and understand all mysteries, and all knowledge; and though I have all faith, so that I could remove mountains, and have not charity, I am nothing.

3 And though I bestow all my goods to feed the poor, and though I give my body to be burned, and have not charity, it profiteth me nothing.

4 Charity suffereth long, and is kind; charity envieth not; charity vaunteth not itself, is not puffed up,

5 Doth not behave itself unseemly, seeketh not her own, is not easily provoked, thinketh no evil;

6 Rejoiceth not in iniquity, but rejoiceth in the truth;

7 Beareth all things, believeth all things, hopeth all things, endureth all things.

8 Charity never faileth: but whether there be prophecies, they shall fail; whether there be tongues, they shall cease; whether there be knowledge, it shall vanish away.

9 For we know in part, and we prophesy in part.

10 But when that which is perfect is come, then that which is in part shall be done away.

11 When I was a child, I spake as a child, I understood as a child, I thought as a child: but when I became a man, I put away childish things.

12 For now we see through a glass, darkly; but then face to face: now I know in part; but then shall I know even as also I am known.

13 And now abideth faith, hope, charity, these three; but the greatest of these is charity.
"#;

const MAX_GENERATIONS: usize = 100_000;
const POPULATION_SIZE: usize = 100_000;

const ELITE_SELECTION_RATE: f64 = 30.0/100.0;
const CROSSOVER_RATE: f64 = 40.0/100.0;
// The above two variables should add up to 100.

const MUTATION_RATE: f64 = 60.0/100.0;

#[derive(Debug, Clone)]
struct Gene {
    data: Vec<u8>,
    score: Option<usize>,
}

impl Gene {
    pub fn new(data: Vec<u8>) -> Gene {
        Self {
            data,
            score: None
        }
    }

    pub fn score(&self) -> usize {
        self.score.expect("calc_score() should have been called first.")
    }

    pub fn crossover<R: Rng>(&self, b: &Gene, rng: &mut R) -> Gene {
        let a = self;
        let data = a.data.iter().zip(b.data.iter()).map(|(a, b)| {
            if rng.gen() {
                *a
            } else {
                *b
            }
        }).collect();
        Gene::new(data)
    }

    pub fn mutate<R: Rng>(&mut self, rng: &mut R) {
        let index: usize = Range::new(0, self.data.len()).ind_sample(rng);

        self.data[index] = Range::new(0, 126).ind_sample(rng);
    }

    pub fn cmp(&self, other: &Gene) -> std::cmp::Ordering {
        self.score().cmp(&other.score())
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    pub fn calc_score(&mut self, goal: &[u8]) {
        if self.score.is_some() {
            return;
        }

        let score = if self.data.len() != goal.len() || self.as_str().is_err() {
            std::usize::MAX
        } else {
            let mut score = 0;
            for (got, expected) in self.data.iter().zip(goal.iter()) {
                if got != expected {
                    score += 1;
                }
            }
            score
        };

        self.score = Some(score)
    }
}

fn generate_random_genes<R: Rng>(n: usize, rng: &mut R) -> Vec<Gene> {
    let mut genes = Vec::with_capacity(n);

    while genes.len() < n {
        // let len = Range::new(1, 10_000).ind_sample(rng);
        let len = GOAL.len();
        let gene = rng.gen_ascii_chars().map(|c| c as u8).take(len).collect();
        genes.push(Gene::new(gene));
    }
    genes
}

fn main() {
    let mut rng = rand::thread_rng();
    let mut population = generate_random_genes(POPULATION_SIZE, &mut rng);
    population.iter_mut().for_each(|gene| gene.calc_score(GOAL));
    population.sort_by(|gene1, gene2| gene1.cmp(gene2));

    let mut children = Vec::with_capacity(population.len());

    let mut generation = 0;

    let elite_selection = (population.len() as f64 * ELITE_SELECTION_RATE) as usize;
    let crossover_amount = (population.len() as f64 * CROSSOVER_RATE) as usize;
    let mutation_amount = (population.len() as f64 * MUTATION_RATE) as usize;

    while generation < MAX_GENERATIONS && population[0].score() != 0 {
        children.clear();
        // Crossover -- Try to combine the effects.
        for _ in 0..crossover_amount {
            let range = Range::new(0, population.len());

            let a = &population[range.ind_sample(&mut rng)];
            let b = &population[range.ind_sample(&mut rng)];

            children.push(a.crossover(b, &mut rng));
        }

        // Elite Selection -- try not to lose the top players.
        population.drain(0..elite_selection).for_each(|gene| children.push(gene));

        // Mutation:
        for _ in 0..Range::new(0, mutation_amount).ind_sample(&mut rng) {
            let index = Range::new(0, children.len()).ind_sample(&mut rng);
            children[index].mutate(&mut rng);
        }
        
        // Sort the results:
        children.iter_mut().for_each(|gene| gene.calc_score(GOAL));
        children.sort_by(|gene1, gene2| gene1.cmp(gene2));

        let temp = population;
        population = children;
        children = temp;
        generation += 1;
        println!("G: {}\nBest:\n'{}'\nscore: {}", generation, population[0].as_str().unwrap(), population[0].score());
    }
    println!("SUCCEDED: G: {} Best: '{}'", generation, population[0].as_str().unwrap());
}
