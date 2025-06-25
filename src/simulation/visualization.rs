use crate::simulation::entities::{Map, ResourceType};
use crate::simulation::map::TerrainType;
use crate::simulation::robot_ai::robot::Robot;
use crate::simulation::robot_ai::types::RobotType;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Utility for visualizing the map in the terminal
pub struct MapVisualizer;

impl MapVisualizer {
    // /// Visualizes the map in the terminal
    // pub fn visualize(map: &Map) -> Result<(), Box<dyn std::error::Error>> {
    //     Self::visualize_with_robots(map, &[])
    // }

    // pub fn visualize_with_robots(
    //     map: &Map,
    //     robots: &[Robot],
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     if crossterm::terminal::is_raw_mode_enabled()? {
    //         Self::visualize_tui(map, robots)
    //     } else {
    //         Self::visualize_fallback(map, robots);
    //         Ok(())
    //     }
    // }

    // fn visualize_tui(map: &Map, robots: &[Robot]) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut stdout = io::stdout();
    //     crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    //     crossterm::terminal::enable_raw_mode()?;

    //     let backend = CrosstermBackend::new(stdout);
    //     let mut terminal = Terminal::new(backend)?;

    //     let mut app = App::new(map, robots);
    //     let res = Self::run_app(&mut terminal, &mut app);

    //     crossterm::terminal::disable_raw_mode()?;
    //     crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    //     terminal.show_cursor()?;

    //     if let Err(err) = res {
    //         println!("{:?}", err);
    //     }

    //     Ok(())
    // }

    // fn visualize_fallback(map: &Map, robots: &[Robot]) {
    //     println!("Map Visualization ({}x{})", map.width, map.height);
    //     println!("Legend: . = Plain, ^ = Hill, # = Mountain, R = Robot, S = Station");
    //     println!();

    //     for y in 0..map.height {
    //         for x in 0..map.width {
    //             let mut cell = match map.terrain[y][x] {
    //                 0 => '.',
    //                 1 => '^',
    //                 2 => '#',
    //                 _ => '?',
    //             };

    //             // Check if there's a robot at this position
    //             for robot in robots {
    //                 if robot.position() == (x, y) {
    //                     cell = 'R';
    //                     break;
    //                 }
    //             }

    //             print!("{}", cell);
    //         }
    //         println!();
    //     }
    //     println!();
    // }

    // fn run_app<B: ratatui::backend::Backend>(
    //     terminal: &mut Terminal<B>,
    //     app: &mut App,
    // ) -> io::Result<()> {
    //     loop {
    //         terminal.draw(|f| ui(f, app))?;

    //         if crossterm::event::poll(std::time::Duration::from_millis(250))? {
    //             if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
    //                 match key.code {
    //                     crossterm::event::KeyCode::Char('q') => return Ok(()),
    //                     crossterm::event::KeyCode::Up => app.scroll_up(),
    //                     crossterm::event::KeyCode::Down => app.scroll_down(),
    //                     crossterm::event::KeyCode::Left => app.scroll_left(),
    //                     crossterm::event::KeyCode::Right => app.scroll_right(),
    //                     _ => {}
    //                 }
    //             }
    //         }
    //     }
    // }

    pub fn ui(f: &mut Frame, app: &App) {
        let stats_height = if app.map.width * app.map.height > 400 {
            12
        } else {
            11
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(stats_height)])
            .split(f.area());

        Self::render_map(f, chunks[0], app);
        Self::render_stats(f, chunks[1], app);
    }

    fn render_map(f: &mut Frame, area: Rect, app: &App) {
        let map_height = area.height as usize - 2;
        let map_width = area.width as usize - 2;

        let cell_width = 2;
        let visible_height = map_height.min(app.map.height);
        let visible_width = (map_width / cell_width).min(app.map.width);

        let mut start_y = app
            .scroll_y
            .min(app.map.height.saturating_sub(visible_height));
        let mut start_x = app
            .scroll_x
            .min(app.map.width.saturating_sub(visible_width));

        let mut center_y = 0;
        let mut center_x = 0;

        if app.map.height < visible_height {
            center_y = (visible_height - app.map.height) / 2;
            start_y = 0;
        }

        if app.map.width < visible_width {
            center_x = (visible_width - app.map.width) / 2;
            start_x = 0;
        }

        let mut lines = Vec::new();

        for row in 0..visible_height {
            let mut spans = Vec::new();

            for col in 0..visible_width {
                let map_y = start_y + row.saturating_sub(center_y);
                let map_x = start_x + col.saturating_sub(center_x);

                if row >= center_y
                    && row < center_y + app.map.height.min(visible_height)
                    && col >= center_x
                    && col < center_x + app.map.width.min(visible_width)
                    && map_x < app.map.width
                    && map_y < app.map.height
                {
                    let (symbol, color) = Self::get_cell_display(app.map, app.robots, map_x, map_y);
                    spans.push(Span::styled(
                        format!("{} ", symbol),
                        Style::default().fg(color),
                    ));
                } else {
                    spans.push(Span::raw("  "));
                }
            }
            lines.push(Line::from(spans));
        }

        let scroll_info = if app.map.height > visible_height || app.map.width > visible_width {
            format!(" - Scroll: ↑↓←→ ({},{})", app.scroll_x, app.scroll_y)
        } else {
            String::new()
        };

        let title = format!("Nova Simulation{}", scroll_info);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_stats(f: &mut Frame, area: Rect, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let (energy_count, mineral_count, scientific_count) =
            App::calculate_resource_stats(app.map);
        let (explorers, harvesters, scientists) = app.get_robot_stats();

        let resource_text = vec![
            Line::from(vec![Span::styled(
                "Resources",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::raw("Energy: "),
                Span::styled(
                    format!("{}", energy_count),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::raw("Minerals: "),
                Span::styled(
                    format!("{}", mineral_count),
                    Style::default().fg(Color::Blue),
                ),
            ]),
            Line::from(vec![
                Span::raw("Scientific: "),
                Span::styled(
                    format!("{}", scientific_count),
                    Style::default().fg(Color::Magenta),
                ),
            ]),
        ];

        let robot_text = vec![
            Line::from(vec![Span::styled(
                "Robots",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled("Explorers", Style::default().fg(Color::Green)),
                Span::raw(": "),
                Span::styled(format!("{}", explorers), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Harvesters", Style::default().fg(Color::Blue)),
                Span::raw(": "),
                Span::styled(format!("{}", harvesters), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Scientists", Style::default().fg(Color::Magenta)),
                Span::raw(": "),
                Span::styled(format!("{}", scientists), Style::default().fg(Color::White)),
            ]),
        ];

        let resource_block = Block::default()
            .title("Resources")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let robot_block = Block::default()
            .title("Robots")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let resource_paragraph = Paragraph::new(resource_text)
            .block(resource_block)
            .wrap(Wrap { trim: true });

        let robot_paragraph = Paragraph::new(robot_text)
            .block(robot_block)
            .wrap(Wrap { trim: true });

        f.render_widget(resource_paragraph, chunks[0]);
        f.render_widget(robot_paragraph, chunks[1]);
    }

    fn get_cell_display(map: &Map, robots: &[Robot], x: usize, y: usize) -> (char, Color) {
        let station_pos = (map.width / 2, map.height / 2);

        if (x, y) == station_pos {
            return ('@', Color::Yellow);
        }

        for robot in robots {
            if robot.x == x && robot.y == y {
                return match robot.robot_type {
                    RobotType::Explorer => ('X', Color::Green),
                    RobotType::Harvester => ('H', Color::Blue),
                    RobotType::Scientist => ('R', Color::Magenta),
                };
            }
        }

        if let Some((resource_type, _)) = map.resources.get(&(x, y)) {
            return match resource_type {
                ResourceType::Energy => ('E', Color::Green),
                ResourceType::Mineral => ('M', Color::Blue),
                ResourceType::ScientificInterest => ('S', Color::Magenta),
            };
        }

        let terrain_type = TerrainType::from(map.terrain[y][x]);
        match terrain_type {
            TerrainType::Plain => ('.', Color::White),
            TerrainType::Hill => ('^', Color::Yellow),
            TerrainType::Mountain => ('▲', Color::Red),
            TerrainType::Canyon => ('#', Color::Red),
        }
    }
}

pub struct App<'a> {
    map: &'a Map,
    robots: &'a [Robot],
    scroll_x: usize,
    scroll_y: usize,
}

impl<'a> App<'a> {
    pub fn new(map: &'a Map, robots: &'a [Robot]) -> Self {
        Self {
            map,
            robots,
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    pub fn calculate_resource_stats(map: &Map) -> (u32, u32, u32) {
        let mut energy_count = 0;
        let mut mineral_count = 0;
        let mut scientific_count = 0;

        for (_, (resource_type, amount)) in &map.resources {
            match resource_type {
                ResourceType::Energy => energy_count += amount,
                ResourceType::Mineral => mineral_count += amount,
                ResourceType::ScientificInterest => scientific_count += amount,
            }
        }

        (energy_count, mineral_count, scientific_count)
    }

    fn get_robot_stats(&self) -> (usize, usize, usize) {
        let mut explorers = 0;
        let mut harvesters = 0;
        let mut scientists = 0;

        for robot in self.robots {
            match robot.robot_type {
                RobotType::Explorer => explorers += 1,
                RobotType::Harvester => harvesters += 1,
                RobotType::Scientist => scientists += 1,
            }
        }

        (explorers, harvesters, scientists)
    }
}
