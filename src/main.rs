use itertools::Itertools;

use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{
    collections::HashMap,
    io::{self, stdout},
    ops::Index,
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{
        canvas::{Canvas, Rectangle},
        *,
    },
};

struct Game {
    track: Track,
    pyramid: Pyramid,
}

#[derive(Clone)]
struct Track {
    spaces: [Vec<CamelColor>; 16],
}

impl Track {
    fn new() -> Self {
        let spaces: [Vec<CamelColor>; 16] = Default::default();
        Track { spaces }
    }

    fn advance(&mut self, color: CamelColor, number: usize) {
        let spaces = &mut self.spaces;
        let source = spaces
            .into_iter()
            .position(|space| space.contains(&color))
            .expect("couldn't find camel");

        let stack = &mut spaces[source];
        let stack_position = stack
            .into_iter()
            .position(|camel| camel == &color)
            .expect("couldn't find camel");

        let mut unit = stack.split_off(stack_position);
        let destination = source + number;
        spaces[destination].append(&mut unit);
    }

    fn print(&self) {
        self.spaces.iter().enumerate().for_each(|(index, camels)| {
            println!("{:x} : {:?}", index, camels);
        })
    }

    fn losing(&self) -> Option<CamelColor> {
        for space in self.spaces.iter() {
            if let Some(camel) = space.first() {
                return Some(*camel);
            }
        }
        None
    }
}

impl Game {
    fn new() -> Self {
        let mut pyramid = Pyramid::new();
        let mut track = Track::new();
        while let Some(roll) = pyramid.roll() {
            track.spaces[roll.number - 1].push(roll.color);
        }
        pyramid.reset();
        Game { track, pyramid }
    }

    fn roll(&mut self) -> Option<Roll> {
        let roll = self.pyramid.roll()?;
        println!("");
        println!("{:?}", roll);
        println!("");

        self.track.advance(roll.color, roll.number);

        Some(roll)
    }
}

struct Pyramid {
    dice: Vec<CamelColor>,
}

#[derive(Debug)]
struct Roll {
    color: CamelColor,
    number: usize,
}

impl Pyramid {
    fn new() -> Self {
        let mut rng = thread_rng();
        let mut dice = vec![
            CamelColor::Red,
            CamelColor::Green,
            CamelColor::Yellow,
            CamelColor::Blue,
            CamelColor::Purple,
        ];

        dice.shuffle(&mut rng);

        Pyramid { dice }
    }

    fn roll(&mut self) -> Option<Roll> {
        let color = self.dice.pop()?;
        let number: usize = thread_rng().gen_range(1..=3);
        return Some(Roll { color, number });
    }

    fn reset(&mut self) {
        let mut rng = thread_rng();
        self.dice = vec![
            CamelColor::Red,
            CamelColor::Green,
            CamelColor::Yellow,
            CamelColor::Blue,
            CamelColor::Purple,
        ];
        self.dice.shuffle(&mut rng);
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
enum CamelColor {
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Black,
    White,
}

fn main() -> io::Result<()> {
    let mut game = Game::new();
    game.track.print();

    let colors = vec![
        CamelColor::Red,
        CamelColor::Green,
        CamelColor::Blue,
        CamelColor::Yellow,
        CamelColor::Purple,
    ];
    let num_colors = colors.len();
    let color_orders = colors.into_iter().permutations(num_colors);

    let numbers: [usize; 3] = [1, 2, 3];
    let number_rolls = (0..num_colors)
        .map(|_| numbers.iter())
        .multi_cartesian_product();

    let outcomes = color_orders
        .cartesian_product(number_rolls)
        .collect::<Vec<_>>();

    let mut loser_tallies = HashMap::from([
        (CamelColor::Red, 0),
        (CamelColor::Blue, 0),
        (CamelColor::Green, 0),
        (CamelColor::Yellow, 0),
        (CamelColor::Purple, 0),
    ]);

    for outcome in outcomes.iter() {
        let mut simulation = game.track.clone();
        for roll in 0..num_colors {
            let color = outcome.0[roll];
            let number = outcome.1[roll];
            simulation.advance(color, *number);
        }
        if let Some(losing) = simulation.losing() {
            let tally = loser_tallies.get_mut(&losing).unwrap();
            *tally += 1;
        }
    }

    println!("{:?}", loser_tallies);

    Ok(())

    /*
    let mut game = Game::new();
    game.print_track();

    game.roll();

    game.print_track();

    Ok(())
    */

    /*
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    let mut pos = 1.0;
    while !should_quit {
        terminal.draw(|f| ui(f, pos))?;
        should_quit = handle_events(&mut pos)?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
    */
}

fn handle_events(pos: &mut f64) -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Up => *pos += 1.0,
                    _ => (),
                }
            }
        }
    }

    Ok(false)
}

fn ui(frame: &mut Frame, pos: f64) {
    let layout = Layout::new(
        Direction::Vertical,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(frame.size());

    frame.render_widget(
        Paragraph::new("Hello world")
            .block(Block::default().title("Gretting").borders(Borders::ALL)),
        layout[0],
    );

    frame.render_widget(
        Canvas::default()
            .block(Block::default().title("Race Track").borders(Borders::ALL))
            .x_bounds([0.0, 16.0])
            .y_bounds([0.0, 10.0])
            .paint(|ctx| {
                ctx.draw(&Rectangle {
                    x: 1.0,
                    y: pos,
                    width: 1.0,
                    height: 1.0,
                    color: Color::Red,
                });
            }),
        layout[1],
    );
}
