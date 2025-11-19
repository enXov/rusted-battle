# Rusted Battle

A 2D multiplayer arena brawler built from scratch in Rust, inspired by Bopl Battle.

## Why This Game?

For years, I've harbored a dream of creating a 2D game with a living, breathing ecosystem—one where I could continuously add new features and abilities each month. Imagine introducing invisibility one month, cloning abilities the next, and countless creative mechanics thereafter. Despite this burning ambition, the dream remained just that: a dream that never materialized into reality.

Until now.

I'm building this game, drawing deep inspiration from [Bopl Battle](https://store.steampowered.com/app/1686940/Bopl_Battle/)—a game I genuinely admire for its charm and innovative gameplay. If I succeed in bringing this vision to life, I'll implement exactly what I've always envisioned: a platform for continuous innovation and monthly content that keeps the game fresh and exciting.

My choice of Rust is deliberate and intentional. Rust's primary selling point—zero memory leaks—is legendary in the programming community, and I want to put this claim to the test in my first major Rust project. A game serves as the perfect proving ground for this promise. As anyone familiar with modern gaming knows, optimization remains one of the industry's greatest challenges. If Rust delivers on its reputation, it should address this fundamental issue at its core XD. Who knows? Perhaps this project will even find its way to platforms like Steam one day.

## Features

- **Pure Rust**: Built from the ground up using Rust, no game engines
- **Zero Memory Leaks**: Leveraging Rust's safety guarantees for flawless performance
- **Cross-Platform**: Native builds for Windows and Linux
- **Open Source**: Free forever, community-driven development
- **Modular Architecture**: Clean separation of rendering, physics, and game logic

## Getting Started

### Prerequisites

**Required:**
- Rust (latest stable) - [Install Rust](https://rustup.rs/)
- GPU with Vulkan, DirectX 12, or Metal support
- Updated GPU drivers

**Linux:**
```bash
# Debian/Ubuntu
sudo apt install build-essential pkg-config libx11-dev libxi-dev libgl1-mesa-dev

# Arch Linux
sudo pacman -S base-devel libx11 libxi mesa
```

**Windows:**
- Visual Studio Build Tools

### Quick Start

```bash
# Clone the repository
git clone https://github.com/enxov/rusted-battle.git
cd rusted-battle

# Run in debug mode (faster compilation)
cargo run

# Run optimized release build
cargo run --release
```

### Development

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy -- -W clippy::all

# Run tests
cargo test

# Build release binary
cargo build --release
# Binary: target/release/rusted-battle (or .exe on Windows)
```

### Cross-Compilation

**Build Windows executable from Linux:**

```bash
# Install cross-compilation tools
cargo install cross
rustup target add x86_64-pc-windows-gnu

# Build Windows .exe
cross build --target x86_64-pc-windows-gnu --release
# Output: target/x86_64-pc-windows-gnu/release/rusted-battle.exe
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the AGPL-3.0 License - see the [LICENSE](LICENSE) file for details.

**Additional Terms**: Redistribution on Steam or similar commercial platforms requires explicit written permission from the copyright holder. This restriction does not apply to non-commercial distribution.
