# Nova - Autonomous Robot Simulation System

Nova is a sophisticated robot simulation system written in Rust that models autonomous robots with specialized behaviors operating in a dynamic environment. The system features three distinct robot types, each with unique capabilities and decision-making processes, supported by an advanced A* pathfinding algorithm and real-time TUI visualization.

## 🚀 Latest Features (v0.4.0)

### Real-Time Robot Movement
- **Live Simulation**: Robots move autonomously in real-time with persistent TUI visualization
- **Interactive Controls**: Press 'q' to quit the simulation gracefully
- **Smooth Animation**: 500ms update intervals for fluid robot movement
- **Energy Management**: Real-time energy consumption and recharge at stations

### Advanced Visualization System
- **Dual-Mode Architecture**: Automatic detection of terminal vs. automated environments
- **Interactive TUI**: Rich terminal interface with Ratatui for exploration
- **Fallback Mode**: Simple text output for scripts and CI/CD compatibility
- **Viewport System**: Efficient rendering for maps of any size (O(viewport_size) complexity)

## 🤖 Robot Types and Behaviors

### Explorer Robot 🗺️
**Primary Mission**: Mapping and area discovery

**Characteristics:**
- **Energy Consumption**: 2 energy units per action (most efficient)
- **Energy Threshold**: Returns to station when energy drops below 20
- **Preferred Resources**: None (exploration-focused)
- **Specialization**: Systematic exploration and mapping

**Behavior Pattern:**
- **Low Energy/Carrying Resources**: Returns to station immediately
- **Primary Action**: Seeks unexplored areas within a 5-unit radius
- **Fallback Action**: Uses deterministic random exploration based on robot state
- **Exploration Strategy**: Systematic area discovery with 3-unit radius exploration tasks

**Decision Logic:**
```rust
Priority 9: Return to station (energy < 20 OR carrying resources)
Priority 7: Explore unexplored areas (radius 3)
Priority 5: Random exploration target (radius 2)
```

### Harvester Robot ⛏️
**Primary Mission**: Resource collection and extraction

**Characteristics:**
- **Energy Consumption**: 3 energy units per action
- **Energy Threshold**: Returns to station when energy drops below 15
- **Preferred Resources**: Energy and Mineral resources
- **Specialization**: Efficient resource gathering

**Behavior Pattern:**
- **Low Energy/Carrying Resources**: Returns to station immediately
- **Primary Action**: Searches for Energy or Mineral resources within 4-unit radius
- **Resource Prioritization**: Targets preferred resources (Energy/Mineral) over others
- **Fallback Action**: Explores for new resource locations when none are nearby

**Decision Logic:**
```rust
Priority 10: Return to station (energy < 15 OR carrying resources)
Priority 8:  Harvest preferred resources (Energy/Mineral)
Priority 6:  Explore for new resource locations
```

### Scientist Robot 🔬
**Primary Mission**: Scientific research and analysis

**Characteristics:**
- **Energy Consumption**: 4 energy units per action (most energy-intensive)
- **Energy Threshold**: Returns to station when energy drops below 25 (highest threshold)
- **Preferred Resources**: Scientific Interest points
- **Specialization**: Scientific analysis and research

**Behavior Pattern:**
- **Low Energy/Carrying Data**: Returns to station immediately
- **Primary Action**: Searches for Scientific Interest points within 6-unit radius
- **Analysis Strategy**: Determines analysis type based on terrain and resource density
- **Research Focus**: Systematic exploration for scientific discovery

**Decision Logic:**
```rust
Priority 9: Return to station (energy < 25 OR carrying data)
Priority 8: Analyze scientific interests (Chemical analysis)
Priority 6: Systematic scientific exploration
```

**Analysis Type Determination:**
- Considers terrain value and nearby resource density
- Enhanced logic for future expansion to Geological, Biological analysis types
- Currently optimized for Chemical analysis based on resource concentration

## 🗺️ Pathfinding System

### A* Algorithm Implementation
The pathfinding system uses the A* algorithm for optimal path calculation with the following features:

**Core Components:**
- **Heuristic Function**: Manhattan distance for efficient path estimation
- **Cost Calculation**: Uniform cost (1) for adjacent moves, √2 for diagonal moves
- **Obstacle Avoidance**: Dynamic terrain analysis and obstacle detection
- **Boundary Checking**: Ensures paths stay within map bounds

**Performance Optimizations:**
- **Early Termination**: Stops when target is reached
- **Efficient Data Structures**: Binary heap for open set management
- **Memory Optimization**: Reuses path nodes when possible
- **Bounded Search**: Prevents infinite loops with reasonable search limits

**Path Features:**
- **8-Directional Movement**: Supports cardinal and diagonal directions
- **Optimal Pathfinding**: Guarantees shortest path when one exists
- **Dynamic Updates**: Recalculates paths when obstacles change
- **Smooth Navigation**: Provides step-by-step movement directions

### Movement Directions
```rust
pub enum Direction {
    North, South, East, West,           // Cardinal directions
    NorthEast, NorthWest,               // Diagonal directions  
    SouthEast, SouthWest
}
```

## 🛠️ Shared Utilities

### SearchUtils Module
Common search patterns used by all robot types:

- **`radius_search`**: Configurable radius-based position searching
- **`find_nearest_resource`**: Locates closest resources of specified types
- **`find_nearest_unexplored`**: Identifies unexplored areas for mapping
- **`find_nearest_scientific_interest`**: Finds scientific points of interest

**Benefits:**
- Eliminates code duplication across robot behaviors
- Consistent search algorithms for all robot types
- Optimized performance with shared implementations
- Easy maintenance and feature updates

## 🎯 Robot Comparison

| Feature | Explorer | Harvester | Scientist |
|---------|----------|-----------|-----------|
| **Energy Cost** | 2/action | 3/action | 4/action |
| **Energy Threshold** | 20 | 15 | 25 |
| **Primary Focus** | Exploration | Resource Collection | Scientific Analysis |
| **Preferred Resources** | None | Energy, Mineral | Scientific Interest |
| **Search Radius** | 5 units | 4 units | 6 units |
| **Specialization** | Mapping | Harvesting | Research |
| **Backup Behavior** | Random Exploration | Resource Exploration | Systematic Exploration |

## 🏗️ System Architecture

### Decision-Making Framework
All robots follow a priority-based decision system:

1. **Energy Management**: Monitor energy levels and return to station when needed
2. **Resource Status**: Handle carrying capacity and resource delivery
3. **Specialized Tasks**: Execute type-specific primary missions
4. **Fallback Behavior**: Explore when primary objectives are unavailable

### Energy Management
- **Dynamic Thresholds**: Each robot type has optimized energy thresholds
- **Efficient Routing**: Pathfinding minimizes energy consumption
- **Strategic Planning**: Robots plan return trips based on remaining energy
- **Resource Optimization**: Balanced energy costs vs. capability benefits

### Information Merging System
- **Git-like Conflict Resolution**: Intelligent merging of robot discoveries
- **Confidence-based Decisions**: Weighted merging based on robot confidence levels
- **Automatic Conflict Detection**: Detects terrain, resource, and confidence conflicts
- **Manual Review System**: Complex conflicts flagged for human intervention
- **Merge Statistics**: Comprehensive tracking of fusion operations

## 🧪 Testing and Quality

### Comprehensive Test Suite
- **115 Unit Tests** covering all robot behaviors and pathfinding
- **Behavior Verification**: Tests for each robot type's decision-making
- **Pathfinding Tests**: A* algorithm correctness and performance
- **Integration Tests**: Robot-environment interaction scenarios
- **Edge Case Coverage**: Boundary conditions and error handling

### Code Quality Features
- **Clean Architecture**: Modular design with clear separation of concerns
- **Rust Best Practices**: Memory safety and zero-cost abstractions
- **Comprehensive Documentation**: Inline docs and behavior specifications
- **Performance Optimized**: Efficient algorithms and data structures

## 🚀 Getting Started

### Running the Simulation
```bash
# Run the main simulation with interactive configuration
cargo run

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test robot_ai::behaviors::explorer
```

### Interactive Configuration
The simulation starts with an interactive configuration menu:
- **Seed**: Determines map generation (default: 42)
- **Map Dimensions**: Width and height of the simulation world
- **Robot Count**: Number of robots to spawn (distributed across types)

### Real-Time Controls
- **'q' Key**: Gracefully exit the simulation
- **Automatic Updates**: Robots move every 500ms
- **Live Statistics**: Real-time resource collection and energy levels

### Project Structure
```
src/
├── simulation/
│   ├── entities/          # Robot, Map, Station definitions
│   ├── engine.rs          # Concurrent simulation engine
│   ├── map.rs             # Procedural map generation
│   ├── visualization.rs   # TUI and fallback visualization
│   └── robot_ai/          # AI system implementation
│       ├── behavior.rs    # Behavior trait and factory
│       ├── behaviors/     # Individual robot behaviors
│       │   ├── explorer.rs
│       │   ├── harvester.rs
│       │   └── scientist.rs
│       ├── pathfinding.rs # A* pathfinding algorithm
│       ├── types.rs       # Task and type definitions
│       └── utils.rs       # Shared utility functions
├── cli/                   # Command-line interface
├── config/                # Interactive configuration
└── main.rs               # Application entry point
```

## 🔬 Scientific Approach

The Nova system demonstrates advanced autonomous robotics concepts:

- **Multi-Agent Systems**: Coordinated robot behaviors without central control
- **Specialized AI**: Type-specific decision-making algorithms
- **Efficient Pathfinding**: Optimal navigation in dynamic environments
- **Resource Management**: Strategic energy and resource allocation
- **Emergent Behavior**: Complex system behavior from simple robot rules
- **Information Fusion**: Git-like conflict resolution for distributed knowledge

## 📈 Future Enhancements

- **Dynamic Task Assignment**: Inter-robot communication and task sharing
- **Advanced Analysis Types**: Geological, Biological analysis capabilities
- **Learning Algorithms**: Adaptive behavior based on environment feedback
- **Multi-Objective Optimization**: Balanced exploration, harvesting, and research
- **Enhanced Visualization**: Mouse support, zoom levels, export capabilities
- **Distributed Simulation**: Multi-node simulation for large-scale scenarios

## 📚 Architecture Decision Records

The project includes comprehensive ADRs documenting key architectural decisions:

- **[ADR-0001: Map Generation System](docs/adr/0001-map-generation-system.md)** - Procedural terrain and resource generation
- **[ADR-0002: Visualization System](docs/adr/0002-visualization-system.md)** - Dual-mode TUI and fallback visualization
- **[ADR-0003: Concurrency Model](docs/adr/0003-concurrency-model.md)** - Tokio-based concurrent simulation engine
- **[ADR-0004: Robot AI Algorithms](docs/adr/0004-robot-ai-algorithms.md)** - Behavior patterns and decision-making systems
- **[ADR-0005: Information Merging System](docs/adr/0005-information-merging-system.md)** - Git-like conflict resolution for robot discoveries

---

*Nova represents a sophisticated simulation of autonomous robot systems, showcasing modern Rust development practices, advanced algorithmic implementations, and real-time interactive visualization.*
