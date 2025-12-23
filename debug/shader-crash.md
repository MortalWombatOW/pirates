# Debug Report: Post-Processing Shader Crash

> **Status**: ✅ RESOLVED
> **Impact**: ~~Game crashes on startup (WGPU Validation Error) or Shader is invisible.~~
> **Date**: 2025-12-22

## 1. Environment & Context
*   **Platform**: macOS (Apple M1)
*   **Backend**: Metal (`bevy_render::renderer::AdapterInfo { backend: Metal, ... }`)
*   **Engine**: Bevy 0.15.1
*   **Feature**: "Ink and Parchment" Post-Processing Shader
*   **Files**: 
    *   `src/plugins/graphics.rs` (Render Graph Node, Pipeline)
    *   `assets/shaders/ink_parchment.wgsl` (Shader logic)

## 2. Issue Summary
We are attempting to apply a full-screen post-processing effect. The implementation uses a custom `ViewNode` in the Bevy Render Graph. We have encountered two distinct failure modes:
1.  **Invisible Shader**: The game runs, but the effect is not applied (screen looks normal).
2.  **Crash (Panic)**: The game crashes with a WGPU Validation Error due to `TextureFormat` mismatch between the Render Pipeline and the Render Pass.

## 3. Timeline of Attempts

### Attempt A: Hardcoded `Bgra8UnormSrgb`
*   **Implementation**: `targets: vec![Some(ColorTargetState { format: TextureFormat::Bgra8UnormSrgb, ... })]`
*   **Result**: Game runs, but shader is **invisible**.
*   **Debug Data**: Added debug code to shader (`return RED`), still invisible.
*   **Hypothesis**: The render pass was running, but perhaps configured incorrectly or inputs were invalid.

### Attempt B: Hardcoded `Bgra8Unorm`
*   **Implementation**: Changed format to `TextureFormat::Bgra8Unorm`.
*   **Result**: **CRASH** on startup.
*   **Error**:
    ```
    wgpu error: Validation Error
    In RenderPass::end ... Render pipeline targets are incompatible with render pass
    Incompatible color attachments at indices [0]: the RenderPass uses textures with formats [Some(Bgra8UnormSrgb)] but the RenderPipeline uses attachments with formats [Some(Bgra8Unorm)]
    ```
*   **Insight**: This confirmed the RenderPass on this M1 Mac expects `Bgra8UnormSrgb`.

### Attempt C: Specialized Pipeline (Current State)
*   **Implementation**: Implemented `SpecializedRenderPipeline` pattern.
    *   In `run()`: `let key = PostProcessPipelineKey { format: view_target.out_texture_format() };`
    *   In `specialize()`: Uses `key.format` to build the pipeline descriptor.
*   **Expected Behavior**: The pipeline format should dynamically match the view format (`Bgra8UnormSrgb`), preventing the crash.
*   **Actual Behavior**: User reports "Nope, it's crashing again".

## 4. The "Impossible Mismatch" Paradox
If the current code is using `view_target.out_texture_format()` to create the pipeline, and `view_target.out_texture()` to create the RenderPass attachment, they **must** match by definition.

If it is still crashing with "Incompatible color attachments", then either:
1.  **The Code Wasn't Deployed**: Is the running binary actually using the new code?
2.  **ViewTarget Inconsistency**: Does `view_target.out_texture_format()` report a different format than `view_target.out_texture()` actually possesses? (Unlikely in Bevy).
3.  **WGPU View Formats**: The texture might be `Bgra8Unorm`, but the *View* created for the RenderPass is `Bgra8UnormSrgb` (sRGB view on non-sRGB texture). If `out_texture_format()` returns the texture's underlying format instead of the view's format, we get a mismatch.

## 5. Relevant Code Snippets

### Render Node `run()`
```rust
fn run(..., (view_target, _), ...) {
    // 1. Determine Format from ViewTarget
    let format = view_target.out_texture_format();
    
    // 2. Specialize Pipeline
    let key = PostProcessPipelineKey { format };
    let descriptor = post_process_pipeline.specialize(key);
    let pipeline_id = pipeline_cache.queue_render_pipeline(descriptor);
    
    // 3. Create RenderPass with ViewTarget
    let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        color_attachments: &[Some(RenderPassColorAttachment {
            view: view_target.out_texture(), // <--- Must match 'format'
            ...
        })],
        ...
    });
}
```

## 6. Hypotheses & Investigation Paths

### H1: sRGB View Format Confusion
Bevy's `ViewTarget` might be creating an sRGB view for the render pass (to ensure linear blending works correctly), but `out_texture_format()` might return the underlying texture format.
*   **Check**: Does `TextureFormat::add_srgb_suffix()` need to be called on the format before creating the pipeline?
*   **Evidence**: The error in Attempt B said RenderPass uses `Bgra8UnormSrgb`. If `out_texture_format()` returned `Bgra8Unorm`, that would cause the crash.

### H2: Vertex Shader Mismatch
If the vertex shader `fullscreen_shader_vertex_state()` is incompatible with the pipeline layout or render pass in some subtle way (e.g., multisampling count), it might cause a validation error that looks like a format error or triggers a generic panic.
*   **Check**: `multisample: MultisampleState::default()` (count: 1). Does the main window have MSAA enabled? If so, `PostProcessNode` needs to handle resolve or match sample count. (Usually post-processing runs after resolve).

### H3: Bind Group Mismatch
The shader defines:
```wgsl
@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
```
If the binding layout in Rust doesn't match this exactly, WGPU will panic.
*   **Status**: Rust code seems to match (0: texture, 1: sampler).

### H4: Egui Conflict
We fixed a startup panic regarding `EguiPlugin`. If that fix wasn't sufficient, the "crashing again" might be the *old* panic returning, not the shader one.
*   **Action**: Check logs to confirm if the panic is `wgpu error` or `EguiUserTextures`.

## 7. Next Steps for Developer
1.  **Get the Logs**: The exact panic message is crucial. Is it "Incompatible color attachments" again?
2.  **Verify sRGB Logic**: Try explicitly forcing `.add_srgb_suffix()` on the format in `run()`.
3.  **Verify MSAA**: Check if `Msaa` resource is added/configured in `main.rs` or `CorePlugin`. Post-processing on the main view target usually inherits sample count 1 (resolved), but if we inserted the node *before* resolve (unlikely in `Core2d` graph), it would crash.

---

## 8. Investigation Log (2025-12-23)

### Attempt D: Reproduce & Analyze

**Actual Error**:
```
thread panicked at bevy_render-0.15.1/src/render_resource/pipeline_cache.rs:546:28:
index out of bounds: the len is 9 but the index is 9
```

**Backtrace Shows**:
```
6: bevy_render::render_resource::pipeline_cache::PipelineCache::get_render_pipeline
7: <pirates::plugins::graphics::PostProcessNode as ViewNode>::run
      at ./src/plugins/graphics.rs:109
```

**Root Cause Identified**:
The current code calls `pipeline_cache.queue_render_pipeline(descriptor)` every frame in `run()`. This:
1. Queues a *new* pipeline compilation request each frame
2. Returns incrementing `CachedRenderPipelineId` values (9, 10, 11...)
3. But `get_render_pipeline(id)` expects the pipeline to exist in the cache array
4. The array hasn't grown to match the queued IDs → **index out of bounds**

**Correct Pattern**:
For `SpecializedRenderPipeline`, Bevy requires using `SpecializedRenderPipelines<T>` resource which:
- Caches the specialized pipeline ID per key
- Only queues compilation once per unique key
- Returns the cached ID on subsequent calls

### Fix Applied (Attempt E)

**Changes Made**:
1. **Cached Pipeline ID**: Store `CachedRenderPipelineId` in `PostProcessPipeline` resource during `FromWorld` instead of creating new pipelines each frame.

2. **Use `post_process_write()`**: Changed from `main_texture_view()` + `out_texture()` to `view_target.post_process_write()` which provides proper double-buffering:
   - `post_process.source` - texture to read from (previous render output)
   - `post_process.destination` - texture to write to (guaranteed different from source)

3. **Correct Texture Format**: Use `Rgba8UnormSrgb` which matches Bevy's internal render texture format for non-HDR 2D cameras, not `Bgra8UnormSrgb` (swapchain format).

**Result**: ✅ RESOLVED - Game runs without crashes, shader effect applied.

---

## 9. Final Working Implementation

```rust
// In FromWorld - cache the pipeline ID once
let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
    // ...
    targets: vec![Some(ColorTargetState {
        format: TextureFormat::Rgba8UnormSrgb, // Internal render texture format
        // ...
    })],
});

// In run() - use double-buffered post-process textures
let post_process = view_target.post_process_write();
let bind_group = /* bind post_process.source as input texture */;
let render_pass = /* write to post_process.destination */;
```

## 10. Key Lessons Learned

1. **Never queue pipelines in `run()`** - Use `FromWorld` to cache `CachedRenderPipelineId`.
2. **Use `post_process_write()`** - Provides safe double-buffering for read/write operations.
3. **Internal vs Swapchain formats differ** - `Rgba8UnormSrgb` for internal textures, `Bgra8UnormSrgb` for Metal swapchain.
