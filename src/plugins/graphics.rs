use bevy::{
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin},
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
        RenderApp,
    },
};

/// Plugin that adds the "Ink and Parchment" post-processing effect.
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<InkParchmentSettings>::default(),
            UniformComponentPlugin::<InkParchmentSettings>::default(),
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

/// Marker component for the camera that enables the Ink/Parchment effect.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType, Default, Reflect)]
#[reflect(Component)]
pub struct InkParchmentSettings {
    pub enabled: u32,
}

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
        &'static InkParchmentSettings,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings): (&ViewTarget, &InkParchmentSettings),
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Read the field to suppress unused warning.
        let _ = settings.enabled;

        let post_process_pipeline = world.resource::<PostProcessPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // Get the cached pipeline (queued during FromWorld)
        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id) else {
            // Pipeline is still compiling, skip this frame
            return Ok(());
        };

        // Get the post-process write struct which handles double-buffering.
        // This gives us a source texture (what was rendered) and a destination
        // texture (where we write our output). They are guaranteed to be different.
        let post_process = view_target.post_process_write();

        // Create the bind group using the SOURCE texture (what we read from)
        let bind_group = render_context.render_device().create_bind_group(
            "ink_parchment_bind_group",
            &post_process_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &post_process_pipeline.sampler,
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

        let layout = render_device.create_bind_group_layout(
            "ink_parchment_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/ink_parchment.wgsl");

        // Queue the pipeline for compilation.
        // On macOS/Metal, the swapchain format is Bgra8UnormSrgb.
        // This matches what ViewTarget.out_texture() uses for the final output.
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
                    // Use Rgba8UnormSrgb which is the internal render texture format
                    // for non-HDR cameras in Bevy's Core2d pipeline
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