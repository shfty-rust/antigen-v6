use std::sync::atomic::Ordering;

use antigen_core::get_tagged_entity;
use antigen_wgpu::{
    buffer_size_of,
    wgpu::{BufferAddress, COPY_BUFFER_ALIGNMENT},
    BufferDataBundle,
};
use hecs::{EntityBuilder, World};

use super::{
    LineIndices, LineInstanceData, LineInstances, LineMeshData, LineMeshIdComponent,
    LineMeshInstanceData, LineMeshInstances, LineMeshes, MeshIds, MeshIdsComponent, Oscilloscope,
    PositionComponent, RotationComponent, ScaleComponent, TriangleIndices, TriangleMeshData,
    TriangleMeshInstanceData, TriangleMeshInstances, TriangleMeshes, VertexData, Vertices, BLACK,
    BLUE, GREEN, MAX_TRIANGLE_MESH_INSTANCES, RED, WHITE,
};

/// Pad a list of triangle indices to COPY_BUFFER_ALIGNMENT
pub fn pad_align_triangle_list(indices: &mut Vec<u16>) {
    while (buffer_size_of::<u16>() * indices.len() as BufferAddress) % COPY_BUFFER_ALIGNMENT > 0 {
        indices.extend(std::iter::repeat(0).take(3));
    }
}

/// Assemble mesh vertices
pub fn vertices_bundle(world: &mut World, vertices: Vec<VertexData>) -> EntityBuilder {
    let mut builder = EntityBuilder::new();

    let vertex_entity = get_tagged_entity::<Vertices>(world).unwrap();

    let vertex_head = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthComponent>(vertex_entity)
        .unwrap();

    let vertex_count = vertices.len();

    let vertex_data = BufferDataBundle::new(
        vertices,
        buffer_size_of::<VertexData>() * vertex_head.load(Ordering::Relaxed),
        vertex_entity,
    );
    builder.add_bundle(vertex_data);

    vertex_head.fetch_add(vertex_count as BufferAddress, Ordering::Relaxed);

    builder
}

/// Assemble line indices for pre-existing mesh vertices
pub fn line_indices_bundle(world: &mut World, indices: Vec<u32>) -> EntityBuilder {
    let mut builder = EntityBuilder::new();

    let line_index_entity = get_tagged_entity::<LineIndices>(world).unwrap();
    let line_index_head = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthComponent>(line_index_entity)
        .unwrap();

    let index_count = indices.len();

    let index_data = BufferDataBundle::new(
        indices,
        buffer_size_of::<u32>() * line_index_head.load(Ordering::Relaxed),
        line_index_entity,
    );

    builder.add_bundle(index_data);

    line_index_head.fetch_add(index_count as BufferAddress, Ordering::Relaxed);

    builder
}

/// Assembles mesh vertices and line indices
pub fn line_mesh_bundle(
    world: &mut World,
    vertices: Vec<VertexData>,
    indices: Vec<u32>,
) -> EntityBuilder {
    let mut builder = EntityBuilder::new();

    let vertex_entity = get_tagged_entity::<Vertices>(world).unwrap();
    let line_index_entity = get_tagged_entity::<LineIndices>(world).unwrap();

    let vertex_offset = world
        .query_one_mut::<&antigen_wgpu::BufferLengthComponent>(vertex_entity)
        .unwrap()
        .load(Ordering::Relaxed);

    let index_offset = world
        .query_one_mut::<&antigen_wgpu::BufferLengthComponent>(line_index_entity)
        .unwrap()
        .load(Ordering::Relaxed);

    let vertex_count = vertices.len();
    let index_count = indices.len();

    builder.add_bundle(vertices_bundle(world, vertices).build());

    builder.add_bundle(line_indices_bundle(world, indices).build());

    builder.add_bundle(
        line_mesh_data_bundle(
            world,
            vertex_offset as u32,
            vertex_count as u32,
            index_offset as u32,
            index_count as u32,
        )
        .build(),
    );

    builder
}

pub fn line_mesh_data_bundle(
    world: &mut World,
    vertex_offset: u32,
    vertex_count: u32,
    index_offset: u32,
    index_count: u32,
) -> EntityBuilder {
    let mut builder = EntityBuilder::new();

    let line_mesh_entity = get_tagged_entity::<LineMeshes>(world).unwrap();
    let line_mesh_head = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthComponent>(line_mesh_entity)
        .unwrap();

    builder.add_bundle(BufferDataBundle::new(
        vec![LineMeshData {
            vertex_offset: vertex_offset,
            vertex_count: vertex_count,
            index_offset: index_offset,
            index_count: index_count,
        }],
        buffer_size_of::<LineMeshData>() * line_mesh_head.load(Ordering::Relaxed),
        line_mesh_entity,
    ));

    line_mesh_head.fetch_add(1, Ordering::Relaxed);

    builder
}

pub fn line_mesh_instance_bundle(
    world: &mut World,
    position: PositionComponent,
    rotation: RotationComponent,
    scale: ScaleComponent,
    line_mesh: LineMeshIdComponent,
    line_count: u32,
) -> EntityBuilder {
    let mut builder = EntityBuilder::new();

    let line_mesh_instance_entity = get_tagged_entity::<LineMeshInstances>(world).unwrap();
    let line_instance_entity = get_tagged_entity::<LineInstances>(world).unwrap();

    let line_mesh_instance_head = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthComponent>(line_mesh_instance_entity)
        .unwrap();
    let base_offset =
        buffer_size_of::<LineMeshInstanceData>() * line_mesh_instance_head.load(Ordering::Relaxed);

    builder.add_bundle(BufferDataBundle::new(
        position,
        base_offset,
        line_mesh_instance_entity,
    ));

    builder.add_bundle(BufferDataBundle::new(
        line_mesh,
        base_offset + buffer_size_of::<[f32; 3]>(),
        line_mesh_instance_entity,
    ));

    builder.add_bundle(BufferDataBundle::new(
        rotation,
        base_offset + buffer_size_of::<[f32; 4]>(),
        line_mesh_instance_entity,
    ));

    builder.add_bundle(BufferDataBundle::new(
        scale,
        base_offset + buffer_size_of::<[f32; 8]>(),
        line_mesh_instance_entity,
    ));

    let mesh_instance = line_mesh_instance_head.load(Ordering::Relaxed) as u32;
    line_mesh_instance_head.fetch_add(1, Ordering::Relaxed);

    let line_instance_head = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthComponent>(line_instance_entity)
        .unwrap();

    builder.add_bundle(BufferDataBundle::new(
        (0..line_count)
            .into_iter()
            .map(|i| LineInstanceData {
                mesh_instance,
                line_index: i,
            })
            .collect::<Vec<_>>(),
        buffer_size_of::<LineInstanceData>() * line_instance_head.load(Ordering::Relaxed),
        line_instance_entity,
    ));

    line_instance_head.fetch_add(line_count as BufferAddress, Ordering::Relaxed);

    builder
}

/// Assemble line indices for a vector of vertices in line list format
pub fn line_list_mesh_bundle(world: &mut World, vertices: Vec<VertexData>) -> EntityBuilder {
    let mut vs = 0u32;
    let indices = vertices
        .chunks(2)
        .flat_map(|_| {
            let ret = [vs, vs + 1];
            vs += 2;
            ret
        })
        .collect::<Vec<_>>();

    line_mesh_bundle(world, vertices, indices)
}

/// Assemble line indices for a vector of vertices in line strip format
pub fn line_strip_mesh_bundle(world: &mut World, vertices: Vec<VertexData>) -> EntityBuilder {
    let mut indices = (0..vertices.len() as BufferAddress).collect::<Vec<_>>();

    let first = indices.remove(0) as u32;
    let last = indices.pop().unwrap() as u32;
    let inter = indices.into_iter().flat_map(|i| [i as u32, i as u32]);
    let indices = std::iter::once(first)
        .chain(inter)
        .chain(std::iter::once(last))
        .collect();

    println!("Line strip indices: {:#?}", indices);

    line_mesh_bundle(world, vertices, indices)
}

/// Assembles an oscilloscope entity
pub fn oscilloscope_mesh_bundle(
    world: &mut World,
    name: &str,
    color: (f32, f32, f32),
    oscilloscope: Oscilloscope,
    intensity: f32,
    delta_intensity: f32,
) -> EntityBuilder {
    let mut builder = EntityBuilder::new();

    let line_mesh_entity = get_tagged_entity::<LineMeshes>(world).unwrap();

    builder.add(oscilloscope);

    let vertices = vec![
        VertexData {
            position: [0.0, 0.0, 0.0],
            surface_color: [color.0, color.1, color.2],
            line_color: [color.0, color.1, color.2],
            intensity,
            delta_intensity,
            ..Default::default()
        },
        VertexData {
            position: [0.0, 0.0, 0.0],
            surface_color: [color.0, color.1, color.2],
            line_color: [color.0, color.1, color.2],
            intensity,
            delta_intensity,
            ..Default::default()
        },
    ];

    let indices = vec![0u32, 1u32];
    let line_mesh = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthComponent>(line_mesh_entity)
        .unwrap()
        .load(Ordering::Relaxed) as u32;

    register_mesh_ids(
        world,
        &format!("oscilloscope_{}", name),
        None,
        Some((line_mesh, 1)),
    );

    builder.add_bundle(line_mesh_bundle(world, vertices, indices).build());

    builder
}

/// Assemble mesh vertices and indices
pub fn triangle_mesh_bundle(
    world: &mut World,
    vertices: Vec<VertexData>,
    mut indices: Vec<u16>,
) -> EntityBuilder {
    let mut builder = EntityBuilder::new();

    let vertex_entity = get_tagged_entity::<Vertices>(world).unwrap();
    let triangle_index_entity = get_tagged_entity::<TriangleIndices>(world).unwrap();

    // Vertices
    let vertex_head = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthComponent>(vertex_entity)
        .unwrap();

    let vertex_offset = buffer_size_of::<VertexData>() * vertex_head.load(Ordering::Relaxed);
    let vertex_count = vertices.len();

    builder.add_bundle(BufferDataBundle::new(
        vertices,
        vertex_offset,
        vertex_entity,
    ));

    vertex_head.fetch_add(vertex_count as u64, Ordering::Relaxed);

    // Indices
    pad_align_triangle_list(&mut indices);

    let triangle_index_head = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthComponent>(triangle_index_entity)
        .unwrap();

    let index_offset = buffer_size_of::<u16>() * triangle_index_head.load(Ordering::Relaxed);
    let index_count = indices.len();

    builder.add_bundle(BufferDataBundle::new(
        indices,
        index_offset,
        triangle_index_entity,
    ));

    triangle_index_head.fetch_add(index_count as u64, Ordering::Relaxed);

    builder
}

pub fn triangle_mesh_data_bundle(
    world: &mut World,
    vertex_count: u32,
    instance_count: u32,
    index_offset: u32,
    vertex_offset: u32,
    indexed_indirect_constructor: impl Fn(u64) -> EntityBuilder,
) -> EntityBuilder {
    let mut builder = EntityBuilder::new();

    let triangle_mesh_entity = get_tagged_entity::<TriangleMeshes>(world).unwrap();
    let triangle_mesh_instance_entity = get_tagged_entity::<TriangleMeshInstances>(world).unwrap();

    let triangle_mesh_head = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthComponent>(triangle_mesh_entity)
        .unwrap();

    builder.add_bundle(BufferDataBundle::new(
        vec![TriangleMeshData {
            vertex_count,
            instance_count,
            index_offset,
            vertex_offset,
            ..Default::default()
        }],
        buffer_size_of::<TriangleMeshData>() * triangle_mesh_head.load(Ordering::Relaxed),
        triangle_mesh_entity,
    ));

    builder.add_bundle(
        indexed_indirect_constructor(triangle_mesh_head.load(Ordering::Relaxed)).build(),
    );

    triangle_mesh_head.fetch_add(1, Ordering::Relaxed);

    let triangle_mesh_instance_heads = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthsComponent>(triangle_mesh_instance_entity)
        .unwrap();

    triangle_mesh_instance_heads.write().push(0);

    builder
}

pub fn triangle_mesh_instance_data_bundle(
    world: &mut World,
    mesh: u32,
    position: PositionComponent,
    rotation: RotationComponent,
    scale: ScaleComponent,
) -> EntityBuilder {
    let mut builder = EntityBuilder::new();

    let triangle_mesh_instance_entity = get_tagged_entity::<TriangleMeshInstances>(world).unwrap();

    let triangle_mesh_instance_heads = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthsComponent>(triangle_mesh_instance_entity)
        .unwrap();

    let mut triangle_mesh_instance_head = triangle_mesh_instance_heads.write();
    let triangle_mesh_instance_head = triangle_mesh_instance_head.get_mut(mesh as usize).unwrap();

    let base_offset = buffer_size_of::<TriangleMeshInstanceData>()
        * (*triangle_mesh_instance_head
            + (mesh * MAX_TRIANGLE_MESH_INSTANCES as u32) as BufferAddress);

    builder.add_bundle(BufferDataBundle::new(
        position,
        base_offset,
        triangle_mesh_instance_entity,
    ));

    builder.add_bundle(BufferDataBundle::new(
        rotation,
        base_offset + buffer_size_of::<[f32; 4]>(),
        triangle_mesh_instance_entity,
    ));

    builder.add_bundle(BufferDataBundle::new(
        scale,
        base_offset + buffer_size_of::<[f32; 8]>(),
        triangle_mesh_instance_entity,
    ));

    *triangle_mesh_instance_head += 1;

    builder
}

/// Assemble triangle indices for a list of vertices in triangle list format
pub fn triangle_list_mesh_bundle(
    world: &mut World,
    mut base_index: u16,
    vertices: Vec<VertexData>,
) -> EntityBuilder {
    let indices = vertices
        .chunks(3)
        .flat_map(|_| {
            let is = [base_index, base_index + 1, base_index + 2];
            base_index += 3;
            is
        })
        .collect::<Vec<_>>();

    triangle_mesh_bundle(world, vertices, indices)
}

/// Assemble triangle indices for a list of vertices in triangle fan format
pub fn triangle_fan_mesh_bundle(
    world: &mut World,
    base_index: u16,
    vertices: Vec<VertexData>,
) -> EntityBuilder {
    let mut current_index = base_index;
    let indices = (0..vertices.len() - 2)
        .flat_map(|_| {
            let is = [base_index, current_index + 1, current_index + 2];
            current_index += 1;
            is
        })
        .collect::<Vec<_>>();

    triangle_mesh_bundle(world, vertices, indices)
}

/// Assemble the Box Bot
pub fn box_bot_mesh_bundle(
    world: &mut World,
    triangle_indexed_indirect_builder: impl Fn(u64) -> EntityBuilder + Copy,
) -> Vec<EntityBuilder> {
    let vertex_entity = get_tagged_entity::<Vertices>(world).unwrap();
    let triangle_index_entity = get_tagged_entity::<TriangleIndices>(world).unwrap();
    let triangle_mesh_entity = get_tagged_entity::<TriangleMeshes>(world).unwrap();
    let line_mesh_entity = get_tagged_entity::<LineMeshes>(world).unwrap();

    // Fetch mesh ID and store into mesh ID map
    let triangle_mesh_head = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthComponent>(triangle_mesh_entity)
        .unwrap()
        .load(Ordering::Relaxed);

    let line_mesh_head = world
        .query_one_mut::<&mut antigen_wgpu::BufferLengthComponent>(line_mesh_entity)
        .unwrap()
        .load(Ordering::Relaxed);

    register_mesh_ids(
        world,
        "box_bot",
        Some(triangle_mesh_head as u32),
        Some((line_mesh_head as u32, 12)),
    );

    // Build mesh components
    let mut builders = vec![];

    let base_vertex = world
        .query_one_mut::<&antigen_wgpu::BufferLengthComponent>(vertex_entity)
        .unwrap()
        .load(Ordering::Relaxed) as u32;

    let base_triangle_index = world
        .query_one_mut::<&antigen_wgpu::BufferLengthComponent>(triangle_index_entity)
        .unwrap()
        .load(Ordering::Relaxed) as u32;

    // Body cube
    let mut builder = EntityBuilder::new();
    builder.add_bundle(
        triangle_mesh_bundle(
            world,
            vec![
                VertexData::new((1.0, 1.0, 1.0), BLACK, BLACK, 0.0, -16.0),
                VertexData::new((-1.0, 1.0, 1.0), BLACK, BLACK, 0.0, -16.0),
                VertexData::new((-1.0, 1.0, -1.0), BLACK, BLACK, 0.0, -16.0),
                VertexData::new((1.0, 1.0, -1.0), BLACK, BLACK, 0.0, -16.0),
                VertexData::new((1.0, -1.0, 1.0), BLACK, BLACK, 0.0, -16.0),
                VertexData::new((-1.0, -1.0, 1.0), BLACK, BLACK, 0.0, -16.0),
                VertexData::new((-1.0, -1.0, -1.0), BLACK, BLACK, 0.0, -16.0),
                VertexData::new((1.0, -1.0, -1.0), BLACK, BLACK, 0.0, -16.0),
            ]
            .into_iter()
            .map(|mut vd| {
                vd.position[0] *= 25.0;
                vd.position[1] *= 25.0;
                vd.position[2] *= 25.0;
                vd
            })
            .chain(
                vec![
                    VertexData::new((1.0, 1.0, 1.0), RED, RED, 2.0, -14.0),
                    VertexData::new((-1.0, 1.0, 1.0), RED, RED, 2.0, -14.0),
                    VertexData::new((-1.0, 1.0, -1.0), RED, RED, 2.0, -14.0),
                    VertexData::new((1.0, 1.0, -1.0), RED, RED, 2.0, -14.0),
                    VertexData::new((1.0, -1.0, 1.0), RED, RED, 2.0, -14.0),
                    VertexData::new((-1.0, -1.0, 1.0), RED, RED, 2.0, -14.0),
                    VertexData::new((-1.0, -1.0, -1.0), RED, RED, 2.0, -14.0),
                    VertexData::new((1.0, -1.0, -1.0), RED, RED, 2.0, -14.0),
                ]
                .into_iter()
                .map(|mut vd| {
                    vd.position[0] *= 10.0;
                    vd.position[1] *= 2.5;
                    vd.position[2] *= 2.5;
                    vd.position[2] -= 25.0;
                    vd
                }),
            )
            .collect(),
            vec![
                // Top
                0, 1, 2, 0, 2, 3, // Bottom
                4, 7, 5, 7, 6, 5, // Front
                3, 2, 6, 3, 6, 7, // Back
                0, 5, 1, 0, 4, 5, // Right
                0, 3, 7, 0, 7, 4, // Left
                1, 5, 6, 1, 6, 2,
            ]
            .into_iter()
            .chain(
                vec![
                    // Top
                    0, 1, 2, 0, 2, 3, // Bottom
                    4, 7, 5, 7, 6, 5, // Front
                    3, 2, 6, 3, 6, 7, // Back
                    0, 5, 1, 0, 4, 5, // Right
                    0, 3, 7, 0, 7, 4, // Left
                    1, 5, 6, 1, 6, 2,
                ]
                .into_iter()
                .map(|i| i + 8),
            )
            .collect(),
        )
        .build(),
    );

    builder.add_bundle(
        triangle_mesh_data_bundle(
            world,
            36 * 2,
            0,
            base_triangle_index,
            base_vertex,
            triangle_indexed_indirect_builder,
        )
        .build(),
    );

    builders.push(builder);

    // Cube lines
    builders.push(line_list_mesh_bundle(
        world,
        vec![
            VertexData::new((-25.0, -25.0, -25.0), RED, RED, 2.0, -30.0),
            VertexData::new((25.0, -25.0, -25.0), GREEN, GREEN, 2.0, -30.0),
            VertexData::new((25.0, -25.0, -25.0), GREEN, GREEN, 2.0, -30.0),
            VertexData::new((25.0, -25.0, 25.0), BLUE, GREEN, 2.0, -30.0),
            VertexData::new((25.0, -25.0, 25.0), BLUE, GREEN, 2.0, -30.0),
            VertexData::new((-25.0, -25.0, 25.0), WHITE, WHITE, 2.0, -30.0),
            VertexData::new((-25.0, -25.0, 25.0), WHITE, WHITE, 2.0, -30.0),
            VertexData::new((-25.0, -25.0, -25.0), RED, RED, 2.0, -30.0),
            VertexData::new((-25.0, 25.0, -25.0), RED, RED, 2.0, -30.0),
            VertexData::new((25.0, 25.0, -25.0), GREEN, RED, 2.0, -30.0),
            VertexData::new((25.0, 25.0, -25.0), GREEN, RED, 2.0, -30.0),
            VertexData::new((25.0, 25.0, 25.0), BLUE, RED, 2.0, -30.0),
            VertexData::new((25.0, 25.0, 25.0), BLUE, RED, 2.0, -30.0),
            VertexData::new((-25.0, 25.0, 25.0), WHITE, RED, 2.0, -30.0),
            VertexData::new((-25.0, 25.0, 25.0), WHITE, RED, 2.0, -30.0),
            VertexData::new((-25.0, 25.0, -25.0), BLACK, RED, 2.0, -30.0),
            VertexData::new((-25.0, -25.0, -25.0), RED, RED, 2.0, -30.0),
            VertexData::new((-25.0, 25.0, -25.0), RED, RED, 2.0, -30.0),
            VertexData::new((25.0, -25.0, -25.0), GREEN, GREEN, 2.0, -30.0),
            VertexData::new((25.0, 25.0, -25.0), GREEN, GREEN, 2.0, -30.0),
            VertexData::new((25.0, -25.0, 25.0), BLUE, BLUE, 2.0, -30.0),
            VertexData::new((25.0, 25.0, 25.0), BLUE, BLUE, 2.0, -30.0),
            VertexData::new((-25.0, -25.0, 25.0), WHITE, WHITE, 2.0, -30.0),
            VertexData::new((-25.0, 25.0, 25.0), WHITE, WHITE, 2.0, -30.0),
        ],
    ));

    builders
}

pub fn register_mesh_ids(
    world: &mut World,
    key: &str,
    triangle_mesh: Option<u32>,
    line_mesh: Option<(u32, u32)>,
) {
    let query = world.query_mut::<&mut MeshIdsComponent>().with::<MeshIds>();
    let (_, mesh_ids) = query.into_iter().next().unwrap();
    mesh_ids
        .write()
        .insert(key.into(), (triangle_mesh, line_mesh));
}

pub fn mesh_instance_builders(
    world: &mut World,
    mesh: &str,
    position: PositionComponent,
    rotation: RotationComponent,
    scale: ScaleComponent,
) -> Vec<EntityBuilder> {
    let mut builders = vec![];

    let query = world.query_mut::<&MeshIdsComponent>().with::<MeshIds>();
    let (_, mesh_ids) = query.into_iter().next().unwrap();
    dbg!("Fetching mesh", mesh);
    let (triangle_mesh, line_mesh) = mesh_ids.read()[mesh];

    if let Some(triangle_mesh) = triangle_mesh {
        builders.push(triangle_mesh_instance_data_bundle(
            world,
            triangle_mesh,
            position,
            rotation,
            scale,
        ));
    }

    if let Some((line_mesh, line_count)) = line_mesh {
        builders.push(line_mesh_instance_bundle(
            world,
            position.into(),
            rotation.into(),
            scale.into(),
            line_mesh.into(),
            line_count,
        ));
    }

    builders
}
