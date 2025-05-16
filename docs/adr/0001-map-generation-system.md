# Architecture Decision Record: Map Generation System

## Submitters

- Nova Development Team

## Change Log

- [approved](#) 2024-06-05

## Referenced Use Case(s)

- [Nova Project Requirements](../../development_plan.md) - Map generation with obstacles, resource placement, and reproducibility from seed

## Context

The map generation system is a fundamental component of the Nova simulation. It creates the environment in which robots operate, including terrain that affects movement and resources that robots need to collect. The design decisions for this system influence many other aspects of the simulation, including robot movement, resource collection, and exploration strategies.

We needed to create a deterministic map generation system that:

1. Produces varied and interesting terrain
2. Distributes resources in a natural but predictable way
3. Allows for serialization and deserialization of maps
4. Supports robot movement with varying costs based on terrain
5. Enables reproducible map generation from a seed

## Proposed Design

### Map Structure

The map is implemented as a 2D grid with the following components:

- **Terrain**: A 2D array of terrain types (Plain, Hill, Mountain, Canyon) with increasing movement costs
- **Resources**: A HashMap mapping coordinates to resource types and amounts
- **Discovery State**: A 2D array tracking which cells have been discovered by robots

```rust
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub terrain: Vec<Vec<u8>>,
    pub resources: HashMap<(usize, usize), (ResourceType, u32)>,
    pub discovered: Vec<Vec<bool>>,
    pub noise: Perlin,
    pub seed: u64,
}
```

### Terrain Generation

Terrain is generated using Perlin noise, which creates natural-looking terrain patterns:

1. The map uses a seed to initialize the Perlin noise generator
2. Each grid cell's terrain is determined by the noise value at that position
3. Noise values are mapped to terrain types with different thresholds:
   - Plain (0-0.4): Low movement cost (1)
   - Hill (0.4-0.7): Medium movement cost (2)
   - Mountain (0.7-0.85): High movement cost (3)
   - Canyon (0.85-1.0): Very high movement cost (4)

### Resource Distribution

Resources are generated using a separate Perlin noise function with different parameters:

1. The same seed (plus an offset) ensures reproducibility
2. Different thresholds determine resource types:
   - Energy (most common): Values > 0.6
   - Minerals (less common): Values > 0.7
   - Scientific interest (rare): Values > 0.85
3. Resource amounts vary randomly but are seeded for reproducibility

### Resource Collection

The map includes a resource collection system:

1. Resources can be collected in specified amounts
2. Collection reduces the resource amount at that location
3. When the amount reaches zero, the resource is removed entirely
4. Appropriate error handling for invalid positions or insufficient resources

### Serialization/Deserialization

Custom serialization is implemented to handle tuple keys in the resource HashMap:

1. HashMap keys are converted to strings during serialization
2. The strings are parsed back to tuple coordinates during deserialization
3. The Perlin noise generator is recreated from the seed after deserialization
4. This allows maps to be saved and loaded while preserving all terrain and resource data

## Considerations

### Alternative Approaches Considered

1. **Different Noise Functions**: We considered using other noise functions like Simplex or Value noise. Perlin was chosen for its natural appearance and efficiency.

2. **Pre-defined Maps**: Instead of procedural generation, we considered using pre-defined maps. This approach was rejected because it would limit the variety of scenarios and the learning aspect of the project.

3. **Resource Regeneration**: We considered implementing resource regeneration over time. This was deferred as it would add complexity, and the current focus is on resource depletion.

4. **3D Terrain**: We considered a more complex 3D terrain model with height values. This was simplified to a 2D grid with terrain types to keep the simulation more accessible.

### Concerns

1. **Performance**: With large maps, generating Perlin noise for each cell could be computationally expensive. This is mitigated by generating the map once at initialization.

2. **Balancing**: The thresholds for terrain and resource generation may need adjustment to create balanced and interesting maps. The current values are based on preliminary testing.

3. **Visualization**: The terminal-based visualization has limitations for large maps. Future improvements may include more advanced visualization options.

## Decision

The implemented design provides a solid foundation for the Nova simulation with the following key decisions:

1. **Deterministic Generation**: Maps are reproducible from the same seed.

2. **Terrain Affects Movement**: Different terrain types have different movement costs, creating strategic decisions for robot navigation.

3. **Resource Distribution**: Resources are distributed according to noise patterns with varying rarity, creating exploration challenges.

4. **Persistent Maps**: Maps can be saved and loaded with all their state.

5. **Appropriate Error Handling**: The system handles edge cases like invalid coordinates and resource depletion.

Future enhancements might include:

- More terrain types with special properties
- Dynamic resource regeneration
- Environmental effects like day/night cycles
- Improved visualization options

## Other Related ADRs

No other ADRs at this time.

## References

- [Perlin Noise](https://en.wikipedia.org/wiki/Perlin_noise) - The basis for terrain generation
- [rust-noise Documentation](https://docs.rs/noise/latest/noise/) - The noise library used in the implementation
