use bevy::{
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
        view::ViewTarget,
        RenderApp,
    },
};

/// Plugin that adds the "Ink and Parchment" post-processing effect.
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        // Load the paper texture and store the handle
        let asset_server = app.world().resource::<AssetServer>();
        let paper_handle: Handle<Image> = asset_server.load("sprites/ui/parchment.png");

        app.insert_resource(PaperTextureHandle(paper_handle));

        app.add_plugins((
            ExtractComponentPlugin::<AestheticSettings>::default(),
            UniformComponentPlugin::<AestheticSettings>::default(),
            ExtractResourcePlugin::<PaperTextureHandle>::default(),
        ));

        // We need to get the render app to set up the render graph
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_graph_node::<ViewNodeRunner<PostProcessNode>>(
                Core2d,
                PostProcessLabel,
            )
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::Tonemapping,
                    PostProcessLabel,
                    Node2d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<PostProcessPipeline>();
    }
}

// ----------------------------------------------------------------------------
// 1. Component & Settings
// ----------------------------------------------------------------------------

/// Settings for the Ink/Parchment aesthetic effect.
/// Attached to the camera entity.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType, Reflect)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[reflect(Component)]
#[repr(C)]
pub struct AestheticSettings {
    /// Strength of paper texture overlay (0.0 - 1.0)
    pub paper_texture_strength: f32,
    /// Strength of vignette darkening (0.0 - 1.0)
    pub vignette_strength: f32,
    /// Radius where vignette starts (0.3 - 0.8)
    pub vignette_radius: f32,
    /// Strength of paper grain noise (0.0 - 0.3)
    pub grain_strength: f32,
    /// Scale of grain noise pattern (50.0 - 200.0)
    pub grain_scale: f32,
    /// Strength of coffee/age stains (0.0 - 0.5)
    pub stain_strength: f32,
    /// Ink feathering blur radius in pixels (0.0 - 3.0)
    pub ink_feather_radius: f32,
    /// Elapsed time for animated effects
    pub time: f32,
}

impl Default for AestheticSettings {
    fn default() -> Self {
        Self {
            paper_texture_strength: 0.15,
            vignette_strength: 0.4,
            vignette_radius: 0.4,
            grain_strength: 0.08,
            grain_scale: 100.0,
            stain_strength: 0.1,
            ink_feather_radius: 1.0,
            time: 0.0,
        }
    }
}

/// Resource holding the paper texture handle for extraction to render world.
#[derive(Resource, Clone, ExtractResource)]
pub struct PaperTextureHandle(pub Handle<Image>);

// ----------------------------------------------------------------------------
// 2. Render Graph Label
// ----------------------------------------------------------------------------

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PostProcessLabel;

// ----------------------------------------------------------------------------
// 3. Render Node
// ----------------------------------------------------------------------------

#[derive(Default)]
struct PostProcessNode;

impl ViewNode for PostProcessNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static AestheticSettings,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings): (&ViewTarget, &AestheticSettings),
        world: &World,
    ) -> Result<(), NodeRunError> {
        let post_process_pipeline = world.resource::<PostProcessPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // Get the cached pipeline (queued during FromWorld)
        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id) else {
            // Pipeline is still compiling, skip this frame
            return Ok(());
        };

        // Get the paper texture from GPU images
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();
        let paper_handle = world.resource::<PaperTextureHandle>();

        let Some(paper_image) = gpu_images.get(&paper_handle.0) else {
            // Paper texture not loaded yet, skip this frame
            return Ok(());
        };

        // Get the post-process write struct which handles double-buffering.
        let post_process = view_target.post_process_write();

        // Create settings uniform buffer
        let settings_buffer = render_context.render_device().create_buffer_with_data(
            &BufferInitDescriptor {
                label: Some("aesthetic_settings_buffer"),
                contents: bytemuck::bytes_of(settings),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            },
        );

        // Create the bind group with all textures and settings
        let bind_group = render_context.render_device().create_bind_group(
            "ink_parchment_bind_group",
            &post_process_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &post_process_pipeline.sampler,
                &paper_image.texture_view,
                &post_process_pipeline.sampler,
                settings_buffer.as_entire_binding(),
            )),
        );

        // Run the render pass writing to the DESTINATION texture
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ink_parchment_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

// ----------------------------------------------------------------------------
// 4. Pipeline Resource
// ----------------------------------------------------------------------------

#[derive(Resource)]
struct PostProcessPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for PostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // Expanded bind group layout:
        // 0: screen texture
        // 1: screen sampler
        // 2: paper texture
        // 3: paper sampler
        // 4: settings uniform
        let layout = render_device.create_bind_group_layout(
            "ink_parchment_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // Screen texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // Screen sampler
                    sampler(SamplerBindingType::Filtering),
                    // Paper texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // Paper sampler
                    sampler(SamplerBindingType::Filtering),
                    // Settings uniform buffer
                    uniform_buffer::<AestheticSettings>(false),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        });

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/ink_parchment.wgsl");

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("ink_parchment_pipeline".into()),
            layout: vec![layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}
