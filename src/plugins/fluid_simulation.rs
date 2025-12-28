//! Fluid Simulation Plugin
//!
//! Custom Stable Fluids solver using Bevy Compute Shaders.
//! Uses double-buffering (ping-pong) for Metal/M1 compatibility.
//!
//! Reference: GPU Gems "Fast Fluid Dynamics Simulation on the GPU"

use bevy::{
    image::TextureFormatPixelInfo,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
        Render, RenderApp, RenderSet,
    },
};

use crate::components::{CombatEntity, Ship};
use crate::plugins::core::GameState;
use crate::resources::{WaterMaterial, WaterSettings};
use avian2d::prelude::LinearVelocity;

/// Grid resolution for the fluid simulation (256x256 per design spec).
pub const FLUID_GRID_SIZE: u32 = 256;

/// Workgroup size for compute shaders (8x8 is safe for Apple Silicon).
pub const WORKGROUP_SIZE: u32 = 8;

// ============================================================================
// Plugin
// ============================================================================

/// Plugin that manages the Stable Fluids simulation for Combat water.
pub struct FluidSimulationPlugin;

impl Plugin for FluidSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<FluidSimulationTextures>::default());
        app.add_plugins(ExtractResourcePlugin::<WakeTextureData>::default());

        // Initialize fluid textures when entering Combat
        app.add_systems(OnEnter(GameState::Combat), (
            setup_fluid_simulation,
            spawn_water_surface.after(setup_fluid_simulation),
        ));

        // Inject ship wakes into fluid simulation during Combat
        app.add_systems(
            FixedUpdate,
            inject_ship_wakes.run_if(in_state(GameState::Combat)),
        );

        // Cleanup when exiting Combat
        app.add_systems(OnExit(GameState::Combat), cleanup_fluid_simulation);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        // Add compute node to render graph
        render_app.add_systems(
            Render, 
            (
                prepare_fluid_bind_groups.in_set(RenderSet::PrepareBindGroups),
                write_wake_texture.in_set(RenderSet::Prepare),
            )
        );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(FluidSimulationLabel, FluidSimulationNode::default());
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<FluidComputePipeline>();
    }
}

// ============================================================================
// Resources
// ============================================================================

/// Holds the texture handles for the fluid simulation.
/// Uses double-buffering (ping-pong) to avoid read-write hazards.
#[derive(Resource, Clone, ExtractResource)]
pub struct FluidSimulationTextures {
    /// Velocity textures (RG32Float) - ping/pong pair
    pub velocity_a: Handle<Image>,
    pub velocity_b: Handle<Image>,

    /// Pressure textures (R32Float) - ping/pong pair
    pub pressure_a: Handle<Image>,
    pub pressure_b: Handle<Image>,

    /// Divergence texture (R32Float) - single texture
    pub divergence: Handle<Image>,

    /// Current ping-pong state (which buffer is "read" vs "write")
    pub ping: bool,
}

/// Simulation parameters that can be tuned.
#[derive(Resource, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct FluidParams {
    /// Grid dimensions
    pub grid_size: u32,
    /// Time step for simulation
    pub dt: f32,
    /// Viscosity (decay rate during advection)
    pub viscosity: f32,
    /// Number of Jacobi iterations for pressure solve
    pub jacobi_iterations: u32,
}

impl Default for FluidParams {
    fn default() -> Self {
        Self {
            grid_size: FLUID_GRID_SIZE,
            dt: 1.0 / 60.0,
            viscosity: 0.99,
            jacobi_iterations: 20,
        }
    }
}

/// Stores pre-computed wake texture data for GPU upload.
/// Updated in main world by inject_ship_wakes, extracted to render world.
#[derive(Resource, Clone, ExtractResource, Default)]
pub struct WakeTextureData {
    /// Complete texture data (256*256*8 bytes for RG32Float)
    pub data: Vec<u8>,
    /// Whether the data has been updated and needs GPU upload
    pub dirty: bool,
}

// ============================================================================
// Setup/Cleanup Systems
// ============================================================================

/// Creates the fluid simulation textures on entering Combat.
fn setup_fluid_simulation(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    info!("FluidSimulation: Setting up 256x256 fluid grid");

    // Create velocity textures (RG32Float for 2D velocity)
    let velocity_a = create_fluid_texture(&mut images, TextureFormat::Rg32Float, "velocity_a");
    let velocity_b = create_fluid_texture(&mut images, TextureFormat::Rg32Float, "velocity_b");

    // Create pressure textures (R32Float for scalar pressure)
    let pressure_a = create_fluid_texture(&mut images, TextureFormat::R32Float, "pressure_a");
    let pressure_b = create_fluid_texture(&mut images, TextureFormat::R32Float, "pressure_b");

    // Create divergence texture (R32Float, intermediate)
    let divergence = create_fluid_texture(&mut images, TextureFormat::R32Float, "divergence");

    commands.insert_resource(FluidSimulationTextures {
        velocity_a,
        velocity_b,
        pressure_a,
        pressure_b,
        divergence,
        ping: true,
    });

    commands.insert_resource(FluidParams::default());
    
    // Initialize wake texture data buffer (256x256 x 8 bytes for RG32Float)
    let grid_size = FLUID_GRID_SIZE as usize;
    commands.insert_resource(WakeTextureData {
        data: vec![0u8; grid_size * grid_size * 8],
        dirty: false,
    });

    info!("FluidSimulation: Textures created successfully");
}

/// Helper to create a storage texture for compute shaders.
fn create_fluid_texture(
    images: &mut Assets<Image>,
    format: TextureFormat,
    _label: &str,
) -> Handle<Image> {
    use bevy::render::render_asset::RenderAssetUsages;
    
    let size = Extent3d {
        width: FLUID_GRID_SIZE,
        height: FLUID_GRID_SIZE,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None, // Labels on Bevy Image assets not needed
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        // Enable CPU-GPU sync so modifications to image.data get uploaded
        asset_usage: RenderAssetUsages::all(),
        ..default()
    };

    // Initialize with zeros (wake data is written dynamically via RenderQueue)
    let bytes_per_pixel = format.pixel_size();
    image.data = vec![0u8; (FLUID_GRID_SIZE * FLUID_GRID_SIZE) as usize * bytes_per_pixel];

    images.add(image)
}

/// Cleans up fluid simulation resources when leaving Combat.
fn cleanup_fluid_simulation(mut commands: Commands) {
    info!("FluidSimulation: Cleaning up resources");
    commands.remove_resource::<FluidSimulationTextures>();
    commands.remove_resource::<FluidParams>();
    commands.remove_resource::<WakeTextureData>();
}

// ============================================================================
// Ship Wake Injection
// ============================================================================

/// Combat arena size for coordinate mapping
const COMBAT_ARENA_SIZE: f32 = 2000.0;

/// Wake splat radius in grid cells
const WAKE_SPLAT_RADIUS: i32 = 8;

/// Force multiplier for wake velocity
const WAKE_FORCE_MULTIPLIER: f32 = 0.5;

/// Injects ship velocities into the WakeTextureData buffer.
/// Uses Gaussian splatting to create smooth wake patterns.
/// The data is later uploaded to GPU in the render world.
fn inject_ship_wakes(
    mut wake_data: Option<ResMut<WakeTextureData>>,
    ships: Query<(&Transform, &LinearVelocity), With<Ship>>,
) {
    let Some(ref mut wake_data) = wake_data else { return };
    
    let grid_size = FLUID_GRID_SIZE as i32;
    let half_arena = COMBAT_ARENA_SIZE / 2.0;
    let grid_scale = FLUID_GRID_SIZE as f32 / COMBAT_ARENA_SIZE;
    
    // Note: We don't clear the buffer - allows simulation history to persist
    // Wake force is reduced to prevent saturation from accumulation
    
    // Process each ship
    let mut ship_count = 0;
    for (transform, velocity) in ships.iter() {
        let pos = transform.translation.truncate();
        let vel = velocity.0;
        
        // Skip if ship is barely moving
        if vel.length() < 1.0 {
            continue;
        }
        
        ship_count += 1;
        if ship_count == 1 {
            // Log first ship's values for debugging
            info!("[WAKE] Ship at ({:.1}, {:.1}) vel=({:.1}, {:.1}) speed={:.1}", 
                  pos.x, pos.y, vel.x, vel.y, vel.length());
        }
        
        // Convert world position to grid coordinates
        // World: -1000 to 1000, Grid: 0 to 255
        // Note: Flip Y because texture Y=0 is at top, but world Y grows upward
        let grid_x = ((pos.x + half_arena) * grid_scale) as i32;
        let grid_y = grid_size - 1 - ((pos.y + half_arena) * grid_scale) as i32;
        
        // Clamp to grid bounds
        if grid_x < 0 || grid_x >= grid_size || grid_y < 0 || grid_y >= grid_size {
            continue;
        }
        
        // Gaussian splat: write velocity to surrounding cells
        for dy in -WAKE_SPLAT_RADIUS..=WAKE_SPLAT_RADIUS {
            for dx in -WAKE_SPLAT_RADIUS..=WAKE_SPLAT_RADIUS {
                let px = grid_x + dx;
                let py = grid_y + dy;
                
                // Skip out-of-bounds pixels
                if px < 0 || px >= grid_size || py < 0 || py >= grid_size {
                    continue;
                }
                
                // Gaussian falloff based on distance
                let dist_sq = (dx * dx + dy * dy) as f32;
                let radius_sq = (WAKE_SPLAT_RADIUS * WAKE_SPLAT_RADIUS) as f32;
                
                // Skip pixels outside circular radius
                if dist_sq > radius_sq {
                    continue;
                }
                
                let sigma = WAKE_SPLAT_RADIUS as f32 / 2.0;
                let weight = (-dist_sq / (2.0 * sigma * sigma)).exp();
                
                // Scale velocity and apply weight
                let splat_vel = vel * WAKE_FORCE_MULTIPLIER * weight;
                
                // Calculate pixel index (RG32Float = 8 bytes per pixel)
                let pixel_idx = ((py * grid_size + px) * 8) as usize;
                
                if pixel_idx + 8 <= wake_data.data.len() {
                    // Read existing velocity
                    let existing_x = f32::from_le_bytes([
                        wake_data.data[pixel_idx],
                        wake_data.data[pixel_idx + 1],
                        wake_data.data[pixel_idx + 2],
                        wake_data.data[pixel_idx + 3],
                    ]);
                    let existing_y = f32::from_le_bytes([
                        wake_data.data[pixel_idx + 4],
                        wake_data.data[pixel_idx + 5],
                        wake_data.data[pixel_idx + 6],
                        wake_data.data[pixel_idx + 7],
                    ]);
                    
                    // Add new velocity (accumulate)
                    let new_x = existing_x + splat_vel.x;
                    let new_y = existing_y + splat_vel.y;
                    
                    // Write back
                    let bytes_x = new_x.to_le_bytes();
                    let bytes_y = new_y.to_le_bytes();
                    wake_data.data[pixel_idx..pixel_idx + 4].copy_from_slice(&bytes_x);
                    wake_data.data[pixel_idx + 4..pixel_idx + 8].copy_from_slice(&bytes_y);
                }
            }
        }
    }
    
    // Mark as dirty if we wrote any ship data
    if ship_count > 0 {
        wake_data.dirty = true;
    }
}

/// Render world system: writes WakeTextureData to GPU texture via RenderQueue.
fn write_wake_texture(
    wake_data: Option<Res<WakeTextureData>>,
    textures: Option<Res<FluidSimulationTextures>>,
    images: Res<RenderAssets<GpuImage>>,
    render_queue: Res<RenderQueue>,
) {
    let Some(wake_data) = wake_data else { return };
    let Some(textures) = textures else { return };
    
    // Only upload if data has changed
    if !wake_data.dirty {
        return;
    }
    
    // Get the GPU texture for velocity_a
    let Some(gpu_image) = images.get(&textures.velocity_a) else { return };
    
    // Write the wake data to the GPU texture
    render_queue.write_texture(
        gpu_image.texture.as_image_copy(),
        &wake_data.data,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(FLUID_GRID_SIZE * 8), // 8 bytes per pixel (RG32Float)
            rows_per_image: Some(FLUID_GRID_SIZE),
        },
        Extent3d {
            width: FLUID_GRID_SIZE,
            height: FLUID_GRID_SIZE,
            depth_or_array_layers: 1,
        },
    );
}

/// Spawns the visible water surface mesh with WaterMaterial.
fn spawn_water_surface(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    textures: Option<Res<FluidSimulationTextures>>,
) {
    info!("[DEBUG] spawn_water_surface: ENTERING");
    
    let Some(textures) = textures else {
        warn!("[DEBUG] spawn_water_surface: No textures available - skipping!");
        return;
    };
    
    info!("[DEBUG] spawn_water_surface: Textures found, velocity_a handle: {:?}", textures.velocity_a);

    // Create a large rectangle mesh for the water surface
    // Combat arena is roughly 2000x2000 units
    let water_size = 2000.0;
    let mesh_handle = meshes.add(Rectangle::new(water_size, water_size));
    info!("[DEBUG] spawn_water_surface: Created mesh {}x{}", water_size, water_size);

    // Create the water material with the velocity texture
    let material_handle = materials.add(WaterMaterial {
        settings: WaterSettings {
            max_speed: 100.0,
            time: 0.0,
            _padding1: 0.0,
            _padding2: 0.0,
        },
        velocity_texture: textures.velocity_a.clone(),
    });
    info!("[DEBUG] spawn_water_surface: Created WaterMaterial");

    // Spawn the water surface entity
    let entity = commands.spawn((
        Name::new("WaterSurface"),
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        Transform::from_xyz(0.0, 0.0, -10.0), // Behind ships
        CombatEntity, // Tag for cleanup
    )).id();

    info!("[DEBUG] spawn_water_surface: Spawned WaterSurface entity {:?} at z=-10", entity);
}

// ============================================================================
// Render Graph Label
// ============================================================================

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct FluidSimulationLabel;

// ============================================================================
// Compute Node
// ============================================================================

#[derive(Default)]
struct FluidSimulationNode;

impl Node for FluidSimulationNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Get pipeline (skip if not ready)
        let Some(pipeline) = world.get_resource::<FluidComputePipeline>() else {
            return Ok(());
        };

        // Get bind groups (skip if not prepared)
        let Some(bind_groups) = world.get_resource::<FluidBindGroups>() else {
            return Ok(());
        };

        let pipeline_cache = world.resource::<PipelineCache>();

        // Get all pipelines (skip if any still compiling)
        let Some(advection_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.advection_pipeline) else {
            return Ok(());
        };
        let Some(divergence_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.divergence_pipeline) else {
            return Ok(());
        };
        let Some(jacobi_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.jacobi_pipeline) else {
            return Ok(());
        };
        let Some(subtract_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.subtract_pipeline) else {
            return Ok(());
        };

        let workgroups = FLUID_GRID_SIZE / WORKGROUP_SIZE;

        // Pass 1: Advection - moves velocity field along itself
        {
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("fluid_advection_pass"),
                    timestamp_writes: None,
                });

            pass.set_pipeline(advection_pipeline);
            pass.set_bind_group(0, &bind_groups.advection, &[]);
            pass.dispatch_workgroups(workgroups, workgroups, 1);
        }

        // Pass 2: Divergence - calculate where fluid is compressing
        {
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("fluid_divergence_pass"),
                    timestamp_writes: None,
                });

            pass.set_pipeline(divergence_pipeline);
            pass.set_bind_group(0, &bind_groups.divergence, &[]);
            pass.dispatch_workgroups(workgroups, workgroups, 1);
        }

        // Pass 3: Jacobi Pressure Solve - iteratively solve for pressure
        // Run 20 iterations, ping-ponging between pressure textures
        for i in 0..JACOBI_ITERATIONS {
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("fluid_jacobi_pass"),
                    timestamp_writes: None,
                });

            pass.set_pipeline(jacobi_pipeline);
            
            // Alternate between jacobi_a (writes to B) and jacobi_b (writes to A)
            if i % 2 == 0 {
                pass.set_bind_group(0, &bind_groups.jacobi_a, &[]);
            } else {
                pass.set_bind_group(0, &bind_groups.jacobi_b, &[]);
            }
            
            pass.dispatch_workgroups(workgroups, workgroups, 1);
        }

        // Pass 4: Gradient Subtract - subtract pressure gradient for incompressibility
        {
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("fluid_subtract_pass"),
                    timestamp_writes: None,
                });

            pass.set_pipeline(subtract_pipeline);
            pass.set_bind_group(0, &bind_groups.subtract, &[]);
            pass.dispatch_workgroups(workgroups, workgroups, 1);
        }

        Ok(())
    }
}

// ============================================================================
// Bind Groups (prepared each frame)
// ============================================================================

/// Jacobi pressure iteration count
const JACOBI_ITERATIONS: u32 = 20;

#[derive(Resource)]
struct FluidBindGroups {
    advection: BindGroup,
    divergence: BindGroup,
    jacobi_a: BindGroup,     // pressure_a -> pressure_b
    jacobi_b: BindGroup,     // pressure_b -> pressure_a
    subtract: BindGroup,
}

fn prepare_fluid_bind_groups(
    mut commands: Commands,
    pipeline: Option<Res<FluidComputePipeline>>,
    textures: Option<Res<FluidSimulationTextures>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
) {
    let Some(pipeline) = pipeline else { return };
    let Some(textures) = textures else { return };

    // Get GPU textures
    let Some(vel_a) = gpu_images.get(&textures.velocity_a) else { return };
    let Some(vel_b) = gpu_images.get(&textures.velocity_b) else { return };
    let Some(pres_a) = gpu_images.get(&textures.pressure_a) else { return };
    let Some(pres_b) = gpu_images.get(&textures.pressure_b) else { return };
    let Some(div) = gpu_images.get(&textures.divergence) else { return };

    // Determine read/write based on ping-pong state
    let (vel_read, vel_write) = if textures.ping {
        (&vel_a.texture_view, &vel_b.texture_view)
    } else {
        (&vel_b.texture_view, &vel_a.texture_view)
    };

    // Advection: read vel_a, write vel_b
    let advection_bind_group = render_device.create_bind_group(
        "fluid_advection_bind_group",
        &pipeline.advection_layout,
        &BindGroupEntries::sequential((
            vel_read,
            vel_write,
        )),
    );

    // Divergence: read vel_b (post-advection), write divergence
    let divergence_bind_group = render_device.create_bind_group(
        "fluid_divergence_bind_group",
        &pipeline.divergence_layout,
        &BindGroupEntries::sequential((
            vel_write, // Read from the advected velocity
            &div.texture_view,
        )),
    );

    // Jacobi ping-pong: need both directions
    // jacobi_a: read pressure_a, read divergence, write pressure_b
    let jacobi_a_bind_group = render_device.create_bind_group(
        "fluid_jacobi_a_bind_group",
        &pipeline.jacobi_layout,
        &BindGroupEntries::sequential((
            &pres_a.texture_view,
            &div.texture_view,
            &pres_b.texture_view,
        )),
    );

    // jacobi_b: read pressure_b, read divergence, write pressure_a
    let jacobi_b_bind_group = render_device.create_bind_group(
        "fluid_jacobi_b_bind_group",
        &pipeline.jacobi_layout,
        &BindGroupEntries::sequential((
            &pres_b.texture_view,
            &div.texture_view,
            &pres_a.texture_view,
        )),
    );

    // Subtract: read vel_b, read pressure_a (final), write vel_a (final output)
    // After even Jacobi iterations, pressure_a is the result
    let subtract_bind_group = render_device.create_bind_group(
        "fluid_subtract_bind_group",
        &pipeline.subtract_layout,
        &BindGroupEntries::sequential((
            vel_write,              // Read advected velocity
            &pres_a.texture_view,   // Read solved pressure
            vel_read,               // Write final velocity (back to original read buffer)
        )),
    );

    commands.insert_resource(FluidBindGroups {
        advection: advection_bind_group,
        divergence: divergence_bind_group,
        jacobi_a: jacobi_a_bind_group,
        jacobi_b: jacobi_b_bind_group,
        subtract: subtract_bind_group,
    });
}

// ============================================================================
// Compute Pipeline Resource
// ============================================================================

#[derive(Resource)]
struct FluidComputePipeline {
    advection_layout: BindGroupLayout,
    advection_pipeline: CachedComputePipelineId,
    divergence_layout: BindGroupLayout,
    divergence_pipeline: CachedComputePipelineId,
    jacobi_layout: BindGroupLayout,
    jacobi_pipeline: CachedComputePipelineId,
    subtract_layout: BindGroupLayout,
    subtract_pipeline: CachedComputePipelineId,
}

impl FromWorld for FluidComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // Advection bind group layout: read velocity, write velocity
        let advection_layout = render_device.create_bind_group_layout(
            "fluid_advection_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Velocity texture (read)
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Velocity texture (write)
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rg32Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ),
            ),
        );

        // Divergence bind group layout: read velocity, write divergence
        let divergence_layout = render_device.create_bind_group_layout(
            "fluid_divergence_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Velocity texture (read)
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Divergence texture (write)
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::R32Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ),
            ),
        );

        // Jacobi bind group layout: read pressure, read divergence, write pressure
        let jacobi_layout = render_device.create_bind_group_layout(
            "fluid_jacobi_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Pressure texture (read)
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Divergence texture (read)
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Pressure texture (write)
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::R32Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ),
            ),
        );

        // Subtract bind group layout: read velocity, read pressure, write velocity
        let subtract_layout = render_device.create_bind_group_layout(
            "fluid_subtract_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // Velocity texture (read)
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Pressure texture (read)
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Velocity texture (write)
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rg32Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ),
            ),
        );

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/fluids.wgsl");

        let pipeline_cache = world.resource::<PipelineCache>();
        
        let advection_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("fluid_advection_pipeline".into()),
            layout: vec![advection_layout.clone()],
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "advect".into(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        let divergence_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("fluid_divergence_pipeline".into()),
            layout: vec![divergence_layout.clone()],
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "divergence".into(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        let jacobi_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("fluid_jacobi_pipeline".into()),
            layout: vec![jacobi_layout.clone()],
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "jacobi".into(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        let subtract_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("fluid_subtract_pipeline".into()),
            layout: vec![subtract_layout.clone()],
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: "subtract".into(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        Self {
            advection_layout,
            advection_pipeline,
            divergence_layout,
            divergence_pipeline,
            jacobi_layout,
            jacobi_pipeline,
            subtract_layout,
            subtract_pipeline,
        }
    }
}

