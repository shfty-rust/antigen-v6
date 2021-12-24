use std::ops::Range;

use antigen_core::{AsUsage, Construct, Indirect, Usage};
use hecs::{Entity, EntityBuilder, World};
use wgpu::{
    BufferAddress, Color, DynamicOffset, IndexFormat, Operations, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, ShaderStages,
};

use crate::{
    BindGroupComponent, BufferComponent, CommandEncoderComponent, PushConstantQuery,
    RenderPipelineComponent, TextureViewComponent,
};

pub enum RenderPassTag {}

pub type RenderPassLabelComponent = Usage<RenderPassTag, Option<String>>;
pub type RenderPassColorAttachmentsComponent = Usage<
    RenderPassTag,
    Vec<(
        Indirect<&'static TextureViewComponent>,
        Option<Indirect<&'static TextureViewComponent>>,
        Operations<Color>,
    )>,
>;
pub type RenderPassDepthAttachmentComponent = Usage<
    RenderPassTag,
    Option<(
        Indirect<&'static TextureViewComponent>,
        Option<Operations<f32>>,
        Option<Operations<u32>>,
    )>,
>;
pub type RenderPassPipelineComponent =
    Usage<RenderPassTag, Indirect<&'static RenderPipelineComponent>>;
pub type RenderPassVertexBuffersComponent =
    Usage<RenderPassTag, Vec<(Indirect<&'static BufferComponent>, Range<BufferAddress>)>>;
pub type RenderPassIndexBufferComponent = Usage<
    RenderPassTag,
    Option<(
        Indirect<&'static BufferComponent>,
        Range<BufferAddress>,
        IndexFormat,
    )>,
>;
pub type RenderPassBindGroupsComponent =
    Usage<RenderPassTag, Vec<(Indirect<&'static BindGroupComponent>, Vec<DynamicOffset>)>>;
pub type RenderPassPushConstantsComponent =
    Usage<RenderPassTag, Vec<(Indirect<PushConstantQuery<'static>>, ShaderStages)>>;
pub type RenderPassDrawComponent = Usage<RenderPassTag, (Range<u32>, Range<u32>)>;
pub type RenderPassDrawIndexedComponent = Usage<RenderPassTag, (Range<u32>, u32, Range<u32>)>;
pub type RenderPassDrawIndirectComponent =
    Usage<RenderPassTag, (Indirect<&'static BufferComponent>, BufferAddress)>;

pub enum RenderPassBundle {}

impl RenderPassBundle {
    pub fn builder(
        label: Option<String>,
        color_attachments: Vec<(Entity, Option<Entity>, Operations<Color>)>,
        depth_attachment: Option<(Entity, Option<Operations<f32>>, Option<Operations<u32>>)>,
        pipeline: Entity,
        vertex_buffers: Vec<(Entity, Range<BufferAddress>)>,
        index_buffers: Option<(Entity, Range<BufferAddress>, IndexFormat)>,
        bind_groups: Vec<(Entity, Vec<DynamicOffset>)>,
        push_constants: Vec<(Entity, ShaderStages)>,
        draw: (Range<u32>, Range<u32>),
    ) -> EntityBuilder {
        let mut builder = EntityBuilder::new();

        builder.add(RenderPassLabelComponent::construct(label));

        let color_attachments = RenderPassColorAttachmentsComponent::construct(
            color_attachments
                .into_iter()
                .map(|(view, resolve_target, ops)| {
                    (
                        Indirect::construct(view),
                        resolve_target.map(Indirect::construct),
                        ops,
                    )
                })
                .collect(),
        );
        builder.add(color_attachments);

        let depth_attachment = RenderPassDepthAttachmentComponent::construct(depth_attachment.map(
            |(view, depth_ops, stencil_ops)| (Indirect::construct(view), depth_ops, stencil_ops),
        ));
        builder.add(depth_attachment);

        let pipeline = RenderPassPipelineComponent::construct(Indirect::construct(pipeline));
        builder.add(pipeline);

        let vertex_buffers = RenderPassVertexBuffersComponent::construct(
            vertex_buffers
                .into_iter()
                .map(|(entity, range)| (Indirect::construct(entity), range))
                .collect(),
        );
        builder.add(vertex_buffers);

        let index_buffer = RenderPassIndexBufferComponent::construct(
            index_buffers
                .map(|(entity, range, format)| (Indirect::construct(entity), range, format)),
        );
        builder.add(index_buffer);

        let bind_groups = RenderPassBindGroupsComponent::construct(
            bind_groups
                .into_iter()
                .map(|(entity, offsets)| (Indirect::construct(entity), offsets))
                .collect(),
        );
        builder.add(bind_groups);

        let push_constants = RenderPassPushConstantsComponent::construct(
            push_constants
                .into_iter()
                .map(|(entity, shader_stages)| (Indirect::construct(entity), shader_stages))
                .collect(),
        );
        builder.add(push_constants);

        let draw = RenderPassDrawComponent::construct(draw);
        builder.add(draw);

        builder
    }
}

#[derive(hecs::Query)]
pub struct RenderPassQuery<'a> {
    label: &'a RenderPassLabelComponent,
    color_attachments: &'a RenderPassColorAttachmentsComponent,
    depth_attachment: &'a RenderPassDepthAttachmentComponent,
    pipeline: &'a RenderPassPipelineComponent,
    vertex_buffers: &'a RenderPassVertexBuffersComponent,
    index_buffer: &'a RenderPassIndexBufferComponent,
    bind_groups: &'a RenderPassBindGroupsComponent,
    push_constants: Option<&'a RenderPassPushConstantsComponent>,
    draw: &'a RenderPassDrawComponent,
    encoder: &'a mut CommandEncoderComponent,
}

pub fn draw_render_passes_system(world: &mut World) -> Option<()> {
    for (
        entity,
        RenderPassQuery {
            label,
            color_attachments,
            depth_attachment,
            pipeline,
            vertex_buffers,
            index_buffer,
            bind_groups,
            push_constants,
            draw,
            encoder,
        },
    ) in world.query::<RenderPassQuery>().into_iter()
    {
        let encoder = encoder.get_mut()?;

        // Collect label
        let label = (**label).clone();
        let label = label.as_deref();

        // Collect color attachments
        let mut color_queries = color_attachments
            .iter()
            .map(|(view, resolve_target, ops)| {
                (
                    view.get(world),
                    resolve_target
                        .as_ref()
                        .map(|resolve_target| resolve_target.get(world)),
                    ops,
                )
            })
            .collect::<Vec<_>>();

        let color = color_queries.iter_mut().map(|(view, resolve_target, ops)| {
            (
                view.get().unwrap().get().unwrap(),
                resolve_target
                    .as_mut()
                    .map(|resolve_target| resolve_target.get().unwrap().get().unwrap()),
                ops,
            )
        });

        let color_attachments = color
            .map(|(view, resolve_target, ops)| {
                let ops = **ops;

                RenderPassColorAttachment {
                    view,
                    resolve_target,
                    ops,
                }
            })
            .collect::<Vec<_>>();

        // Collect depth stencil attachment
        let mut depth_stencil_query = depth_attachment
            .as_ref()
            .map(|(view, depth_ops, stencil_ops)| (view.get(world), depth_ops, stencil_ops));

        let depth_stencil = depth_stencil_query
            .as_mut()
            .map(|(query, depth_ops, stencil_ops)| {
                (query.get().unwrap().get().unwrap(), depth_ops, stencil_ops)
            });

        let depth_stencil_attachment = depth_stencil.map(|(view, depth_ops, stencil_ops)| {
            let depth_ops = **depth_ops;
            let stencil_ops = **stencil_ops;

            RenderPassDepthStencilAttachment {
                view,
                depth_ops,
                stencil_ops,
            }
        });

        // Collect pipeline
        let mut query = pipeline.get(world);
        let pipeline = query.get()?;
        let pipeline = pipeline.get()?;

        // Collect vertex buffer queries
        let mut vertex_buffer_queries = vertex_buffers
            .iter()
            .map(|(vertex_buffer, range)| (vertex_buffer.get(world), range))
            .collect::<Vec<_>>();

        let vertex_buffers = vertex_buffer_queries
            .iter_mut()
            .map(|(query, range)| {
                let bind_group = query.get().unwrap();
                (bind_group.get().unwrap(), range)
            })
            .collect::<Vec<_>>();

        // Collect index buffer query
        let mut index_buffer_query = index_buffer
            .as_ref()
            .map(|(index_buffer, range, format)| (index_buffer.get(world), range, format));

        let index_buffer = index_buffer_query.as_mut().map(|(query, range, format)| {
            let bind_group = query.get().unwrap();
            (bind_group.get().unwrap(), range, format)
        });

        // Collect bind group queries
        let mut bind_group_queries = bind_groups
            .iter()
            .map(|(bind_group, offsets)| (bind_group.get(world), offsets))
            .collect::<Vec<_>>();

        let bind_groups = bind_group_queries
            .iter_mut()
            .map(|(query, offsets)| {
                let bind_group = query.get().unwrap();
                (bind_group.get().unwrap(), offsets)
            })
            .collect::<Vec<_>>();

        // Collect push constant queries
        let mut push_constant_queries = if let Some(push_constants) = push_constants {
            let push_constant_queries = push_constants
                .iter()
                .map(|(push_constant, shader_stages)| (push_constant.get(world), shader_stages))
                .collect::<Vec<_>>();

            push_constant_queries
        } else {
            vec![]
        };

        let push_constants = push_constant_queries
            .iter_mut()
            .map(|(query, shader_stages)| (query.get().unwrap(), shader_stages))
            .collect::<Vec<_>>();

        // Begin render pass
        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label,
            color_attachments: &color_attachments,
            depth_stencil_attachment,
        });

        println!("Setting pipeline {:?}", pipeline);
        rpass.set_pipeline(pipeline);

        // Set vertex buffers
        for (i, (vertex_buffer, range)) in vertex_buffers.iter().enumerate() {
            println!(
                "Setting vertex buffer {}: {:?} with range {:?}",
                i as u32, vertex_buffer, range
            );
            rpass.set_vertex_buffer(i as u32, vertex_buffer.slice((***range).clone()));
        }

        // Set index buffer
        if let Some((index_buffer, range, format)) = index_buffer {
            println!(
                "Setting index buffer {:?} with range {:?} and format {:?}",
                index_buffer, range, format
            );
            rpass.set_index_buffer(index_buffer.slice((*range).clone()), **format);
        }

        // Set bind groups
        for (i, (bind_group, offsets)) in bind_groups.iter().enumerate() {
            println!(
                "Setting bind group {}: {:?} with offsets {:?}",
                i as u32, bind_group, offsets
            );
            rpass.set_bind_group(i as u32, bind_group, &offsets);
        }

        // Set push constants
        for (push_constant, shader_stages) in push_constants {
            println!(
                "Setting push constant with offset {}",
                **push_constant.offset
            );
            rpass.set_push_constants(
                **shader_stages,
                **push_constant.offset,
                &***push_constant.data,
            );
        }

        println!(
            "Drawing vertices {:?}, instances {:?} for entity {:?}",
            draw.0, draw.1, entity
        );
        rpass.draw(draw.0.clone(), draw.1.clone());
    }

    Some(())
}
