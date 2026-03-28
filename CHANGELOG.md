# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial public release
- Discrete event simulation engine for warehouse robotics
- YAML-based scenario configuration
- Multiple task allocation policies (nearest_idle, least_busy, round_robin)
- Station assignment strategies (nearest, shortest_queue, fastest_completion)
- Routing policies (shortest_path, congestion_aware)
- Parameter sweep functionality for optimization
- A/B testing and statistical comparison tools
- Benchmarking and stress testing capabilities
- Metrics collection and analysis
- CLI with run, validate, demo, sweep, compare, and analyze commands
- Built-in presets for quick simulations
- Comprehensive documentation

## [0.1.0] - 2025-02-06

### Added
- Core simulation framework
- Robot movement and task lifecycle modeling
- Battery and maintenance simulation
- Traffic management with capacity and congestion handling
- Deadlock detection and resolution
- Path reservation system
- Storage and map configuration
- Order generation (Poisson, burst, scheduled)
- Metrics export in multiple formats
- Web-based UI for visualization

[Unreleased]: https://github.com/waremax/waremax/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/waremax/waremax/releases/tag/v0.1.0
