use bytemuck::{Pod, Zeroable};
use parking_lot::RwLock;
use rapier3d::prelude::IntersectionEvent;
use std::{borrow::Cow, collections::{BTreeMap, BTreeSet}, sync::Arc, time::Instant, marker::PhantomData};

use antigen_core::{Changed, LazyComponent, Usage};

// Phosphor renderer tag
pub struct PhosphorRenderer;

// Usage tags
pub enum StartTime {}
pub enum Timestamp {}
pub enum TotalTime {}
pub enum DeltaTime {}

pub struct BeamBuffer;
pub struct BeamMultisample;
pub struct BeamDepthBuffer;

#[derive(Debug, Copy, Clone)]
pub struct Vertices;

#[derive(Debug, Copy, Clone)]
pub struct TriangleIndices;

#[derive(Debug, Copy, Clone)]
pub struct TriangleMeshes;

#[derive(Debug, Copy, Clone)]
pub struct TriangleMeshInstances;

#[derive(Debug, Copy, Clone)]
pub struct LineVertices;

#[derive(Debug, Copy, Clone)]
pub struct LineIndices;

#[derive(Debug, Copy, Clone)]
pub struct LineMeshes;

#[derive(Debug, Copy, Clone)]
pub struct LineMeshInstances;

#[derive(Debug, Copy, Clone)]
pub struct LineInstances;

pub struct Uniform;
pub struct StorageBuffers;
pub struct PhosphorDecay;
pub struct PhosphorFrontBuffer;
pub struct PhosphorBackBuffer;
pub struct Beam;
pub struct BeamClear;
pub struct BeamLines;
pub struct BeamTriangles;
pub struct Tonemap;

pub enum MapFile {}

// Usage-tagged components
pub type StartTimeComponent = Usage<StartTime, Instant>;
pub type TimestampComponent = Usage<Timestamp, Instant>;
pub type TotalTimeComponent = Usage<TotalTime, f32>;
pub type DeltaTimeComponent = Usage<DeltaTime, f32>;

pub struct PerspectiveMatrix;
pub type PerspectiveMatrixComponent = Usage<PerspectiveMatrix, nalgebra::Matrix4<f32>>;

pub struct OrthographicMatrix;
pub type OrthographicMatrixComponent = Usage<OrthographicMatrix, nalgebra::Matrix4<f32>>;

pub struct Camera;

#[derive(Debug, Default, Copy, Clone)]
pub struct PlayerInputComponent {
    pub forward: f32,
    pub back: f32,
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
}

/// Mesh ID map
#[derive(Copy, Clone)]
pub struct TriangleMeshIds;
pub type TriangleMeshIdsComponent = Arc<RwLock<BTreeMap<Cow<'static, str>, u32>>>;

#[derive(Copy, Clone)]
pub struct LineMeshIds;
pub type LineMeshIdsComponent = Arc<RwLock<BTreeMap<Cow<'static, str>, (u32, u32)>>>;

// Line Mesh ID
pub enum LineMeshId {}
pub type LineMeshIdComponent = Usage<LineMeshId, u32>;

/// Singleton shader data
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct UniformData {
    perspective: [[f32; 4]; 4],
    orthographic: [[f32; 4]; 4],
    cam_pos: [f32; 4],
    cam_rot: [f32; 4],
    total_time: f32,
    delta_time: f32,
    _pad_0: [f32; 2],
}

/// Vertex data for 2D line meshes
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct LineVertexData {
    pub position: [f32; 3],
    pub end: f32,
}

pub type LineVertexDataComponent = Vec<LineVertexData>;

/// Vertex data for 3D triangle meshes
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct VertexData {
    pub position: [f32; 3],
    pub surface_color: [f32; 3],
    pub line_color: [f32; 3],
    pub intensity: f32,
    pub delta_intensity: f32,
    pub _pad: f32,
}

impl VertexData {
    pub fn new(
        position: (f32, f32, f32),
        surface_color: (f32, f32, f32),
        line_color: (f32, f32, f32),
        intensity: f32,
        delta_intensity: f32,
    ) -> Self {
        VertexData {
            position: [position.0, position.1, position.2],
            surface_color: [surface_color.0, surface_color.1, surface_color.2],
            line_color: [line_color.0, line_color.1, line_color.2],
            intensity,
            delta_intensity,
            ..Default::default()
        }
    }
}

pub type VertexDataComponent = Vec<VertexData>;

pub type TriangleIndexData = u16;
pub type TriangleIndexDataComponent = Vec<TriangleIndexData>;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct TriangleMeshData {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub index_offset: u32,
    pub vertex_offset: u32,
    pub _pad: u32,
}

pub type TriangleMeshDataComponent = Vec<TriangleMeshData>;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct TriangleMeshInstanceData {
    pub position: [f32; 3],
    pub _pad1: f32,
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub _pad2: f32,
}

pub type TriangleMeshInstanceDataComponent = Vec<TriangleMeshInstanceData>;

pub type LineIndexData = u32;
pub type LineIndexDataComponent = Vec<LineIndexData>;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct LineMeshData {
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub index_offset: u32,
    pub index_count: u32,
}

pub type LineMeshDataComponent = Vec<LineMeshData>;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct LineMeshInstanceData {
    pub position: [f32; 3],
    pub mesh: u32,
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub _pad: f32,
}

pub type LineMeshInstanceDataComponent = Vec<LineMeshInstanceData>;

/// Instance data representing a single line
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct LineInstanceData {
    pub mesh_instance: u32,
    pub line_index: u32,
}

pub type LineInstanceDataComponent = Vec<LineInstanceData>;

pub struct Oscilloscope {
    f: Box<dyn Fn(f32) -> (f32, f32, f32) + Send + Sync>,
    speed: f32,
    magnitude: f32,
}

impl Oscilloscope {
    pub fn new<F>(speed: f32, magnitude: f32, f: F) -> Self
    where
        F: Fn(f32) -> (f32, f32, f32) + Send + Sync + 'static,
    {
        Oscilloscope {
            speed,
            magnitude,
            f: Box::new(f),
        }
    }

    pub fn eval(&self, f: f32) -> (f32, f32, f32) {
        let (x, y, z) = (self.f)(f * self.speed);
        (x * self.magnitude, y * self.magnitude, z * self.magnitude)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Timer {
    pub timestamp: std::time::Instant,
    pub duration: std::time::Duration,
}

pub type TimerComponent = Changed<Timer>;

pub enum TriangleMeshInstance {}
pub type TriangleMeshInstanceComponent<'a> =
    Usage<TriangleMeshInstance, LazyComponent<(), Cow<'static, str>>>;

pub enum LineMeshInstance {}
pub type LineMeshInstanceComponent<'a> =
    Usage<LineMeshInstance, LazyComponent<(), Cow<'static, str>>>;

pub struct SharedShapes;
pub type SharedShapesComponent = Usage<
    SharedShapes,
    BTreeMap<
        String,
        Box<
            dyn Fn(nalgebra::Vector3<f32>) -> rapier3d::geometry::SharedShape
                + Send
                + Sync
                + 'static,
        >,
    >,
>;

pub struct EventInput;
pub type EventInputComponent<T> = Usage<EventInput, Vec<T>>;

pub struct EventOutput;
pub type EventOutputComponent<T> = Usage<EventOutput, Vec<T>>;

pub struct EventTransformComponent<I, O>(PhantomData<(I, O)>);

impl<I, O> Default for EventTransformComponent<I, O> {
    fn default() -> Self {
        EventTransformComponent(PhantomData)
    }
}

impl EventTransformComponent<(), ()> {
    pub fn unit() -> Self {
        Default::default()
    }
}

impl<I, O> EventTransformComponent<I, O> {
    pub fn with_input_type<T>(self) -> EventTransformComponent<T, O> {
        Default::default()
    }

    pub fn with_output_type<T>(self) -> EventTransformComponent<I, T> {
        Default::default()
    }
}

pub struct EulerAngles;
pub type EulerAnglesComponent = Usage<EulerAngles, nalgebra::Vector3<f32>>;

pub struct PositionOffset;
pub type PositionOffsetComponent =
    Usage<PositionOffset, (nalgebra::Vector3<f32>, nalgebra::Vector3<f32>)>;

pub struct RotationOffset;
pub type RotationOffsetComponent =
    Usage<RotationOffset, (nalgebra::Vector3<f32>, nalgebra::Vector3<f32>)>;

pub struct Speed;
pub type SpeedComponent = Usage<Speed, f32>;

pub struct MoverOpen;
pub type MoverOpenComponent = Usage<MoverOpen, bool>;

#[derive(Debug, Copy, Clone)]
pub enum MoverEvent {
    Open,
    Close,
}

pub type MoverEventInputComponent = EventInputComponent<MoverEvent>;
pub type MoverEventOutputComponent = EventOutputComponent<MoverEvent>;

pub type ColliderEventInputComponent = EventInputComponent<IntersectionEvent>;
pub type ColliderEventOutputComponent = EventOutputComponent<IntersectionEvent>;

pub struct EventIn;
pub type EventInComponent = Usage<EventIn, Cow<'static, str>>;

pub struct EventOut;
pub type EventOutComponent = Usage<EventOut, Cow<'static, str>>;

pub struct EventTarget<T>(PhantomData<T>);
pub type EventTargetComponent<T> = Usage<EventTarget<T>, Cow<'static, str>>;
