mod base;
mod cell;
mod collector;
mod map;
mod message;
mod position;
mod resource;
mod robot;
mod simulation;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;

use map::Map;
use simulation::Simulation;

const NUM_SCOUTS: usize = 3;
const NUM_COLLECTORS: usize = 3;
const POLL_INTERVAL: Duration = Duration::from_millis(80);

fn main() -> io::Result<()> {
    let map = Map::new(80, 30);
    let base_position = map.base_position;

    let mut simulation = Simulation::new(map, base_position, NUM_SCOUTS, NUM_COLLECTORS);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &mut simulation);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    simulation.stop();

    result
}

fn run<B: Backend>(terminal: &mut Terminal<B>, simulation: &mut Simulation) -> io::Result<()> {
    loop {
        simulation.process_messages();

        {
            let map_guard = simulation.map.read().unwrap();
            let base_guard = simulation.base.lock().unwrap();
            let views_guard = simulation.robot_views.lock().unwrap();

            terminal.draw(|frame| {
                ui::render(frame, &map_guard, &base_guard, &views_guard);
            })?;
        }

        if event::poll(POLL_INTERVAL)? {
            if let Event::Key(key_event) = event::read()? {
                // Sous Windows, la console envoie aussi les événements de relâchement de touche
                // (ex: l'Entrée utilisée pour lancer `cargo run`) : on ignore tout sauf l'appui réel.
                if key_event.kind == KeyEventKind::Press {
                    break;
                }
            }
        }
    }

    Ok(())
}
