# Assets Directory Structure

This directory contains all game assets organized by type.

## Directory Layout

```
assets/
├── textures/    # Sprites, backgrounds, UI elements
│   ├── characters/
│   ├── effects/
│   ├── ui/
│   └── arena/
├── sounds/      # Sound effects and music
│   ├── sfx/
│   └── music/
└── fonts/       # Font files for text rendering
```

## Texture Assets

**Location**: `textures/`  
**Supported Formats**: PNG, JPEG  
**Naming Convention**: lowercase with underscores (e.g., `player_idle.png`)

### Organization

- `characters/` - Character sprites and animations
- `effects/` - Particle effects, explosions, impact effects
- `ui/` - Menu backgrounds, buttons, icons
- `arena/` - Platforms, backgrounds, hazards

### Guidelines

- Use PNG for sprites with transparency
- Use power-of-2 dimensions when possible (64x64, 128x128, etc.)
- Keep file sizes reasonable (< 1MB per sprite)
- Group related sprites for potential atlasing

## Sound Assets

**Location**: `sounds/`  
**Supported Formats**: WAV, OGG, MP3  
**Naming Convention**: lowercase with underscores (e.g., `jump_sound.wav`)

### Organization

- `sfx/` - Short sound effects (jumps, hits, abilities)
- `music/` - Background music tracks

### Guidelines

- Use WAV or OGG for best quality
- Keep sound effects short (< 2 seconds)
- Normalize volume levels
- Use 44.1kHz sample rate

## Font Assets

**Location**: `fonts/`  
**Supported Formats**: TTF, OTF  
**Naming Convention**: Original font name

### Guidelines

- Include license information for each font
- Prefer fonts with full character set support
- Test readability at different sizes

## Asset Loading

Assets are loaded through the `AssetManager` system:

```rust
// Load a texture
let player_texture = asset_manager.load_texture("characters/player.png")?;

// Create color texture (useful for prototyping)
let red_square = asset_manager.create_color_texture("red", [255, 0, 0, 255])?;

// Check if asset exists
if asset_manager.asset_exists(AssetType::Texture, "player.png") {
    // ...
}
```

## Hot Reloading (Development Mode)

In debug builds, assets are automatically watched for changes:

```rust
// Check for reloaded assets
let reloaded = asset_manager.check_hot_reload();
for asset in reloaded {
    println!("Reloaded: {}", asset);
}
```

When you modify an asset file, it will be automatically reloaded in the game without restarting.

## Texture Atlases

For performance, multiple small sprites can be packed into a single texture atlas:

```rust
let mut builder = AtlasBuilder::new(512, 512);
builder.add_sprite("player", 64, 64);
builder.add_sprite("enemy", 64, 64);
let atlas = builder.build();

asset_manager.add_atlas("game_sprites", atlas);
```

## Performance Tips

1. **Use Atlases**: Pack related sprites into atlases to reduce draw calls
2. **Lazy Loading**: Only load assets when needed
3. **Cache Results**: Store handles, don't reload repeatedly
4. **Optimize Sizes**: Keep textures as small as possible
5. **Disable Hot Reload**: Hot reloading is debug-only for performance