# Nova - Robot Swarm Simulation

Nova is a Rust-based robot swarm simulation system featuring live visualization, procedural map generation, and autonomous robot movement. The system demonstrates real-time robot behavior through an interactive terminal interface with advanced information merging capabilities.

## 🎮 Live Simulation Features

### Real-Time TUI Visualization

- **Interactive Terminal Interface**: Built with ratatui for smooth, responsive visualization
- **Live Robot Movement**: Watch 25 robots move across a 100x100 procedurally generated map
- **Terrain Visualization**: See different terrain types (Plains, Hills, Mountains, Canyons)
- **Resource Display**: Energy (E), Minerals (M), and Scientific Interest (S) points
- **Station Monitoring**: Central station (@) for robot recharging and coordination

### Controls

- **'q'**: Quit simulation gracefully
- **Arrow Keys**: Scroll around large maps (when applicable)

## 🤖 Robot System

### Three Robot Types

- **Explorer (X)**: Autonomous exploration robots
- **Harvester (H)**: Resource collection specialists
- **Scientist (S)**: Research and analysis units

### Robot Behavior

- **Random Movement**: Robots explore the map using intelligent random patterns
- **Energy Management**: Automatic recharging when energy drops below 50%
- **Boundary Awareness**: Robots avoid map edges and invalid positions
- **Continuous Operation**: Forgiving energy system ensures constant movement

### Energy System

- **Movement Cost**: 3 energy units per move (reduced for better gameplay)
- **Automatic Recharging**: Robots gain 20 energy when below 50%
- **Station Support**: Station provides 10,000 energy units for robot operations
- **Sustainable Operation**: Energy system designed for continuous exploration

## 🗺️ Procedural World Generation

### Perlin Noise-Based Terrain

- **Deterministic Generation**: Same seed produces identical maps
- **Four Terrain Types**:
  - **Plains (.)**: Easy traversal, common areas
  - **Hills (^)**: Moderate difficulty terrain
  - **Mountains (▲)**: Challenging high-altitude areas
  - **Canyons (#)**: Deep terrain features

### Resource Distribution

- **Energy Sources (E)**: Power for robot operations
- **Mineral Deposits (M)**: Valuable extraction targets
- **Scientific Interest (S)**: Research and analysis points
- **Realistic Density**: Balanced resource placement across terrain

### Map Features

- **Configurable Size**: Default 100x100, customizable dimensions
- **Seed-Based**: Reproducible worlds for testing and development
- **Serialization**: Save/load maps in JSON format
- **Boundary Management**: Proper edge handling for robot movement

## 🏭 Station & Information System

### Central Station

- **Robot Coordination**: Central hub for robot operations
- **Energy Distribution**: Manages power supply for robot fleet
- **Information Processing**: Handles robot discoveries and data

### Git-Like Information Merging

- **Conflict Detection**: Identifies contradictory robot reports
- **Automatic Resolution**: Intelligent merging of compatible information
- **Conflict Types**:
  - Resource amount differences (>20% variance)
  - Resource type conflicts (Energy vs Mineral)
  - Terrain mismatches (different terrain reports)
  - Confidence conflicts (reliability differences)

### Knowledge Management

- **Location Tracking**: Maintains database of discovered positions
- **Merge Statistics**: Tracks successful merges and conflicts
- **Conflict Resolution**: Manual review system for serious conflicts
- **Resource Estimates**: Provides exploration recommendations

## 🛠️ Technical Implementation

### Concurrent Architecture

- **Tokio Runtime**: Async/await for smooth simulation performance
- **Non-Blocking TUI**: Real-time updates without freezing
- **Parallel Processing**: Efficient robot state management
- **Performance Optimized**: 500ms update cycles with 100ms render loops

### Pathfinding System

- **A\* Algorithm**: Optimal pathfinding with Manhattan distance heuristic
- **Overflow Protection**: Saturating arithmetic prevents panics
- **Terrain Awareness**: Movement costs based on terrain difficulty
- **Boundary Checking**: Safe navigation within map limits

### Data Structures

- **HashMap Resources**: Efficient sparse resource storage
- **Vector Terrain**: Fast terrain lookup and modification
- **Station Knowledge**: Comprehensive discovery tracking
- **Conflict Management**: Structured conflict resolution system

## 🧪 Quality Assurance

### Comprehensive Testing

- **87 Passing Tests**: Full coverage of core functionality
- **Unit Tests**: Individual component verification
- **Integration Tests**: System interaction validation
- **Property-Based Tests**: Map generation consistency
- **Performance Tests**: Concurrent simulation load testing

### Code Quality

- **Clean Linting**: Clippy and rustfmt compliance
- **Memory Safety**: Rust's ownership system prevents common bugs
- **Error Handling**: Comprehensive Result types and error propagation
- **Documentation**: Inline docs and architectural decision records

## 🚀 Getting Started

### Quick Start

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and run Nova
git clone <repository-url>
cd nova
cargo run -- start

# Follow the interactive setup:
# - Seed: 42 (or any number for different worlds)
# - Map Size: 100x100 (default)
# - Robot Count: 25 (default)

# Watch robots explore in real-time!
# Press 'q' to quit when done
```

### Development Commands

```bash
# Run all tests
make all

# Format code
cargo fmt

# Run linter
cargo clippy

# Build release version
cargo build --release

# Run specific test module
cargo test simulation::entities
```

### Configuration Options

```bash
# Custom map size
cargo run -- start
> Map Width [100]: 50
> Map Height [100]: 50

# Different robot count
> Robots Count [25]: 10

# Custom seed for reproducible worlds
> Seed [42]: 12345
```

## 📁 Project Structure

```
nova/
├── src/
│   ├── main.rs                 # Entry point with TUI simulation loop
│   ├── cli/                    # Command-line interface
│   ├── config/                 # Configuration management
│   └── simulation/
│       ├── entities.rs         # Robot, Station, Map core types
│       ├── map.rs             # Perlin noise map generation
│       ├── visualization.rs    # TUI rendering with ratatui
│       ├── pathfinding.rs     # A* algorithm implementation
│       ├── ai/                # Robot behavior systems
│       └── engine.rs          # Concurrent simulation engine
├── docs/
│   └── adr/                   # Architecture Decision Records
├── Makefile                   # Development commands
└── Cargo.toml                 # Dependencies and metadata
```

## 🎯 Key Features Demonstrated

### Real-Time Visualization

- **Smooth Animation**: Watch robots move across terrain in real-time
- **Resource Tracking**: See energy levels and resource distribution
- **Interactive Interface**: Responsive TUI with immediate feedback

### Advanced Data Management

- **Information Conflicts**: Git-like merging when robots report different data
- **Automatic Resolution**: Smart algorithms for compatible information
- **Manual Review**: Human oversight for critical conflicts

### Robust Architecture

- **Concurrent Processing**: Multiple robots operating simultaneously
- **Memory Efficient**: Optimized data structures for large simulations
- **Error Resilient**: Comprehensive error handling and recovery

## 🔬 Educational Value

Nova demonstrates several important computer science concepts:

- **Multi-Agent Systems**: Autonomous robots operating independently
- **Real-Time Visualization**: Responsive user interfaces with complex data
- **Procedural Generation**: Algorithmic world creation with Perlin noise
- **Conflict Resolution**: Distributed system data consistency challenges
- **Concurrent Programming**: Async/await patterns in Rust
- **Clean Architecture**: Modular design with clear separation of concerns

## 🎨 Visual Experience

When you run Nova, you'll see:

```
=== NOVA SIMULATION ===
Map: 100x100 | Robots: 25 | Station: (50,50)

▲▲▲...^^^...EEE...▲▲▲...^^^...
...^^^...MMM...^^^...SSS...^^^
^^^...▲▲▲...^^^...EEE...^^^...
...SSS...^^^...MMM...▲▲▲...^^^
^^^...EEE...^^^...@...^^^...EEE
...^^^...SSS...^^^...MMM...^^^
▲▲▲...^^^...EEE...^^^...SSS...
...MMM...^^^...▲▲▲...^^^...EEE

Legend: . Plain  ^ Hill  ▲ Mountain  # Canyon
Resources: E Energy  M Mineral  S Scientific
Robots: X Explorer  H Harvester  S Scientist  @ Station

Statistics: Size: 100x100 (10000 cells)
Energy: 85407 units | Minerals: 71135 units
Scientific: 29981 units | Robots: 25 active
Energy: Total: 1847 | Avg: 73.9
```

## 🚧 Future Enhancements

- **Smart Robot AI**: Replace random movement with intelligent exploration
- **Resource Collection**: Implement actual harvesting mechanics
- **Robot Communication**: Inter-robot coordination and task sharing
- **Advanced Visualization**: 3D rendering or web-based interface
- **Performance Metrics**: Detailed simulation analytics and reporting
- **Scenario System**: Predefined challenges and objectives

---

_Nova showcases modern Rust development practices with real-time visualization, concurrent programming, and robust system architecture. Perfect for learning about multi-agent systems, procedural generation, and interactive terminal applications._
