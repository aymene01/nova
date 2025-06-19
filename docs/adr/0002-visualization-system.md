# Architecture Decision Record: Visualization System

## Submitters

- Nova Development Team

## Change Log

- [approved](#) 2025-05-28 - Initial dual-mode architecture
- [updated](#) 2025-01-15 - Real-time TUI with persistent visualization

## Referenced Use Case(s)

- [ADR-0001: Map Generation System](./0001-map-generation-system.md) - Addresses visualization concerns mentioned in the map generation ADR
- [ADR-0006: Real-Time Simulation System](./0006-real-time-simulation-system.md) - Real-time visualization requirements
- Interactive map visualization and user experience requirements

## Context

The visualization system is crucial for understanding and debugging the Nova simulation. The initial implementation used simple terminal output with colored text, but this approach had significant limitations:

1. **Scalability Issues**: Large maps were difficult to view and navigate
2. **Limited Interactivity**: No way to scroll or explore different parts of the map
3. **Poor User Experience**: Static output provided no real-time feedback
4. **Accessibility**: No adaptation to different terminal sizes or environments

We needed a visualization system that:

1. Scales gracefully from small to large maps
2. Provides interactive navigation capabilities
3. Works in both interactive and automated environments
4. Maintains optimal performance regardless of map size
5. Follows Clean Code principles with minimal complexity
6. **NEW**: Supports real-time robot movement visualization
7. **NEW**: Provides persistent TUI with live updates
8. **NEW**: Offers graceful user controls and exit mechanisms

## Proposed Design

### Dual-Mode Architecture

The visualization system implements a dual-mode architecture that automatically detects the execution environment:

```rust
pub fn visualize(map: &Map) -> Result<(), Box<dyn std::error::Error>> {
    if io::stdout().is_terminal() && io::stdin().is_terminal() {
        Self::visualize_tui(map)
    } else {
        Self::visualize_fallback(map);
        Ok(())
    }
}
```

### Interactive TUI Mode (Ratatui)

When running in a proper terminal environment, the system uses Ratatui for rich interactive visualization:

**Key Features:**

- **Viewport System**: Only renders visible portions of the map (O(viewport_size) complexity)
- **Scrollable Navigation**: Arrow keys for exploring large maps
- **Dynamic Layout**: Adapts panel sizes based on map dimensions
- **Color-Coded Display**: Terrain and resources use distinct colors for clarity
- **Real-time Statistics**: Live resource counts and density calculations
- **Smart Centering**: Small maps are automatically centered in the viewport
- **NEW**: **Real-time Robot Movement**: Live visualization of robot positions and actions
- **NEW**: **Persistent Updates**: Continuous refresh at 500ms intervals
- **NEW**: **Interactive Controls**: 'q' key for graceful exit
- **NEW**: **Energy Display**: Real-time energy levels for each robot
- **NEW**: **Resource Tracking**: Live updates of collected resources

**Technical Implementation:**

- Uses crossterm for terminal control and event handling
- Implements proper terminal state management (raw mode, alternate screen)
- Graceful error handling and cleanup on exit
- Memory-efficient rendering with minimal allocations
- **NEW**: Non-blocking event polling for responsive controls
- **NEW**: Terminal state restoration on exit
- **NEW**: Real-time data synchronization with simulation engine

### Fallback Mode

When stdin/stdout is redirected (pipes, scripts, CI/CD), the system automatically falls back to simple text output:

- Maintains all essential information (map layout, resources, statistics)
- Preserves scriptability and automation compatibility
- No dependencies on terminal capabilities
- Identical data representation in different format
- **NEW**: Periodic updates for long-running simulations
- **NEW**: Progress indicators for automated runs

### Adaptive Rendering

The system adapts to different map sizes intelligently:

**Small Maps (≤ viewport):**

- Automatic centering both horizontally and vertically
- No scroll indicators needed
- Compact statistics panel

**Large Maps (> viewport):**

- Viewport-based rendering with scroll indicators
- Current position display: `Scroll: ↑↓←→ (x,y)`
- Enhanced statistics with density metrics
- Efficient bounds checking

### Real-Time Visualization Features

#### Robot Movement Display
```rust
// Real-time robot position updates
for robot in robots {
    let symbol = match robot.robot_type() {
        RobotType::Explorer => "🗺️",
        RobotType::Harvester => "⛏️",
        RobotType::Scientist => "🔬",
    };
    
    // Display with energy level indicator
    let energy_bar = format!("[{}]", "█".repeat(robot.energy() as usize / 10));
    // Render at robot position
}
```

#### Live Statistics Panel
```rust
// Real-time resource tracking
let stats = format!(
    "Energy: {} | Minerals: {} | Discoveries: {} | Active Robots: {}",
    station.get_resource_amount(&ResourceType::Energy),
    station.get_resource_amount(&ResourceType::Mineral),
    station.discoveries,
    robots.iter().filter(|r| !r.is_low_energy()).count()
);
```

#### Interactive Controls
```rust
// Non-blocking event handling
if event::poll(Duration::from_millis(100)).unwrap_or(false) {
    if let Ok(Event::Key(key)) = event::read() {
        match key.code {
            KeyCode::Char('q') => return Ok(()), // Graceful exit
            KeyCode::Char('h') => { /* Help */ },
            KeyCode::Char('s') => { /* Save state */ },
            _ => {}
        }
    }
}
```

## Considerations

### Alternative Approaches Considered

1. **Web-based Visualization**: Considered HTML/JavaScript frontend but rejected due to complexity and deployment requirements

2. **GUI Framework**: Evaluated native GUI frameworks (egui, iced) but rejected due to:

   - Additional dependencies and complexity
   - Platform-specific issues
   - Reduced portability

3. **Always-TUI Approach**: Considered forcing TUI mode always but rejected due to:

   - Breaks automation and scripting
   - CI/CD compatibility issues
   - Accessibility concerns

4. **Separate Visualization Binary**: Considered splitting visualization into separate tool but rejected due to:
   - Increased project complexity
   - User experience fragmentation
   - Maintenance overhead

5. **NEW**: **WebSocket-based Real-time Updates**: Considered but rejected due to:
   - Additional network complexity
   - Performance overhead for local simulation
   - Increased deployment complexity

6. **NEW**: **Shared Memory Visualization**: Considered but rejected due to:
   - Platform-specific implementation
   - Security concerns
   - Debugging complexity

### Technical Decisions

1. **Ratatui over other TUI libraries**: Chosen for its:

   - Active maintenance and community
   - Excellent crossterm integration
   - Clean, composable widget system
   - Performance characteristics

2. **Automatic Mode Detection**: Using `std::io::IsTerminal` provides:

   - Reliable environment detection
   - No additional dependencies
   - Standard library stability

3. **Viewport-based Rendering**: Ensures:
   - Constant memory usage regardless of map size
   - Responsive performance on large maps
   - Scalable architecture

4. **NEW**: **Non-blocking Event Polling**: Chosen for:
   - Responsive user controls
   - No impact on simulation performance
   - Graceful handling of rapid key presses

5. **NEW**: **Terminal State Management**: Implemented for:
   - Proper cleanup on exit
   - Cross-platform compatibility
   - User experience consistency

### Concerns and Mitigations

1. **Terminal Compatibility**:

   - **Concern**: Different terminals may have varying capabilities
   - **Mitigation**: Crossterm provides excellent cross-platform compatibility

2. **Performance on Very Large Maps**:

   - **Concern**: Rendering performance with massive maps
   - **Mitigation**: Viewport culling ensures O(viewport_size) complexity

3. **User Experience Consistency**:
   - **Concern**: Different experiences between TUI and fallback modes
   - **Mitigation**: Both modes provide identical information, just different presentation

4. **NEW**: **Real-time Performance**:
   - **Concern**: Impact of continuous updates on simulation performance
   - **Mitigation**: 500ms update intervals balance responsiveness and performance

5. **NEW**: **Memory Usage**:
   - **Concern**: Potential memory leaks during long sessions
   - **Mitigation**: Proper cleanup and minimal allocations in render loop

## Decision

The implemented dual-mode visualization system provides optimal user experience across different environments:

### Key Architectural Decisions:

1. **Environment-Aware Design**: Automatic detection and mode switching based on terminal availability

2. **Performance-First Approach**: Viewport-based rendering ensures scalability to any map size

3. **Clean Code Compliance**:

   - Single responsibility: each mode handles its specific use case
   - Minimal complexity: O(viewport_size) time and space complexity
   - No premature optimization: simple, clear implementations

4. **Graceful Degradation**: Full functionality maintained in both interactive and automated environments

5. **User-Centric Design**:
   - Interactive navigation for exploration
   - Automatic adaptation to different map sizes
   - Clear visual feedback and information display

6. **NEW**: **Real-time Capabilities**:
   - Live robot movement visualization
   - Persistent TUI with continuous updates
   - Responsive user controls
   - Graceful exit mechanisms

### Benefits Achieved:

- **Scalability**: Handles maps from 5x5 to 50x50+ with equal performance
- **Usability**: Interactive exploration with intuitive controls
- **Automation-Friendly**: Maintains scriptability and CI/CD compatibility
- **Maintainability**: Clean separation of concerns between modes
- **Performance**: Constant memory usage and responsive rendering
- **NEW**: **Immersive Experience**: Real-time visualization of robot behaviors
- **NEW**: **Responsive Controls**: Immediate user feedback and control
- **NEW**: **Robust Operation**: Graceful handling of errors and exit conditions

### Future Enhancement Opportunities:

- Mouse support for navigation
- Zoom levels for very large maps
- Export capabilities (PNG, SVG)
- Real-time simulation visualization
- Multi-map comparison views
- **NEW**: **Advanced Controls**: Pause/resume, speed control, robot selection
- **NEW**: **Enhanced Display**: Robot trails, heat maps, statistics graphs
- **NEW**: **Recording Features**: Save/load simulation states, replay functionality

## Other Related ADRs

- [ADR-0001: Map Generation System](./0001-map-generation-system.md) - Addresses visualization limitations mentioned in the original map generation ADR
- [ADR-0006: Real-Time Simulation System](./0006-real-time-simulation-system.md) - Real-time visualization requirements and implementation

## References

- [Ratatui Documentation](https://docs.rs/ratatui/) - The TUI framework used for interactive visualization
- [Crossterm Documentation](https://docs.rs/crossterm/) - Cross-platform terminal manipulation library
- [std::io::IsTerminal](https://doc.rust-lang.org/std/io/trait.IsTerminal.html) - Standard library terminal detection
- [Tokio Documentation](https://docs.rs/tokio/) - Async runtime for real-time features
