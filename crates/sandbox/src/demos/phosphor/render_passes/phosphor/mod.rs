use crate::demos::phosphor::HDR_TEXTURE_FORMAT;

use antigen_wgpu::{
    wgpu::{
        BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
        BindingResource, BindingType, FragmentState, MultisampleState, PipelineLayoutDescriptor,
        PrimitiveState, RenderPipelineDescriptor, SamplerBindingType, ShaderStages,
        TextureSampleType, TextureViewDimension, VertexState,
    },
    BindGroupComponent, BindGroupLayoutComponent, DeviceComponent, RenderPipelineComponent,
    SamplerComponent, ShaderModuleComponent, TextureViewComponent,
};

pub fn phosphor_prepare_phosphor_decay(
    device: &DeviceComponent,
    phosphor_bind_group_layout: &mut BindGroupLayoutComponent,
    front_bind_group: &mut BindGroupComponent,
    back_bind_group: &mut BindGroupComponent,
    phosphor_decay_pipeline: &mut RenderPipelineComponent,
    uniform_bind_group_layout: &BindGroupLayoutComponent,
    phosphor_decay_shader: &ShaderModuleComponent,
    linear_sampler: &SamplerComponent,
    beam_buffer_view: &TextureViewComponent,
    phosphor_front_buffer_view: &TextureViewComponent,
    phosphor_back_buffer_view: &TextureViewComponent,
) -> Option<()> {
    let uniform_bind_group_layout = uniform_bind_group_layout.get()?;
    let phosphor_decay_shader = phosphor_decay_shader.get()?;
    let linear_sampler = linear_sampler.get()?;
    let beam_buffer_view = beam_buffer_view.get()?;
    let phosphor_front_buffer_view = phosphor_front_buffer_view.get()?;
    let phosphor_back_buffer_view = phosphor_back_buffer_view.get()?;

    // Phosphor bind group
    let phosphor_bind_group_layout =
        if let Some(phosphor_bind_group_layout) = phosphor_bind_group_layout.get() {
            phosphor_bind_group_layout
        } else {
            let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Phosphor Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

            phosphor_bind_group_layout.set_ready_with(bind_group_layout);
            phosphor_bind_group_layout.get().unwrap()
        };

    if front_bind_group.is_pending() {
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &phosphor_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&phosphor_back_buffer_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&beam_buffer_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&linear_sampler),
                },
            ],
            label: None,
        });
        front_bind_group.set_ready_with(bind_group);
    }

    if back_bind_group.is_pending() {
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &phosphor_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&phosphor_front_buffer_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&beam_buffer_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&linear_sampler),
                },
            ],
            label: None,
        });
        back_bind_group.set_ready_with(bind_group);
    }

    if phosphor_decay_pipeline.is_pending() {
        let pipeline_layout = device.create_pipeline_layout(&mut PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniform_bind_group_layout, &phosphor_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Phosphor decay pipeline
        println!("Creating phosphor decay pipeline");
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &phosphor_decay_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &phosphor_decay_shader,
                entry_point: "fs_main",
                targets: &[HDR_TEXTURE_FORMAT.into()],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        phosphor_decay_pipeline.set_ready_with(pipeline);
    }

    Some(())
}
