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
        (view_target, _settings): (&ViewTarget, &InkParchmentSettings),
        world: &World,
    ) -> Result<(), NodeRunError> {
        let post_process_pipeline = world.resource::<PostProcessPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // Get the pipeline from the cache, specialized for the current view format
        let key = PostProcessPipelineKey {
            format: view_target.out_texture_format(),
        };
        let descriptor = post_process_pipeline.specialize(key);
        let pipeline_id = pipeline_cache.queue_render_pipeline(descriptor);

        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id) else {
            return Ok(());
        };

        // Create the bind group
        let bind_group = render_context.render_device().create_bind_group(
            "ink_parchment_bind_group",
            &post_process_pipeline.layout,
            &BindGroupEntries::sequential((
                view_target.main_texture_view(),
                &post_process_pipeline.sampler,
            )),
        );

        // Run the render pass
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ink_parchment_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: view_target.out_texture(),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(LinearRgba::BLACK.into()),
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
    shader: Handle<Shader>,
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

        Self {
            layout,
            sampler,
            shader,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct PostProcessPipelineKey {
    format: TextureFormat,
}

impl SpecializedRenderPipeline for PostProcessPipeline {
    type Key = PostProcessPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("ink_parchment_pipeline".into()),
            layout: vec![self.layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: key.format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        }
    }
}