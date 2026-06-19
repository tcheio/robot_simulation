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
            Constraint::Min(14),
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

    let label = Style::default().fg(Color::Gray);
    let val_energy = Style::default().fg(Color::Green);
    let val_crystal = Style::default().fg(Color::LightMagenta);
    let val_scout = Style::default().fg(Color::Red);
    let val_collector = Style::default().fg(Color::Magenta);
    let val_neutral = Style::default().fg(Color::White);
    let hint = Style::default().fg(Color::DarkGray);
    let log_style = Style::default().fg(Color::Yellow);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Énergie collectée    : ", label),
            Span::styled(base.energy.to_string(), val_energy),
        ]),
        Line::from(vec![
            Span::styled("Cristaux collectés   : ", label),
            Span::styled(base.crystals.to_string(), val_crystal),
        ]),
        Line::from(vec![
            Span::styled("Robots éclaireurs    : ", label),
            Span::styled(scouts.to_string(), val_scout),
        ]),
        Line::from(vec![
            Span::styled("Robots collecteurs   : ", label),
            Span::styled(collectors.to_string(), val_collector),
        ]),
        Line::from(vec![
            Span::styled("Obstacles connus : ", label),
            Span::styled(base.known_obstacles.len().to_string(), val_neutral),
            Span::styled("  |  Ressources connues : ", label),
            Span::styled(base.known_resources.len().to_string(), val_neutral),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("── Logs ──────────────────────────────────────────", Style::default().fg(Color::DarkGray))]),
    ];

    if base.logs.is_empty() {
        lines.push(Line::from(vec![Span::styled("  (aucun log pour l'instant)", Style::default().fg(Color::DarkGray))]));
    } else {
        for log in base.logs.iter().rev().take(4) {
            lines.push(Line::from(vec![Span::styled(log.clone(), log_style)]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("Appuyez sur une touche pour quitter", hint)]));

    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Statistiques", Style::default().fg(Color::Cyan))),
    )
}
