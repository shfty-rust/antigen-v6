use crate::TwoWayChannel;
use crossbeam_channel::{Receiver, RecvError, SendError, Sender, TryRecvError, TrySendError};
use hecs::{Component, DynamicBundle, Entity, Query, World};
use std::any::TypeId;

/// Struct for coordinating cross-thread communication between worlds
#[derive(Default)]
pub struct WorldExchange {
    channels: Vec<(TypeId, WorldChannel)>,
}

impl WorldExchange {
    pub fn create_channel<U: 'static>(&mut self) -> WorldChannel {
        let (cl, cr) = TwoWayChannel::unbounded();
        self.channels.push((TypeId::of::<U>(), WorldChannel(cl)));
        WorldChannel(cr)
    }

    pub fn spawn(self) {
        std::thread::spawn(move || {
            // Build a channel selector
            let mut sel = crossbeam_channel::Select::new();
            for (_, channel) in &self.channels {
                sel.recv(channel.rx());
            }

            loop {
                // Block until a channel is ready
                let oper = sel.select();

                // Retrieve the ready channel
                let index = oper.index();
                let (type_id, channel) = &self.channels[index];

                // Receive from the channel
                if let Ok(mut message) = oper.recv(&channel.rx()) {
                    message.sender = Some(*type_id);
                    let (_, to_channel) = self
                        .channels
                        .iter()
                        .find(|(candidate, _)| *candidate == message.receiver)
                        .unwrap();
                    to_channel.tx().send(message).unwrap();
                }
            }
        });
    }
}

/// Two-way channel of world messages
pub struct WorldChannel(TwoWayChannel<WorldMessage, WorldMessage>);

impl WorldChannel {
    pub fn tx(&self) -> &Sender<WorldMessage> {
        &self.0.tx
    }

    pub fn rx(&self) -> &Receiver<WorldMessage> {
        &self.0.rx
    }

    pub fn send(&self, message: WorldMessage) -> Result<(), SendError<WorldMessage>> {
        self.0.tx.send(message)
    }

    pub fn try_send(&self, message: WorldMessage) -> Result<(), TrySendError<WorldMessage>> {
        self.0.tx.try_send(message)
    }

    pub fn recv(&self) -> Result<WorldMessage, RecvError> {
        self.0.rx.recv()
    }

    pub fn try_recv(&self) -> Result<WorldMessage, TryRecvError> {
        self.0.rx.try_recv()
    }
}

/// Trait for sending a message to a given world while inferring its type via param F
pub trait SendTo<Message, Value> {
    fn send_to<T>(&self, f: Message) -> Result<(), SendError<Value>>
    where
        T: 'static;

    fn try_send_to<T>(&self, f: Message) -> Result<(), TrySendError<Value>>
    where
        T: 'static;
}

impl<Message> SendTo<Message, WorldMessage> for WorldChannel
where
    Message: for<'a, 'b> FnOnce(MessageContext<'a, 'b>) -> MessageResult<'a, 'b> + Send + 'static,
{
    fn send_to<W>(&self, f: Message) -> Result<(), SendError<WorldMessage>>
    where
        W: 'static,
    {
        self.send(WorldMessage::to::<W, _>(f))
    }

    fn try_send_to<W>(&self, f: Message) -> Result<(), TrySendError<WorldMessage>>
    where
        W: 'static,
    {
        self.try_send(WorldMessage::to::<W, _>(f))
    }
}

/// Cross-thread message between worlds
pub struct WorldMessage {
    sender: Option<std::any::TypeId>,
    receiver: std::any::TypeId,
    message: Box<
        dyn for<'a, 'b> FnOnce(MessageContext<'a, 'b>) -> MessageResult<'a, 'b> + Send + 'static,
    >,
}

impl WorldMessage {
    pub fn sender(&self) -> std::any::TypeId {
        self.sender
            .expect("Sender is not available until the message is sent")
    }

    pub fn receiver(&self) -> std::any::TypeId {
        self.receiver
    }

    pub fn message(
        self,
    ) -> Box<dyn for<'a, 'b> FnOnce(MessageContext<'a, 'b>) -> MessageResult<'a, 'b> + Send + 'static>
    {
        self.message
    }
}

impl std::fmt::Debug for WorldMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldMessage")
            .field("from", &self.sender)
            .field("to", &self.receiver)
            .finish()
    }
}

impl WorldMessage {
    /// Construct a message to be sent to world U
    pub fn to<
        U: 'static,
        F: for<'a, 'b> FnOnce(MessageContext<'a, 'b>) -> MessageResult<'a, 'b> + Send + 'static,
    >(
        message: F,
    ) -> Self {
        let receiver = TypeId::of::<U>();
        let message = Box::new(message);
        WorldMessage {
            sender: None,
            receiver,
            message,
        }
    }

    pub fn reply<
        F: for<'a, 'b> FnOnce(MessageContext<'a, 'b>) -> MessageResult<'a, 'b> + Send + 'static,
    >(
        &self,
        message: F,
    ) -> Self {
        let receiver = self.receiver();
        let message = Box::new(message);
        WorldMessage {
            sender: None,
            receiver,
            message,
        }
    }
}

/// References to a world and its exchange channel
pub type MessageContext<'a, 'b> = (&'a mut World, &'b WorldChannel);

/// Result wrapper to enable compositional flow control
pub type MessageResult<'a, 'b> = Result<MessageContext<'a, 'b>, Box<dyn std::error::Error>>;

/// Lift a type into a MessageResult
pub trait Lift {
    type To;
    fn lift(self) -> Self::To;
}

impl<'a, 'b> Lift for MessageContext<'a, 'b> {
    type To = MessageResult<'a, 'b>;

    fn lift(self) -> Self::To {
        Ok(self)
    }
}

/// Returns a function that will spawn `component` into a provided world
fn spawn_bundle<C: DynamicBundle>(
    bundle: C,
) -> impl for<'a, 'b> FnOnce(MessageContext<'a, 'b>) -> MessageResult<'a, 'b> {
    move |mut ctx| {
        let (world, _) = &mut ctx;
        println!(
            "thread {} spawning {}",
            std::thread::current().name().unwrap(),
            std::any::type_name::<C>(),
        );
        world.spawn(bundle);
        Ok(ctx)
    }
}

fn insert_component<C: DynamicBundle>(
    entity: Entity,
    component: C,
) -> impl for<'a, 'b> FnOnce(MessageContext<'a, 'b>) -> MessageResult<'a, 'b> {
    move |mut ctx| {
        let (world, _) = &mut ctx;
        println!(
            "thread {} inserting {} for entity {:?}",
            std::thread::current().name().unwrap(),
            std::any::type_name::<C>(),
            entity,
        );
        world.insert(entity, component).unwrap();
        Ok(ctx)
    }
}

pub trait ClonedBundle {
    type Bundle: DynamicBundle + Send + Sync + 'static;

    fn cloned_bundle(&self) -> Self::Bundle;
}

impl<T1> ClonedBundle for (&T1,)
where
    T1: Clone + Component,
{
    type Bundle = (T1,);

    fn cloned_bundle(&self) -> Self::Bundle {
        (self.0.clone(),)
    }
}

impl<T1, T2> ClonedBundle for (&T1, &T2)
where
    T1: Clone + Component,
    T2: Clone + Component,
{
    type Bundle = (T1, T2);

    fn cloned_bundle(&self) -> Self::Bundle {
        (self.0.clone(), self.1.clone())
    }
}

impl<T1, T2, T3> ClonedBundle for (&T1, &T2, &T3)
where
    T1: Clone + Component,
    T2: Clone + Component,
    T3: Clone + Component,
{
    type Bundle = (T1, T2, T3);

    fn cloned_bundle(&self) -> Self::Bundle {
        (self.0.clone(), self.1.clone(), self.2.clone())
    }
}

impl<T1, T2, T3, T4> ClonedBundle for (&T1, &T2, &T3, &T4)
where
    T1: Clone + Component,
    T2: Clone + Component,
    T3: Clone + Component,
    T4: Clone + Component,
{
    type Bundle = (T1, T2, T3, T4);

    fn cloned_bundle(&self) -> Self::Bundle {
        (
            self.0.clone(),
            self.1.clone(),
            self.2.clone(),
            self.3.clone(),
        )
    }
}

/// Clone component C and send it to world U
pub fn send_clone_query<Q, U>(
    entity: Entity,
) -> impl for<'a, 'b> FnOnce(MessageContext<'a, 'b>) -> MessageResult<'a, 'b>
where
    Q: Query,
    for<'q> <<Q as Query>::Fetch as hecs::Fetch<'q>>::Item: ClonedBundle,
    U: Send + 'static,
{
    move |mut ctx| {
        let (world, channel) = &mut ctx;

        let query_name = std::any::type_name::<Q>();
        let thread_name = std::any::type_name::<U>();

        println!(
            "Thread {} sending cloned {} to thread {}",
            std::thread::current().name().unwrap(),
            query_name,
            thread_name,
        );

        let components = world.query_one_mut::<Q>(entity).unwrap();
        let bundle = components.cloned_bundle();
        drop(components);

        channel
            .send(WorldMessage::to::<U, _>(spawn_bundle(bundle)))
            .unwrap();

        Ok(ctx)
    }
}

/// Clone singleton component C and send it to world U
pub fn send_copy_component<C, U>(
    entity: Entity,
) -> impl for<'a, 'b> FnOnce(MessageContext<'a, 'b>) -> MessageResult<'a, 'b>
where
    C: Component + Copy,
    U: Send + 'static,
{
    move |mut ctx| {
        let (world, channel) = &mut ctx;

        let component_name = std::any::type_name::<C>();
        let thread_name = std::any::type_name::<U>();
        println!(
            "Thread {} sending copied {} to thread {}",
            std::thread::current().name().unwrap(),
            component_name,
            thread_name,
        );

        let mut query = world.query_one::<&C>(entity).unwrap();
        let component = *if let Some(component) = query.get() {
            component
        } else {
            Err(format!("Error: No such {} component", component_name))?
        };

        channel
            .send(WorldMessage::to::<U, _>(spawn_bundle((component,))))
            .unwrap();

        drop(query);

        Ok(ctx)
    }
}

/// Move Send component C from entity with key component T to `entity` in world U
pub fn send_component<C, U, T>(
    key: T,
    entity: Entity,
) -> impl for<'a, 'b> FnOnce(MessageContext<'a, 'b>) -> MessageResult<'a, 'b>
where
    C: Component,
    T: Component + PartialEq,
    U: Send + 'static,
{
    move |mut ctx| {
        let (world, channel) = &mut ctx;

        let component_name = std::any::type_name::<C>();
        let thread_name = std::any::type_name::<U>();

        let ids = world
            .query_mut::<(&T, &C)>()
            .into_iter()
            .filter(|(_, (k, _))| **k == key)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();

        for id in ids {
            let value = world.remove::<(C,)>(id).unwrap();

            println!(
                "Thread {} sending {} to thread {}",
                std::thread::current().name().unwrap(),
                component_name,
                thread_name,
            );

            channel
                .send(WorldMessage::to::<U, _>(insert_component(entity, value)))
                .unwrap();
        }

        Ok(ctx)
    }
}

/// Receive any pending messages from `channel` and handle them
pub fn try_receive_messages(
    world: &mut World,
    channel: &WorldChannel,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Ok(message) = channel.try_recv() {
        (message.message())((world, channel))?;
    }
    Ok(())
}

/// Block until a message is received from `channel` and handle it
pub fn receive_messages(
    world: &mut World,
    channel: &WorldChannel,
) -> Result<(), Box<dyn std::error::Error>> {
    let message = channel.recv().unwrap();
    (message.message())((world, channel))?;
    Ok(())
}
