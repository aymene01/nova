use crate::simulation::entities::{Map, ResourceType, Robot, RobotType};
use crate::simulation::map::TerrainType;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::io::{self, IsTerminal};

/// Utility for visualizing the map in the terminal
pub struct MapVisualizer;

impl MapVisualizer {
    /// Visualizes the map in the terminal
    #[allow(dead_code)]
    pub fn visualize(map: &Map) -> Result<(), Box<dyn std::error::Error>> {
        Self::visualize_with_robots(map, &[])
    }

    /// Visualizes the map with robots in the terminal
    pub fn visualize_with_robots(
        map: &Map,
        robots: &[Robot],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if io::stdout().is_terminal() && io::stdin().is_terminal() {
            Self::visualize_tui(map, robots)
        } else {
            Self::visualize_fallback(map, robots);
            Ok(())
        }
    }

    fn visualize_tui(map: &Map, robots: &[Robot]) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut app = App::new(map, robots);
        let res = Self::run_app(&mut terminal, &mut app);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{err:?}");
        }

        Ok(())
    }

    fn visualize_fallback(map: &Map, robots: &[Robot]) {
        println!(
            "Map {}x{} (seed: {}) - {} robots",
            map.width,
            map.height,
            map.seed,
            robots.len()
        );
        println!("Legend: . Plain  ^ Hill  ▲ Mountain  # Canyon");
        println!("Resources: E Energy  M Mineral  S Scientific Interest");
        println!("Robots: X Explorer  H Harvester  R Scientist  @ Station");
        println!();

        for y in 0..map.height {
            for x in 0..map.width {
                let (symbol, _) = Self::get_cell_display(map, robots, x, y);
                print!("{} ", symbol);
            }
            println!();
        }

        let (energy_count, mineral_count, scientific_count) = App::calculate_resource_stats(map);
        println!("\nResource Statistics:");
        println!("Energy: {} units", energy_count);
        println!("Minerals: {} units", mineral_count);
        println!("Scientific Interest: {} units", scientific_count);

        println!("\nRobot Status:");
        for robot in robots {
            println!(
                "  Robot {}: {:?} at ({},{}) with {} energy",
                robot.id, robot.robot_type, robot.x, robot.y, robot.energy
            );
        }
    }

    fn run_app<B: ratatui::backend::Backend>(
        terminal: &mut Terminal<B>,
        app: &mut App,
    ) -> io::Result<()> {
        // Non-blocking version - just draw once and return
        terminal.draw(|f| Self::ui(f, app))?;

        // Check for quit key non-blocking
        if event::poll(std::time::Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "User quit"));
                }
            }
        }

        Ok(())
    }

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
            .offset_y
            .min(app.map.height.saturating_sub(visible_height));
        let mut start_x = app
            .offset_x
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
            format!(" - Scroll: ↑↓←→ ({},{})", app.offset_x, app.offset_y)
        } else {
            String::new()
        };

        let title = format!(
            "Map {}x{} (seed: {}) - {} robots{} - 'q' to quit",
            app.map.width,
            app.map.height,
            app.map.seed,
            app.robots.len(),
            scroll_info
        );

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_stats(f: &mut Frame, area: Rect, app: &App) {
        let (energy_count, mineral_count, scientific_count) = app.get_resource_stats();

        let map_density = ((app.map.width * app.map.height) as f32 / 100.0).max(1.0);
        let _resource_density =
            (energy_count + mineral_count + scientific_count) as f32 / map_density;

        // Calculate robot statistics
        let (explorers, harvesters, scientists) = app.get_robot_stats();
        let total_energy: u32 = app.robots.iter().map(|r| r.energy).sum();
        let avg_energy = if !app.robots.is_empty() {
            total_energy as f32 / app.robots.len() as f32
        } else {
            0.0
        };

        let legend_text = vec![
            Line::from(vec![
                Span::styled("Legend: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(".", Style::default().fg(Color::Green)),
                Span::raw(" Plain  "),
                Span::styled("^", Style::default().fg(Color::Yellow)),
                Span::raw(" Hill  "),
                Span::styled("▲", Style::default().fg(Color::Red)),
                Span::raw(" Mountain  "),
                Span::styled("#", Style::default().fg(Color::DarkGray)),
                Span::raw(" Canyon"),
            ]),
            Line::from(vec![
                Span::styled("Resources: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled("E", Style::default().fg(Color::Cyan)),
                Span::raw(" Energy  "),
                Span::styled("M", Style::default().fg(Color::Magenta)),
                Span::raw(" Mineral  "),
                Span::styled("S", Style::default().fg(Color::Blue)),
                Span::raw(" Scientific"),
            ]),
            Line::from(vec![
                Span::styled("Robots: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled("X", Style::default().fg(Color::White)),
                Span::raw(" Explorer  "),
                Span::styled("H", Style::default().fg(Color::Yellow)),
                Span::raw(" Harvester  "),
                Span::styled("R", Style::default().fg(Color::Cyan)),
                Span::raw(" Scientist  "),
                Span::styled("@", Style::default().fg(Color::Green)),
                Span::raw(" Station"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Statistics: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    "Size: {}x{} ({} cells)",
                    app.map.width,
                    app.map.height,
                    app.map.width * app.map.height
                )),
            ]),
            Line::from(vec![
                Span::styled("Energy: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} units", energy_count)),
            ]),
            Line::from(vec![
                Span::styled("Minerals: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} units", mineral_count)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Scientific: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{} units", scientific_count)),
            ]),
            Line::from(vec![
                Span::styled("Robots: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(
                    "{} total ({}X/{}H/{}R)",
                    app.robots.len(),
                    explorers,
                    harvesters,
                    scientists
                )),
            ]),
            Line::from(vec![
                Span::styled("Energy: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("Total: {} | Avg: {:.1}", total_energy, avg_energy)),
            ]),
        ];

        let paragraph = Paragraph::new(legend_text)
            .block(Block::default().borders(Borders::ALL).title("Statistics"))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn get_cell_display(map: &Map, robots: &[Robot], x: usize, y: usize) -> (char, Color) {
        // Check if there's a robot at this position
        for robot in robots {
            if robot.x == x && robot.y == y {
                return match robot.robot_type {
                    RobotType::Explorer => ('X', Color::White),
                    RobotType::Harvester => ('H', Color::Yellow),
                    RobotType::Scientist => ('R', Color::Cyan),
                };
            }
        }

        // Check if this is the station position (center of map)
        let station_x = map.width / 2;
        let station_y = map.height / 2;
        if x == station_x && y == station_y {
            return ('@', Color::Green);
        }

        // Check for resources first
        if let Some((resource_type, _)) = map.resources.get(&(x, y)) {
            return match resource_type {
                ResourceType::Energy => ('E', Color::Cyan),
                ResourceType::Mineral => ('M', Color::Magenta),
                ResourceType::ScientificInterest => ('S', Color::Blue),
            };
        }

        // Then check terrain
        let terrain_value = map.terrain[y][x];
        match TerrainType::from(terrain_value) {
            TerrainType::Plain => ('.', Color::Green),
            TerrainType::Hill => ('^', Color::Yellow),
            TerrainType::Mountain => ('▲', Color::Red),
            TerrainType::Canyon => ('#', Color::DarkGray),
        }
    }
}

pub struct App<'a> {
    map: &'a Map,
    robots: &'a [Robot],
    offset_x: usize,
    offset_y: usize,
}

impl<'a> App<'a> {
    pub fn new(map: &'a Map, robots: &'a [Robot]) -> Self {
        Self {
            map,
            robots,
            offset_x: 0,
            offset_y: 0,
        }
    }

    #[allow(dead_code)]
    fn scroll_up(&mut self) {
        self.offset_y = self.offset_y.saturating_sub(1);
    }

    #[allow(dead_code)]
    fn scroll_down(&mut self) {
        if self.offset_y + 1 < self.map.height {
            self.offset_y += 1;
        }
    }

    #[allow(dead_code)]
    fn scroll_left(&mut self) {
        self.offset_x = self.offset_x.saturating_sub(1);
    }

    #[allow(dead_code)]
    fn scroll_right(&mut self) {
        if self.offset_x + 1 < self.map.width {
            self.offset_x += 1;
        }
    }

    fn get_resource_stats(&self) -> (u32, u32, u32) {
        Self::calculate_resource_stats(self.map)
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

    fn calculate_resource_stats(map: &Map) -> (u32, u32, u32) {
        let mut energy_count = 0;
        let mut mineral_count = 0;
        let mut scientific_count = 0;

        for (resource_type, amount) in map.resources.values() {
            match resource_type {
                ResourceType::Energy => energy_count += amount,
                ResourceType::Mineral => mineral_count += amount,
                ResourceType::ScientificInterest => scientific_count += amount,
            }
        }

        (energy_count, mineral_count, scientific_count)
    }
}
