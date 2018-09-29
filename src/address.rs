use crate::*;
use crate::address::message::*;

pub mod message;

pub trait Sender<M>
where
    M: Message + Send + Sync + 'static,
{
    fn send(&mut self, msg: M);
    fn boxed(&self) -> Box<Sender<M>>;
}

pub struct Addr<A>
where 
    A: Actor
{
    handle: TaskWakeHandle,
    tx: mpsc::Sender<PackedMessage<A>>,
}

impl<A> Clone for Addr<A>
where
    A: Actor,
{
    fn clone(&self) -> Self {
        Addr {
            handle: self.handle.clone(),
            tx: self.tx.clone(),
        }
    }
}

#[derive(Clone)]
pub struct EmptyAddr<M>
where
    M: Message,
{
    marker: std::marker::PhantomData<M>,
}

// ActorAddr<A>
// EmptyAddr<M>
// TypedAddr<M>

pub struct Recipient<M>
where
    M: Message,
{
    sender: Box<Sender<M>>,
}

unsafe impl<M> Send for Recipient<M>
where
    M: Message { }

unsafe impl<M> Sync for Recipient<M>
where
    M: Message { }

impl<M> Clone for Recipient<M>
where
    M: Message + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Recipient {
            sender: self.sender.boxed()
        }
    }
}

impl<A> Addr<A>
where
    A: Actor
{
    pub fn new(handle: TaskWakeHandle, tx: mpsc::Sender<PackedMessage<A>>) -> Self
    {
        Addr {
            handle,
            tx,
        }
    }

    pub fn recipient<M>(&self) -> Recipient<M>
    where
        M: Message + Send + Sync + 'static,
        A: Handler<M>,
    {
        Recipient::from(self)
    }
}

impl<M> EmptyAddr<M>
where
    M: Message + Send + Sync + 'static,
{
    pub fn new() -> Self {
        EmptyAddr {
            marker: std::marker::PhantomData,
        }
    }

    pub fn recipient(&self) -> Recipient<M>
    {
        Recipient::from(self)
    }
}


impl<M> Recipient<M>
where
    M: Message + Send + Sync + 'static,
{
    pub fn from(sender: &Sender<M>) -> Self
    {
        Recipient {
            sender: sender.boxed(),
        }
    }
}

impl<A, M> Sender<M> for Addr<A>
where
    M: Message + Send + Sync + 'static,
    A: Actor + Handler<M>,
{
    fn send(&mut self, msg: M)
    {
        let guard = self.handle.notify();
        self.tx.send(Self::pack(msg));   
    }

    fn boxed(&self) -> Box<Sender<M>>
    {
        Box::new(self.clone())
    }
}

impl<M> Sender<M> for EmptyAddr<M>
where
    M: Message + Send + Sync + 'static,
{
    fn send(&mut self, msg: M)
    { }

    fn boxed(&self) -> Box<Sender<M>>
    {
        Box::new(EmptyAddr::new())
    }
}

impl<M> Sender<M> for Recipient<M>
where
    M: Message + Send + Sync + 'static,
{
    fn send(&mut self, msg: M)
    {
        self.sender.send(msg);
    }

    fn boxed(&self) -> Box<Sender<M>>
    {
        self.sender.boxed()
    }
}    