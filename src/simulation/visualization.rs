use crate::simulation::entities::{Map, ResourceType};
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
    pub fn visualize(map: &Map) -> Result<(), Box<dyn std::error::Error>> {
        if io::stdout().is_terminal() && io::stdin().is_terminal() {
            Self::visualize_tui(map)
        } else {
            Self::visualize_fallback(map);
            Ok(())
        }
    }

    fn visualize_tui(map: &Map) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut app = App::new(map);
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

    fn visualize_fallback(map: &Map) {
        println!("Map {}x{} (seed: {})", map.width, map.height, map.seed);
        println!("Legend: . Plain  ^ Hill  ▲ Mountain  # Canyon");
        println!("Resources: E Energy  M Mineral  S Scientific Interest");
        println!();

        for y in 0..map.height {
            for x in 0..map.width {
                let (symbol, _) = Self::get_cell_display(map, x, y);
                print!("{} ", symbol);
            }
            println!();
        }

        let (energy_count, mineral_count, scientific_count) = App::calculate_resource_stats(map);
        println!("\nResource Statistics:");
        println!("Energy: {} units", energy_count);
        println!("Minerals: {} units", mineral_count);
        println!("Scientific Interest: {} units", scientific_count);
    }

    fn run_app<B: ratatui::backend::Backend>(
        terminal: &mut Terminal<B>,
        app: &mut App,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|f| Self::ui(f, app))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => app.scroll_up(),
                    KeyCode::Down => app.scroll_down(),
                    KeyCode::Left => app.scroll_left(),
                    KeyCode::Right => app.scroll_right(),
                    _ => {}
                }
            }
        }
    }

    fn ui(f: &mut Frame, app: &App) {
        let stats_height = if app.map.width * app.map.height > 400 {
            9
        } else {
            8
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
                    let (symbol, color) = Self::get_cell_display(app.map, map_x, map_y);
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
            "Map {}x{} (seed: {}){} - 'q' to quit",
            app.map.width, app.map.height, app.map.seed, scroll_info
        );

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_stats(f: &mut Frame, area: Rect, app: &App) {
        let (energy_count, mineral_count, scientific_count) = app.get_resource_stats();

        let map_density = ((app.map.width * app.map.height) as f32 / 100.0).max(1.0);
        let resource_density =
            (energy_count + mineral_count + scientific_count) as f32 / map_density;

        let legend_text = vec![
            Line::from(vec![
                Span::styled("Legend: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(".", Style::default().fg(Color::Green)),
                Span::raw(" Plain  "),
                Span::styled("^", Style::default().fg(Color::Yellow)),
                Span::raw(" Hill  "),
                Span::styled("▲", Style::default().fg(Color::Red)),
                Span::raw(" Mountain  "),
                Span::styled("#", Style::default().fg(Color::Magenta)),
                Span::raw(" Canyon"),
            ]),
            Line::from(vec![
                Span::raw("Resources: "),
                Span::styled("E", Style::default().fg(Color::LightGreen)),
                Span::raw(" Energy  "),
                Span::styled("M", Style::default().fg(Color::LightBlue)),
                Span::raw(" Mineral  "),
                Span::styled("S", Style::default().fg(Color::LightCyan)),
                Span::raw(" Scientific"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Statistics:", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(
                    " Size: {}x{} ({} cells)",
                    app.map.width,
                    app.map.height,
                    app.map.width * app.map.height
                )),
            ]),
            Line::from(format!("Energy: {} units", energy_count)),
            Line::from(format!("Minerals: {} units", mineral_count)),
            Line::from(format!("Scientific: {} units", scientific_count)),
            Line::from(format!(
                "Density: {:.1} resources/100 cells",
                resource_density
            )),
        ];

        let paragraph = Paragraph::new(legend_text)
            .block(Block::default().borders(Borders::ALL).title("Info"))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn get_cell_display(map: &Map, x: usize, y: usize) -> (char, Color) {
        if let Some((resource_type, _)) = map.resources.get(&(x, y)) {
            match resource_type {
                ResourceType::Energy => ('E', Color::LightGreen),
                ResourceType::Mineral => ('M', Color::LightBlue),
                ResourceType::ScientificInterest => ('S', Color::LightCyan),
            }
        } else {
            let terrain_type = TerrainType::from(map.terrain[y][x]);
            match terrain_type {
                TerrainType::Plain => ('.', Color::Green),
                TerrainType::Hill => ('^', Color::Yellow),
                TerrainType::Mountain => ('▲', Color::Red),
                TerrainType::Canyon => ('#', Color::Magenta),
            }
        }
    }
}

struct App<'a> {
    map: &'a Map,
    offset_x: usize,
    offset_y: usize,
}

impl<'a> App<'a> {
    fn new(map: &'a Map) -> Self {
        Self {
            map,
            offset_x: 0,
            offset_y: 0,
        }
    }

    fn scroll_up(&mut self) {
        self.offset_y = self.offset_y.saturating_sub(1);
    }

    fn scroll_down(&mut self) {
        if self.map.height > 1 {
            self.offset_y = (self.offset_y + 1).min(self.map.height - 1);
        }
    }

    fn scroll_left(&mut self) {
        self.offset_x = self.offset_x.saturating_sub(1);
    }

    fn scroll_right(&mut self) {
        if self.map.width > 1 {
            self.offset_x = (self.offset_x + 1).min(self.map.width - 1);
        }
    }

    fn get_resource_stats(&self) -> (u32, u32, u32) {
        Self::calculate_resource_stats(self.map)
    }

    fn calculate_resource_stats(map: &Map) -> (u32, u32, u32) {
        let mut energy_count = 0;
        let mut mineral_count = 0;
        let mut scientific_count = 0;

        for ((_, _), (res_type, amount)) in &map.resources {
            match res_type {
                ResourceType::Energy => energy_count += amount,
                ResourceType::Mineral => mineral_count += amount,
                ResourceType::ScientificInterest => scientific_count += amount,
            }
        }

        (energy_count, mineral_count, scientific_count)
    }
}
