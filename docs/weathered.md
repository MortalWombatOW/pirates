# The Weathered Document: Technical Design Specification

> **Project**: Pirates — Ink & Parchment Aesthetic System

---

## 1. Executive Summary

This document describes the technical architecture for transforming Pirates from a game with a sepia filter into a game that looks and feels like an animated antique nautical chart. The player should feel they are looking at a physical artifact: aged parchment, hand-drawn with quill and iron gall ink, stained by time and sea spray.

We achieve this through three layered systems:

| Layer | Epic | Purpose |
|-------|------|---------|
| **Foundation** | 8.3 — The Weathered Document | Paper physicality (texture, aging, imperfections) |
| **Style** | 8.4 — The Cartographer's Sketch | Hand-drawn linework (edge detection, wobble) |
| **Life** | 8.5 — Living Ink | Dynamic ink behavior (reveals, trails, splatter) |

Each layer builds on the previous. They share a unified shader pipeline and coordinate through a central `AestheticSettings` resource.

---

## 2. Design Principles

### 2.1 Subtlety Over Spectacle
These effects should enhance immersion, not distract from gameplay. A player deeply engaged in combat should not consciously notice the paper grain—but they should *feel* something is different from a typical game.

### 2.2 Coherent Material Language
Every effect must answer: "How would this look on aged parchment drawn with a quill?" If an effect doesn't fit that metaphor, it doesn't belong.

### 2.3 Performance Budget
All effects run in a single post-processing pass. Target: <2ms GPU time on M1 Mac at 1080p. Profile early, profile often.

### 2.4 Tunable by Design
Every effect has float parameters exposed to a settings resource. Artists (or players) can dial effects up/down without code changes.

---

## 3. System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Main World                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ FogOfWar     │  │ InkReveal    │  │ AestheticSettings    │  │
│  │ (existing)   │  │ (new)        │  │ (new)                │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│         │                 │                    │                │
│         └─────────────────┴────────────────────┘                │
│                           │ ExtractComponent                    │
└───────────────────────────┼─────────────────────────────────────┘
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Render World                               │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                  PostProcessPipeline                      │  │
│  │  ┌─────────────────────────────────────────────────────┐ │  │
│  │  │              ink_parchment.wgsl                     │ │  │
│  │  │                                                     │ │  │
│  │  │  1. Sample screen texture                           │ │  │
│  │  │  2. Edge Detection (Sobel) ──────────► Edge Buffer  │ │  │
│  │  │  3. Apply ink/paper coloring                        │ │  │
│  │  │  4. Overlay paper texture + grain                   │ │  │
│  │  │  5. Apply vignette + stains                         │ │  │
│  │  │  6. Add edge wobble + line weight                   │ │  │
│  │  │  7. Composite ink reveals + trails                  │ │  │
│  │  │                                                     │ │  │
│  │  └─────────────────────────────────────────────────────┘ │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.1 Key Resources

#### `AestheticSettings` (Component on Camera, extracted to Render World)
```rust
#[derive(Component, ExtractComponent, ShaderType, Reflect)]
pub struct AestheticSettings {
    // Epic 8.3: Paper
    pub paper_texture_strength: f32,    // 0.0 - 1.0
    pub vignette_strength: f32,         // 0.0 - 1.0
    pub vignette_radius: f32,           // 0.3 - 0.8
    pub grain_strength: f32,            // 0.0 - 0.3
    pub grain_scale: f32,               // 50.0 - 200.0
    pub stain_strength: f32,            // 0.0 - 0.5
    pub ink_feather_radius: f32,        // 0.0 - 3.0 pixels

    // Epic 8.4: Edges
    pub edge_detection_enabled: u32,    // bool as u32
    pub edge_threshold: f32,            // 0.05 - 0.3
    pub edge_thickness: f32,            // 1.0 - 3.0
    pub wobble_amplitude: f32,          // 0.0 - 2.0 pixels
    pub wobble_frequency: f32,          // 5.0 - 20.0
    pub crosshatch_enabled: u32,        // bool as u32
    pub crosshatch_density: f32,        // 0.0 - 1.0

    // Epic 8.5: Dynamics (time-based)
    pub time: f32,                      // elapsed seconds
}
```

#### `InkRevealMap` (Resource, updated per frame)
A GPU texture (same resolution as fog tilemap) where each pixel's R channel represents reveal progress (0.0 = hidden, 1.0 = fully revealed). The G channel stores time-since-reveal for animation curves.

---

## 4. Epic 8.3: The Weathered Document

### 4.1 Paper Texture Overlay

**Approach**: Sample the existing `parchment.png` (1402x1408) and blend it with the scene.

```wgsl
// Screen-space tiling with slight rotation to avoid obvious repetition
let paper_uv = rotate2d(in.uv * paper_tile_scale, 0.02);
let paper = textureSample(paper_texture, paper_sampler, paper_uv);
let paper_factor = (paper.rgb - 0.5) * paper_texture_strength;
color.rgb += paper_factor;
```

**Considerations**:
- Tile at ~3x screen size to avoid visible seams
- Add subtle rotation (1-2 degrees) to break grid alignment
- Modulate by luminance: paper texture more visible on lighter areas

### 4.2 Vignette

**Approach**: Radial darkening from center, shaped like natural light falloff on paper.

```wgsl
let center = vec2(0.5, 0.5);
let dist = distance(in.uv, center);
let vignette = smoothstep(vignette_radius, vignette_radius + 0.3, dist);
color.rgb = mix(color.rgb, ink_color, vignette * vignette_strength);
```

**Considerations**:
- Use squared distance for natural falloff
- Vignette toward ink color, not black (maintains palette)
- Slightly asymmetric (heavier at bottom) mimics aged document

### 4.3 Paper Grain (FBM Noise)

**Approach**: Procedural noise layered at multiple octaves.

```wgsl
fn fbm(uv: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    for (var i = 0; i < octaves; i++) {
        value += amplitude * noise2d(uv * frequency * grain_scale);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

let grain = fbm(in.uv + time * 0.001, 4); // slight animation
color.rgb += (grain - 0.5) * grain_strength;
```

**Considerations**:
- 4 octaves balances quality vs. performance
- Animate slowly (0.001 * time) for organic feel without distraction
- Modulate grain by paper color—more visible on light areas

### 4.4 Ink Feathering

**Approach**: Blur dark edges slightly while keeping light areas crisp.

```wgsl
// Sample surrounding pixels
let blur = gaussianBlur3x3(screen_texture, in.uv, texel_size);
let luminance = dot(color.rgb, vec3(0.299, 0.587, 0.114));
// Feather only dark areas (ink)
let feather_mask = 1.0 - smoothstep(0.2, 0.5, luminance);
color.rgb = mix(color.rgb, blur, feather_mask * ink_feather_radius * 0.1);
```

### 4.5 Stain Spots (Voronoi)

**Approach**: Procedural stains using cellular/Voronoi noise.

```wgsl
fn voronoi(uv: vec2<f32>) -> f32 {
    let cell = floor(uv);
    var min_dist = 1.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let neighbor = cell + vec2(f32(x), f32(y));
            let point = neighbor + hash2d(neighbor);
            min_dist = min(min_dist, distance(uv, point));
        }
    }
    return min_dist;
}

let stain_noise = voronoi(in.uv * 3.0 + vec2(42.0, 17.0));
let stain_mask = smoothstep(0.1, 0.3, stain_noise);
let stain_color = vec3(0.7, 0.6, 0.5); // tea/coffee brown
color.rgb = mix(color.rgb, color.rgb * stain_color, (1.0 - stain_mask) * stain_strength);
```

**Considerations**:
- Stains should be sparse—3-5 visible on screen max
- Color shift toward brown/yellow, slight darkening
- Stains are STATIC (seed-based position), not animated

### 4.6 Fold Creases

**Approach**: Pre-authored crease positions rendered as subtle shadow lines.

```wgsl
// Two diagonal creases (as if document folded twice)
let crease1 = abs(in.uv.x + in.uv.y - 1.0);
let crease2 = abs(in.uv.x - in.uv.y);
let crease_mask = smoothstep(0.002, 0.01, min(crease1, crease2));
color.rgb *= mix(0.92, 1.0, crease_mask); // subtle darkening
```

---

## 5. Epic 8.4: The Cartographer's Sketch

### 5.1 Sobel Edge Detection

**Approach**: Classic Sobel operator in screen space.

```wgsl
fn sobel(uv: vec2<f32>, texel: vec2<f32>) -> f32 {
    // Sample 3x3 neighborhood (grayscale)
    let tl = luminance(sample(uv + vec2(-1, -1) * texel));
    let t  = luminance(sample(uv + vec2( 0, -1) * texel));
    let tr = luminance(sample(uv + vec2( 1, -1) * texel));
    let l  = luminance(sample(uv + vec2(-1,  0) * texel));
    let r  = luminance(sample(uv + vec2( 1,  0) * texel));
    let bl = luminance(sample(uv + vec2(-1,  1) * texel));
    let b  = luminance(sample(uv + vec2( 0,  1) * texel));
    let br = luminance(sample(uv + vec2( 1,  1) * texel));

    // Sobel kernels
    let gx = -tl - 2.0*l - bl + tr + 2.0*r + br;
    let gy = -tl - 2.0*t - tr + bl + 2.0*b + br;

    return sqrt(gx*gx + gy*gy);
}
```

### 5.2 Edge Rendering

**Approach**: Threshold edges, render as ink strokes over muted fill.

```wgsl
let edge = sobel(in.uv, texel_size);
let edge_mask = smoothstep(edge_threshold, edge_threshold + 0.1, edge);

// Mute the fill colors (push toward paper)
let muted_color = mix(color.rgb, paper_color, 0.3);

// Draw edges in ink
let final_color = mix(muted_color, ink_color, edge_mask);
```

### 5.3 Line Wobble

**Approach**: Displace UV before edge detection using sine waves.

```wgsl
fn wobble_uv(uv: vec2<f32>, time: f32) -> vec2<f32> {
    let wobble_x = sin(uv.y * wobble_frequency + time) * wobble_amplitude * texel_size.x;
    let wobble_y = cos(uv.x * wobble_frequency * 1.3 + time) * wobble_amplitude * texel_size.y;
    return uv + vec2(wobble_x, wobble_y);
}
```

**Considerations**:
- Wobble the UV used for edge detection, not the final output
- Different frequencies for X/Y to avoid uniform waves
- Animate slowly (time * 0.5) for organic drift, not jitter

### 5.4 Variable Line Weight

**Approach**: Dilate edges based on edge strength or luminance.

```wgsl
// Thicker lines for stronger edges
let line_weight = mix(1.0, edge_thickness, edge);
// Sample with variable kernel size
let thick_edge = max(
    sobel(in.uv, texel_size * line_weight),
    sobel(in.uv + texel_size * 0.5, texel_size * line_weight)
);
```

### 5.5 Crosshatch Shading

**Approach**: Procedural hatching pattern in dark areas.

```wgsl
fn crosshatch(uv: vec2<f32>, density: f32) -> f32 {
    let scale = 100.0;
    let line1 = abs(sin((uv.x + uv.y) * scale));
    let line2 = abs(sin((uv.x - uv.y) * scale));
    let hatch1 = step(density, line1);
    let hatch2 = step(density * 0.7, line2); // second layer slightly denser
    return min(hatch1, hatch2);
}

let darkness = 1.0 - luminance(color.rgb);
let hatch_mask = crosshatch(in.uv, crosshatch_density);
let hatched = mix(ink_color, paper_color, hatch_mask);
color.rgb = mix(color.rgb, hatched, darkness * crosshatch_enabled);
```

---

## 6. Epic 8.5: Living Ink

### 6.1 InkReveal System

**Architecture**:
1. `FogOfWar` tracks newly explored tiles (existing)
2. `InkRevealSystem` converts tile reveals to pixel-space animations
3. `InkRevealMap` texture uploaded to GPU each frame
4. Shader reads map and applies spreading ink effect

```rust
// Main World System
fn ink_reveal_system(
    fog: Res<FogOfWar>,
    time: Res<Time>,
    mut reveal_map: ResMut<InkRevealMap>,
) {
    for tile in fog.take_newly_explored() {
        reveal_map.start_reveal(tile, time.elapsed_seconds());
    }
    reveal_map.update_animations(time.elapsed_seconds());
}
```

### 6.2 Ink Spread Animation

**Approach**: Radial spread from center of revealed tile.

```wgsl
fn ink_spread(uv: vec2<f32>, center: vec2<f32>, progress: f32) -> f32 {
    let dist = distance(uv, center);
    let radius = progress * max_radius;

    // Organic edge using noise
    let edge_noise = noise2d(uv * 50.0 + center) * 0.2;
    let organic_radius = radius + edge_noise * radius;

    // Soft falloff at edge (ink bleeding)
    return smoothstep(organic_radius + 0.02, organic_radius, dist);
}
```

**Animation Curve**: Ease-out (fast start, slow finish) over ~0.5 seconds.

### 6.3 Ship Wake Trails

**Approach**: Particle-like trail stored in a scrolling buffer.

```rust
#[derive(Component)]
pub struct InkTrail {
    pub points: VecDeque<(Vec2, f32)>, // position, spawn_time
    pub max_points: usize,
    pub fade_duration: f32,
}
```

Rendered as connected line segments with decreasing alpha.

### 6.4 Damage Splatter

**Approach**: Spawn temporary ink splatter sprites at hit location.

```rust
fn spawn_ink_splatter(
    commands: &mut Commands,
    position: Vec2,
    intensity: f32, // based on damage amount
) {
    commands.spawn((
        SpriteBundle { /* splatter texture */ },
        InkSplatter {
            lifetime: Timer::from_seconds(0.3, TimerMode::Once),
            scale_curve: EaseOut,
        },
    ));
}
```

### 6.5 Water Ink Wash

**Approach**: Blend coastlines with watercolor bleeding effect.

```wgsl
// At tile boundaries (water/land), apply blur + color bleed
let boundary_dist = /* distance to nearest land tile */;
let wash_mask = smoothstep(0.0, 0.1, boundary_dist);
let wash_color = mix(water_tint, land_tint, 0.3);
color.rgb = mix(color.rgb, wash_color, (1.0 - wash_mask) * wash_strength);
```

---

## 7. Bind Group Layout

The post-process shader needs additional textures beyond the screen:

```rust
BindGroupLayoutEntries::sequential(
    ShaderStages::FRAGMENT,
    (
        // @binding(0): Screen texture (existing)
        texture_2d(TextureSampleType::Float { filterable: true }),
        // @binding(1): Screen sampler (existing)
        sampler(SamplerBindingType::Filtering),
        // @binding(2): Paper texture
        texture_2d(TextureSampleType::Float { filterable: true }),
        // @binding(3): Paper sampler
        sampler(SamplerBindingType::Filtering),
        // @binding(4): Ink reveal map
        texture_2d(TextureSampleType::Float { filterable: true }),
        // @binding(5): Settings uniform
        uniform_buffer::<AestheticSettings>(true),
    ),
)
```

---

## 8. Implementation Order

### Phase 1: Foundation (Epic 8.3.1 - 8.3.4)
1. Extend shader bind group to include paper texture
2. Add `AestheticSettings` component + extraction
3. Implement paper overlay, vignette, grain, feathering
4. **Milestone**: Game looks like it's drawn on paper

### Phase 2: Linework (Epic 8.4.1 - 8.4.4)
5. Add Sobel edge detection
6. Render edges as ink strokes
7. Add wobble and line weight variation
8. **Milestone**: Game looks hand-drawn

### Phase 3: Aging (Epic 8.3.5 - 8.3.7)
9. Add stains and creases
10. Create torn edge UI frame
11. **Milestone**: Paper feels aged and weathered

### Phase 4: Dynamics (Epic 8.5.1 - 8.5.4)
12. Implement `InkRevealMap` system
13. Add fog reveal animation
14. Add wake trails and splatter
15. **Milestone**: Ink feels alive

### Phase 5: Polish (Epic 8.4.5-6, 8.5.5-7)
16. Crosshatch shading
17. Water wash effect
18. Storm distortion
19. Threshold tuning pass
20. **Milestone**: Ship it

---

## 9. Performance Considerations

| Effect | Cost | Mitigation |
|--------|------|------------|
| Sobel (9 samples) | Medium | Use separable filter if needed |
| FBM Noise (4 octaves) | Medium | Reduce octaves on low-end |
| Paper texture sample | Low | Single bilinear sample |
| Voronoi (9 cells) | Medium | Pre-bake to texture if needed |
| Ink reveal map | Low | Only update changed regions |

**Target Budget**: 2ms total for all effects at 1080p on M1.

**Profiling Strategy**: Add `tracy` spans around each effect stage. Disable effects individually to measure isolated cost.

---

## 10. Testing Checklist

- [ ] Effects visible in all game states (MainMenu, HighSeas, Combat, Port)
- [ ] No visual artifacts at screen edges
- [ ] Paper texture tiles seamlessly
- [ ] Edge detection works on ships, UI, and terrain
- [ ] Wobble animation is smooth, not jittery
- [ ] Ink reveals complete within 0.5s
- [ ] Performance within budget on target hardware
- [ ] All settings tunable via `AestheticSettings`
- [ ] Effects can be disabled entirely (accessibility)

---

## 11. Open Questions

1. **Paper texture source**: Use existing `parchment.png` or create new seamless tile?
2. **Edge detection on UI**: Should egui panels get edge treatment or remain "clean"?
3. **Colorblind considerations**: Does sepia + ink palette cause issues?
4. **Mobile/web targets**: Do we need a "lite" mode with reduced effects?

---

## 12. References

- [Sobel Edge Detection (GLSL)](https://gist.github.com/Hebali/6ebfc66106459aacee6a9fac029d0115)
- [LYGIA Shader Library](https://lygia.xyz/generative) — WGSL noise functions
- [The Book of Shaders: Noise](https://thebookofshaders.com/11/)
- [Northway Games: Ink Bleed Fade](https://northwaygames.com/an-ink-bleed-fade/)
- [Artineering: Watercolor Shader](https://artineering.io/styles/watercolor)
- [Return of the Obra Dinn](https://obradinn.com/) — Visual inspiration (1-bit dithering, though we use continuous tones)
