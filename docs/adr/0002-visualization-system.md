# Architecture Decision Record: Visualization System

## Submitters

- Nova Development Team

## Change Log

- [approved](#) 2025-05-28

## Referenced Use Case(s)

- [ADR-0001: Map Generation System](./0001-map-generation-system.md) - Addresses visualization concerns mentioned in the map generation ADR
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

**Technical Implementation:**

- Uses crossterm for terminal control and event handling
- Implements proper terminal state management (raw mode, alternate screen)
- Graceful error handling and cleanup on exit
- Memory-efficient rendering with minimal allocations

### Fallback Mode

When stdin/stdout is redirected (pipes, scripts, CI/CD), the system automatically falls back to simple text output:

- Maintains all essential information (map layout, resources, statistics)
- Preserves scriptability and automation compatibility
- No dependencies on terminal capabilities
- Identical data representation in different format

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

### Performance Optimizations

1. **Viewport Culling**: Only processes visible cells during rendering
2. **Minimal Allocations**: Reuses data structures where possible
3. **Efficient Layout**: 2-character cell width for optimal readability
4. **Smart Updates**: Only redraws when necessary

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

### Benefits Achieved:

- **Scalability**: Handles maps from 5x5 to 50x50+ with equal performance
- **Usability**: Interactive exploration with intuitive controls
- **Automation-Friendly**: Maintains scriptability and CI/CD compatibility
- **Maintainability**: Clean separation of concerns between modes
- **Performance**: Constant memory usage and responsive rendering

### Future Enhancement Opportunities:

- Mouse support for navigation
- Zoom levels for very large maps
- Export capabilities (PNG, SVG)
- Real-time simulation visualization
- Multi-map comparison views

## Other Related ADRs

- [ADR-0001: Map Generation System](./0001-map-generation-system.md) - Addresses visualization limitations mentioned in the original map generation ADR

## References

- [Ratatui Documentation](https://docs.rs/ratatui/) - The TUI framework used for interactive visualization
- [Crossterm Documentation](https://docs.rs/crossterm/) - Cross-platform terminal manipulation library
- [std::io::IsTerminal](https://doc.rust-lang.org/std/io/trait.IsTerminal.html) - Standard library terminal detection
