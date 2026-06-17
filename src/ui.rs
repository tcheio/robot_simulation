use std::collections::HashMap;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::base::Base;
use crate::cell::CellType;
use crate::map::Map;
use crate::resource::ResourceType;
use crate::simulation::{RobotKind, RobotView};

pub fn render(frame: &mut Frame, map: &Map, base: &Base, robot_views: &HashMap<usize, RobotView>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(map.height as u16 + 2),
            Constraint::Min(8),
        ])
        .split(area);

    frame.render_widget(render_map(map, robot_views), chunks[0]);
    frame.render_widget(render_stats(base, robot_views), chunks[1]);
}

fn render_map(map: &Map, robot_views: &HashMap<usize, RobotView>) -> Paragraph<'static> {
    let mut overlay: HashMap<(usize, usize), (char, Color)> = HashMap::new();
    for view in robot_views.values() {
        let (ch, color) = match view.kind {
            RobotKind::Scout => ('x', Color::Red),
            RobotKind::Collector => ('o', Color::Magenta),
        };
        overlay.insert((view.position.x, view.position.y), (ch, color));
    }

    let mut lines = Vec::with_capacity(map.height);
    for y in 0..map.height {
        let mut spans = Vec::with_capacity(map.width);
        for x in 0..map.width {
            let (ch, color) = if let Some((c, col)) = overlay.get(&(x, y)) {
                (*c, *col)
            } else {
                match &map.cells[y][x].cell_type {
                    CellType::Empty => ('.', Color::DarkGray),
                    CellType::Obstacle => ('O', Color::LightCyan),
                    CellType::Base => ('#', Color::LightGreen),
                    CellType::Resource(resource) => match resource.resource_type {
                        ResourceType::Energy => ('E', Color::Green),
                        ResourceType::Crystal => ('C', Color::LightMagenta),
                    },
                }
            };
            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
        lines.push(Line::from(spans));
    }

    Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Carte"))
}

fn render_stats(base: &Base, robot_views: &HashMap<usize, RobotView>) -> Paragraph<'static> {
    let scouts = robot_views
        .values()
        .filter(|v| v.kind == RobotKind::Scout)
        .count();
    let collectors = robot_views
        .values()
        .filter(|v| v.kind == RobotKind::Collector)
        .count();

    let mut lines = vec![
        Line::from(format!("Énergie collectée   : {}", base.energy)),
        Line::from(format!("Cristaux collectés   : {}", base.crystals)),
        Line::from(format!("Robots éclaireurs    : {}", scouts)),
        Line::from(format!("Robots collecteurs   : {}", collectors)),
        Line::from(format!(
            "Obstacles connus : {}  |  Ressources connues : {}",
            base.known_obstacles.len(),
            base.known_resources.len()
        )),
        Line::from(""),
        Line::from("Appuyez sur une touche pour quitter"),
    ];

    for log in base.logs.iter().rev().take(4) {
        lines.push(Line::from(log.clone()));
    }

    Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Statistiques"))
}
