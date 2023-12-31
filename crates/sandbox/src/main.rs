// TODO: [✓] Refactor antigen-fs and antigen-shambler to use message pattern instead of systems
//
// TODO: [✓] Reimplement component indirection
//
// TODO: [✓] Finish porting phosphor demo
//
// TODO: [✓] Refactor prepare function to avoid unnecessary resource creation
//           * Split into specific sub-functions to ease maintenance
//
// TODO: [✓] Split texture and view components for phosphor demo
//
// TODO: [✓] Refactor wgpu types to remove usage generics
//
// TODO: [✓] Finish generalized compute pipeline dispatch
//
// TODO: [✓] Codify buffer flipping as components + systems
//           * Will allow phosphor decay and tonemap to draw via ECS
//           [✓] Phosphor-specific implementation
//           [✓] Generalized implementation for antigen-wgpu
//
// TODO: [✓] Implement command buffer sorting
//           * Order of commands currently depends on ECS iteration order
//           * Best to encode order while recording, more concurrecy-friendly
//           * CommandBufferComponent<T>(BTreeMap<T, CommandBuffer>) where T: PartialOrd ?
//             * Provide T during render pass init
//             * Use type defaults for better ergonomics
//
// TODO: [✓] Update render pass draw ranges via system
//
// TODO: [✓] Replace line instances compute shader with storage buffer usage
//           * Bind mesh vertices as storage buffer
//           * Calculate base index as instance_index * 2
//           [✓] Clean up remaining references to compute
//
// TODO: [✓] Mesh instancing for phosphor renderer
//           * As per line_instancing notes in crate root
//           * Objective is to be able to load each SVG font grapheme once,
//             draw multiple copies without duplicating vertex data
//           * Will require a mesh instance abstraction to encode mesh ID + world position
//           * Should also inform data design for triangle mesh instancing,
//             and provide the basis for loading map entities as individual meshes / ECS entities
//           [✓] First working implementation with new data model
//           [✓] Separate mesh loading and instance creation
//           [✓] Implement text object - read string from map file, spawn grapheme line mesh instances
//           [✓] Instancing for triangle meshes
//               [✓] Fix incorrect instance positioning in beam_mesh vertex shader
//               [✓] Separate instance creation from mesh loading
//           [✓] Load room brush entities as separate meshes
//
// TODO: [✓] Improve mesh / line spawning ergonomics
//           * Manually creating a local mutable index and writing it back is too much boilerplate
//           * Too many state variables to pass around
//           * Is it feasible to read entities from the world when creating a builder?
//             * Ostensibly yes, since the count components are fetched by calling code
//
// TODO: [✓] Rotation and scale support for triangle and line meshes
//           * [✓] Use quaternions for rotation
//           * [✓] Vec3 for scale
//
// TODO: [✓] Respect angle and mangle when spawning point entities
//           * Will need to convert from quake-forward to wgpu-forward
//
// TODO: [✓] Stratify mesh loading
//           * Need to be able to create mesh instances by name instead of manually caching IDs
//           [✓] Store name-id map as component, write during mesh load, lookup during instancing
//
// TODO: [✓] Separate triangle / line mesh instance position, rotation, scale out into distinct components
//           * Should be able to create a single BufferWrite per member with appropriate offsets
//
// TODO: [✓] Implement filesystem thread map loading / building
//           * Need to be able to read and write buffers from different threads
//           * Use Arc<Buffer> and clone between threads
//             * Render thread holds buffers, meshes, render passes
//             * Game thread holds buffers, mesh instances
//             * Create a RemoteComponent<T> abstraction for sharing components across threads
//           [✓] Separate oscilloscope mesh creation from instancing
//           [✓] Separate test geo triangle mesh creation from instancing
//           [✓] Use Arc + RwLock around buffer LazyComponent to avoid having to force-create buffers before send
//           [✓] Reduce boilerplate for cross-thread setup
//               * Too much repetition in phosphor mod.rs
//           [✓] Move map processing to filesystem thread
//
// TODO: [✓] Replace room with brush
//           [✓] Generalized component support for brush entities
//               * Should be able to use any point entity component
//               * Treat entity center as transform origin
//           [✓] Generalize face culling via special properties
//
// TODO: [✓] Separate box bot from player start
//           * Player start should represent the camera for now
//           * Implement as a box_bot point entity
//
// TODO: [✓] Refactor TB oscilloscope handling
//           * Semantically, oscilloscope is an animation over a line segment
//             * Should be able to split off into an animation component
//             * Leave line mesh creation and instancing to their respective properties
//
// TODO: [✓] Refactor TB text handling
//
// TODO: [✓] Consistent use_target_targetname overrides for all name-identified properties
//     * Visual meshes, convex hulls, trimeshes, etc.
//     * Allows for nice TrenchBroom visualization
//     * Nice to have enabled by default for subclass entities
// 
// TODO: [✗] Fix wayland behavior
//           * PresentMode::Fifo seems to work when windowed
//             * Other modes cause flickering
//             * Fullscreen causes flickering
//               * Need to see if setting the winit window to fullscreen will fix this
//           * No global keyboard input
//             * Probably best to switch to window events rather than device events
//
//           * Works as expected with WINIT_UNIX_BACKEND=x11
//             * Only issue is fullscreen running at 60FPS instead of monitor-native refresh
//               * Performance appears to scale with window size,
//                 so this may be down to the renderer implementation
//             * Rendering issues likely specific to nvidia + wayland
//             * Keyboard issues may be a result of wayland security protocols
//
// TODO: [>] Implement generalized render pass dispatch
//           [✓] Draw implementation
//           [✓] Draw indexed implementation
//           [✓] Implement remaining RenderPass parameters
//           [✓] Draw indirect implementation
//           [✓] Draw indexed indirect implementation
//           [ ] Multi-draw implementations
//           [ ] Execute Bundles implementation
//           [ ] Struct parameters for bundle constructors
//               * wgpu descriptors, but with entities instead of references
//           [ ] Builder pattern for RenderPass bundles?
//
// TODO: [>] Integrate rapier physics
//           [✓] Create collision from brush hulls
//           [✓] Scale support for colliders
//               * Rapier has no concept of scale
//               * Will need to generate one SharedShape instance for each scaled entity
//                 * Multiply ball radius by largest scale axis
//                 * Multiply cuboid extents by scale
//                 * Scale vertices for convex hulls and trimeshes
//           [✓] Trimesh brush collision
//           [>] Sensors
//           [>] Contact / intersection event handling
//               * Receiver component queues up events during collision tick
//                 * Use custom EventHandler to queue
//                 * Better than querying NarrowPhase
//                   * May miss certain CCD events that begin and end within one frame
//               * Systems read events after collision tick
//               * Queues are cleared before next frame
//               * Would be useful to have a TB-side wiring solution for this
//                 * Ex. triggers -> doors, timers, etc
//           [ ] Kinematic Controller
//
// TODO: [>] Fix lines projecting from behind the camera
//           [ ] Fix corner case
//               * Appears to be a precision issue
//               * May be using camera position instead of near plane as clipping predicate
//               * When very far away from the camera,
//                 lines begin to duplicate at the opposite end of world space
//
// TODO: [>] Implement camera abstraction
//           [>] First-person controls
//           [ ] Spawn at first player start
//           [ ] Mouse capture
//
// TODO: [ ] Fix compound convex hulls behaving incorrectly under scaling
//
// TODO: [ ] Use AABBs to determine brush entity center
//           * Current method allows brush splits to influence center
//
// [ ] 'component' point entity class
//     * Inherit from base entity class
//     * Use target / targetname to remotely add components to a 'point' entity
//       * Can use attach / detach events to spawn and despawn component
//     [ ] Need to allow spawning entities without components
//         * Each point / brush entity should map to a hecs entity regardless
//
// [ ] 'inherit' point entity class
//     * Use target / targetname to identify parent
//     * Re-instance parent with optional extra / overridden components
//     * Allows for composition in TB
//     * Need to think of a better name - too associated with OOP semantics
//
// TODO: [ ] Text entity refactor
//           * Needs to work as a component that controls a set of text mesh instance entities
//           * Should be able to update mesh instances when the underlying string changes
//           * Take inspiration from terminal emulators
//             * Use control characters for color, blink, etc
//               * Could extend if unused control characters exist
//                 * Fading, text animations, etc
//             * Damage system for reusing untouched text mesh instances
//           * Use-case for parent/child relation - transforms
//
// TODO: [ ] Figure out why lower-case z is missing from text test
//
// TODO: [ ] Implement compute-based frustum culling
//
// TODO: [ ] Implement generalized render pass setup
//
// TODO: [ ] Implement portal rendering
//           * Ideally all portal rendering should happen in existing draw calls for performance's sake
//               * Just add more geometry
//                 * Effectively an extra layer of room -> mesh instances indirection
//                 * Re-instance rooms and their contents when viewed through a portal
//               * Stencil buffer seems the best approach to early-out from invisible fragments
//               * Each portal recursion adds 1 to the stencil value
//               * Use less-than stencil comparator
//          * Will need a way to track the current room in order to begin portal traversal
//            * Point-in-box checks against room hulls
//            * If camera is not inside a room, find the closest one
//              * Ideally should use distance-to-nearest-surface
//              * If impractical, distance-to-center should suffice
//            * Updating current room on portal traversal will be more efficient
//              after starting sector has been determined
//          * Rendering the whole scene twice with a small offset is a good place to start
//
// TODO: [ ] Investigate box portals for room-inside-room
//
// TODO: [ ] Generalize map -> entities + components conversion
//           * Need a way to map classname to a set of entities, properties to components
//          [>] Catch-all Point and Brush entity classnames
//             * Collects all relevant components into single classnames
//             * Specialize to bundle-like constructs by subclassing in FGD and overriding with default values
//             * Covers both pre-made and fully-customizable cases
//           * Simple bool property to instantiate component
//             * Allows defaults to commicate on/off for subclasses
//           * component.member naming to map to component members
//           * Traits + cons lists to model classname -> components relation?
//             * Would be ideal to do this at build-time
//             * Could use plugin-registry from antigen-v4
//             * Separate build target that draws from the registered types
//               and outputs a TrenchBroom game config + fgd
//             * Should allow for both tool and runtime usage via shared code
//               * Tool use case can be a CLI program using args + stdout
//               * Runtime usage should embody the 'game as its own editor' paradigm
//                 * Same functionality, different interface
//
// TODO: [ ] TrenchBroom special entity support for shambler
//           * Implement as its own GeoMap-dependent struct
//
// TODO: [ ] Surface / Content flags support for shambler
//           * Should be able to use for trimesh collision lookup,
//             provided that rapier returns face information
//
// TODO: [ ] Implement HDR bloom pass
//
// TODO: [ ] Calculate exact near plane intersection for line clipping
//           * Currently using an arbitrarily large value
//           * Could have overdraw implications
//           * Should probably clip in view space instead of NDC for this
//

mod demos;

use antigen_core::{
    receive_messages, send_clone_query, try_receive_messages, NamedEntitiesComponent,
    PositionComponent, RotationComponent, ScaleComponent, TaggedEntitiesComponent, WorldChannel,
    WorldExchange,
};
use antigen_wgpu::{
    wgpu::DeviceDescriptor, AdapterComponent, DeviceComponent, InstanceComponent, QueueComponent,
};
use antigen_winit::EventLoopHandler;
use demos::phosphor::{LineMeshInstance, MoverEvent, TriangleMeshInstance};
use rapier3d::prelude::IntersectionEvent;
use std::{
    thread::JoinHandle,
    time::{Duration, Instant},
};
use winit::{event::Event, event_loop::ControlFlow, event_loop::EventLoopWindowTarget};

use hecs::{EntityBuilder, World};

use antigen_rapier3d::physics_backend_builder;

const GAME_THREAD_TICK: Duration = Duration::from_nanos(16670000);

enum Game {}
enum Render {}
enum Filesystem {}

fn main() {
    //tracing_subscriber::fmt::fmt().pretty().init();

    // Create world exchange
    let mut exchange = WorldExchange::default();

    // Create thread-specific channels
    let fs_channel = exchange.create_channel::<Filesystem>();
    let game_channel = exchange.create_channel::<Game>();
    let render_channel = exchange.create_channel::<Render>();

    // Spawn exchange into its own thread
    exchange.spawn();

    // Create worlds
    let fs_world = World::new();
    let mut game_world = World::new();
    let mut render_world = World::new();

    // Setup game world
    game_world.spawn((TaggedEntitiesComponent::default(),));
    game_world.spawn((NamedEntitiesComponent::default(),));

    let mut builder = EntityBuilder::new();
    builder.add(demos::phosphor::SharedShapes);
    builder.add(demos::phosphor::SharedShapesComponent::default());
    game_world.spawn(builder.build());

    // Setup render world
    render_world.spawn((TaggedEntitiesComponent::default(),));
    render_world.spawn(antigen_winit::BackendBundle::default());

    let wgpu_backend_entity = render_world.spawn(antigen_wgpu::BackendBundle::from_env(
        &DeviceDescriptor {
            label: Some("Device"),
            features: Default::default(),
            limits: Default::default(),
        },
        None,
        None,
    ));

    let mut builder = EntityBuilder::new();
    builder.add(demos::phosphor::TriangleMeshIds);
    builder.add(demos::phosphor::TriangleMeshIdsComponent::default());
    let triangle_mesh_ids_entity = render_world.spawn(builder.build());

    let mut builder = EntityBuilder::new();
    builder.add(demos::phosphor::LineMeshIds);
    builder.add(demos::phosphor::LineMeshIdsComponent::default());
    let line_mesh_ids_entity = render_world.spawn(builder.build());

    // Clone mesh IDs to game thread
    send_clone_query::<
        (
            &demos::phosphor::TriangleMeshIds,
            &demos::phosphor::TriangleMeshIdsComponent,
        ),
        Game,
    >(triangle_mesh_ids_entity)((&mut render_world, &render_channel))
    .unwrap();

    send_clone_query::<
        (
            &demos::phosphor::LineMeshIds,
            &demos::phosphor::LineMeshIdsComponent,
        ),
        Game,
    >(line_mesh_ids_entity)((&mut render_world, &render_channel))
    .unwrap();

    // Clone WGPU backend components to game thread
    send_clone_query::<
        (
            &InstanceComponent,
            &AdapterComponent,
            &DeviceComponent,
            &QueueComponent,
        ),
        Game,
    >(wgpu_backend_entity)((&mut render_world, &render_channel))
    .unwrap();

    // Spawn filesystem and game threads
    spawn_world::<Filesystem, _, _>(fs_thread(fs_world, fs_channel));
    spawn_world::<Game, _, _>(game_thread(game_world, game_channel));

    // Assemble phosphor renderer
    demos::phosphor::assemble(&mut render_world, &render_channel);

    // Enter winit event loop
    winit::event_loop::EventLoop::new().run(antigen_winit::wrap_event_loop(
        render_world,
        render_channel,
        antigen_winit::winit_event_handler(antigen_wgpu::winit_event_handler(
            demos::phosphor::winit_event_handler(render_thread(
                antigen_winit::winit_event_terminator(),
            )),
        )),
    ));
}

/// Spawn a thread with a world and function entrypoint
fn spawn_world<U, F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    std::thread::Builder::new()
        .name(std::any::type_name::<U>().into())
        .spawn(f)
        .unwrap()
}

/// Runs `f` at  `duration` intervals using a spin-lock for timing
fn spin_loop<F: FnMut()>(duration: Duration, mut f: F) -> ! {
    let mut ts = Instant::now();
    loop {
        f();
        while Instant::now().duration_since(ts) < duration {
            std::hint::spin_loop();
        }
        ts = Instant::now();
    }
}

/// Filesystem thread
fn fs_thread(mut world: World, channel: WorldChannel) -> impl FnMut() {
    move || loop {
        receive_messages(&mut world, &channel).expect("Error receiving message");
    }
}

/// Game thread
fn game_thread(mut world: World, channel: WorldChannel) -> impl FnMut() {
    // Create the physics backend
    world.spawn(physics_backend_builder(nalgebra::Vector3::new(0.0, -98.1, 0.0)).build());

    move || {
        spin_loop(GAME_THREAD_TICK, || {
            try_receive_messages(&mut world, &channel).expect("Error handling message");

            // Preparation systems
            demos::phosphor::assemble_triangle_mesh_instances_system(&mut world);
            demos::phosphor::assemble_line_mesh_instances_system(&mut world);

            antigen_rapier3d::insert_colliders_system(&mut world);
            antigen_rapier3d::insert_rigid_bodies_system(&mut world);

            antigen_core::insert_named_entities_system(&mut world);

            // Entity transform systems
            demos::phosphor::movers_position_system(&mut world);
            demos::phosphor::movers_rotation_system(&mut world);

            // Write component transforms to physics system
            antigen_rapier3d::write_rigid_body_isometries_system(&mut world);

            // Step physics
            antigen_rapier3d::step_physics_system(&mut world);

            // Event output
            demos::phosphor::intersection_event_output_system(&mut world);

            // Intersection event dispatch
            demos::phosphor::event_dispatch_system::<IntersectionEvent>(&mut world);

            // Event transformation
            demos::phosphor::event_transform_system::<IntersectionEvent, MoverEvent, _>(
                &mut world,
                |intersection| {
                    if intersection.intersecting {
                        MoverEvent::Close
                    } else {
                        MoverEvent::Open
                    }
                },
            );

            // Mover event dispatch
            demos::phosphor::event_dispatch_system::<MoverEvent>(&mut world);

            // Event input
            demos::phosphor::movers_event_input_system(&mut world);

            // Event clear
            demos::phosphor::clear_event_input_system::<IntersectionEvent>(&mut world);
            demos::phosphor::clear_event_input_system::<MoverEvent>(&mut world);

            demos::phosphor::clear_event_output_system::<IntersectionEvent>(&mut world);
            demos::phosphor::clear_event_output_system::<MoverEvent>(&mut world);

            antigen_rapier3d::clear_physics_event_collector_system(&mut world);

            // Read physics transforms back into components
            antigen_rapier3d::read_back_rigid_body_isometries_system(&mut world);

            // Copy transform components to triangle mesh instances
            antigen_core::copy_to_system::<TriangleMeshInstance, PositionComponent>(&mut world);
            antigen_core::copy_to_system::<TriangleMeshInstance, RotationComponent>(&mut world);
            antigen_core::copy_to_system::<TriangleMeshInstance, ScaleComponent>(&mut world);

            // Copy transform components to line mesh instances
            antigen_core::copy_to_system::<LineMeshInstance, PositionComponent>(&mut world);
            antigen_core::copy_to_system::<LineMeshInstance, RotationComponent>(&mut world);
            antigen_core::copy_to_system::<LineMeshInstance, ScaleComponent>(&mut world);

            // Write buffers to GPU
            antigen_wgpu::buffer_write_slice_system::<
                demos::phosphor::TriangleMeshInstanceDataComponent,
                _,
            >(&mut world);
            antigen_wgpu::buffer_write_slice_system::<
                demos::phosphor::LineMeshInstanceDataComponent,
                _,
            >(&mut world);
            antigen_wgpu::buffer_write_slice_system::<demos::phosphor::LineInstanceDataComponent, _>(
                &mut world,
            );
            antigen_wgpu::buffer_write_system::<antigen_core::PositionComponent>(&mut world);
            antigen_wgpu::buffer_write_system::<antigen_core::RotationComponent>(&mut world);
            antigen_wgpu::buffer_write_system::<antigen_core::ScaleComponent>(&mut world);
            antigen_wgpu::buffer_write_system::<demos::phosphor::LineMeshIdComponent>(&mut world);
        })
    }
}

/// Render thread
pub fn render_thread<T: Clone>(mut f: impl EventLoopHandler<T>) -> impl EventLoopHandler<T> {
    move |world: &mut World,
          channel: &WorldChannel,
          event: Event<'static, T>,
          event_loop_window_target: &EventLoopWindowTarget<T>,
          control_flow: &mut ControlFlow| {
        try_receive_messages(world, channel).expect("Error handling message");

        match event {
            winit::event::Event::MainEventsCleared => {
                println!("Main events cleared");
            }
            _ => (),
        }

        f(
            world,
            channel,
            event,
            event_loop_window_target,
            control_flow,
        )
    }
}
