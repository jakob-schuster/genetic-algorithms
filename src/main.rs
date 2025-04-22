use std::time::Duration;

use genetic::{RenderState, State};

use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    widgets::{Block, Padding, Paragraph, Row, Table},
};

mod genetic;

#[derive(Parser)]
pub struct Config {
    // String to search for
    #[arg()]
    string: String,

    // Population size in each generation
    #[arg(short, long, default_value_t = 200)]
    population: usize,

    // Factor by which to multiply fitness when generating mating pool
    #[arg(short = 'f', long, default_value_t = 200)]
    mating_pool_factor: usize,

    // Rate of random mutations
    #[arg(short, long, default_value_t = 0.01)]
    mutation_rate: f32,
}

fn main() -> Result<()> {
    let config = Config::parse();

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal, &config);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, config: &Config) -> Result<()> {
    let mut state = State::new(
        &config.string,
        config.population,
        config.mating_pool_factor,
        config.mutation_rate,
    );

    loop {
        let state1 = state.update();
        terminal.draw(|frame| render(frame, &state1.get_render_state()))?;
        state = state1;

        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    event::KeyCode::Char('q') | event::KeyCode::Esc => break Ok(()),
                    _ => {}
                }
            }
        }
    }
}

fn render(frame: &mut Frame, state: &RenderState) {
    let vertical = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(15),
    ]);

    let [area_0, area_1, area_2, area_3] = vertical.areas(frame.area());

    frame.render_widget(
        "Press [q] to quit, or hold any other key to advance simulation",
        area_0,
    );

    let text = Paragraph::new(state.top_word.to_string()).block(
        Block::bordered()
            .padding(Padding::proportional(1))
            .title(" Current top phrase "),
    );
    frame.render_widget(text, area_1);

    let generations_string = state.generation.to_string();
    let fitness_string = state.average_fitness.to_string();
    let population_string = state.total_population.to_string();
    let mutation_string = format!("{}%", (state.mutation_rate * 100.0).floor());
    let text = Table::new(
        [
            Row::new(["total generations:", &generations_string]),
            Row::new(["average fitness:", &fitness_string]),
            Row::new(["total population:", &population_string]),
            Row::new(["mutation rate:", &mutation_string]),
        ],
        [20, 10],
    )
    .block(Block::new().padding(Padding::proportional(1)));
    frame.render_widget(text, area_2);

    let table = Table::new(state.top_n.iter().map(|a| Row::new([a.clone()])), [30])
        .block(Block::bordered().padding(Padding::proportional(1)));
    frame.render_widget(table, area_3);
}
