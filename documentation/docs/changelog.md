# Changelog

All notable changes to Waremax.

---

## [Unreleased]

### Added
- Comprehensive MkDocs documentation
- User guide and tutorials
- Developer guide for extending Waremax

---

## [0.1.0] - Initial Release

### Added

#### Core Features
- Discrete event simulation engine
- Graph-based warehouse maps
- Robot movement and pathfinding
- Task allocation and execution
- Station service modeling

#### Robot Features
- Configurable fleet size
- Speed and movement parameters
- Battery management and charging
- Maintenance modeling (MTBF/MTTR)

#### Station Types
- Pick stations
- Drop stations
- Inbound/outbound stations
- Charging stations
- Maintenance stations

#### Policies
- Task allocation: nearest_idle, least_busy, round_robin
- Station assignment: nearest, shortest_queue, fastest_completion
- Routing: shortest_path, congestion_aware
- Batching support

#### Traffic Management
- Node and edge capacity
- Congestion detection
- Deadlock detection and resolution
- Rerouting support

#### Metrics
- Throughput tracking
- Task time breakdown
- Utilization metrics
- Time series export
- JSON and CSV output

#### CLI Commands
- `run` - Execute simulation
- `validate` - Check configuration
- `demo` - Quick demonstration
- `generate` - Create scenarios
- `sweep` - Parameter exploration
- `compare` - Compare configurations
- `ab-test` - Statistical comparison
- `benchmark` - Performance testing
- `stress-test` - Load testing
- `analyze` - Result analysis
- `list-presets` - Show presets

#### Presets
- minimal
- quick
- standard
- baseline
- high_load
- peak_hours
- stress_test
- battery_test
- maintenance_test

#### Configuration
- YAML-based scenarios
- Comprehensive validation
- Parameter overrides at runtime

---

## Version History

| Version | Date | Notes |
|---------|------|-------|
| 0.1.0 | TBD | Initial release |

---

## Versioning

Waremax follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Incompatible API changes
- **MINOR**: New features, backwards compatible
- **PATCH**: Bug fixes, backwards compatible

---

## Upgrade Guide

### From Pre-release to 0.1.0

No breaking changes from pre-release versions.

### Future Upgrades

Check this changelog for:
- Breaking changes (marked with ⚠️)
- Migration steps
- Deprecated features

---

## Contributing

See [Contributing Guide](developer/contributing/code-style.md) for how to contribute changes.

To add to this changelog:
1. Add entry under [Unreleased]
2. Use categories: Added, Changed, Deprecated, Removed, Fixed, Security
3. Include issue/PR references where applicable
