use crate::*;
use crate::address::message::*;

pub struct OuterContext<A>
where
    A: Actor,
{
    act: A,
    ctx: Context<A>,
}

impl<A> OuterContext<A>
where
    A: Actor,
{
    pub fn new(act: A, ctx: Context<A>) -> Self
    {
        OuterContext {
            act,
            ctx,
        }
    }

    pub fn addr(&self) -> Addr<A>
    {
        self.ctx.addr()
    } 
}

#[derive(Clone)]
pub enum ContextState
{
    Starting,
    Started,
    Stopping,
    Stopped,
}

pub struct Context<A>
where 
    A: Actor,
{
    state: ContextState,

    handle: TaskWakeHandle,
    incoming_tx: mpsc::Sender<PackedMessage<A>>,
    incoming_rx: mpsc::Receiver<PackedMessage<A>>,
    futures: Vec<Box<Future<Item=(), Error=()> + 'static + Send>>,
    marker: std::marker::PhantomData<A>,

    // TODO: child actors, supervisor strat
}

impl<A> Context<A>
where
    A: Actor
{
    pub fn new() -> Self 
    {
        let (tx, rx) = mpsc::channel();
        Context {
            state: ContextState::Starting,
            handle: TaskWakeHandle::empty(),
            incoming_tx: tx,
            incoming_rx: rx,
            futures: Vec::new(),
            marker: std::marker::PhantomData,
        }
    }

    pub fn stop(&mut self)
    {
        self.handle.notify();
        self.state = ContextState::Stopping;
    }

    pub fn terminate(&mut self)
    {
        // TODO: state
    }

    pub fn addr(&self) -> Addr<A>
    {
        Addr::new(self.handle.clone(), self.incoming_tx.clone())
    }

    pub fn state(&self) -> ContextState
    {
        self.state.clone()
    }

    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Item=(), Error=()> + Send + 'static,
    {
        self.futures.push(Box::new(future));
    }

}

impl<A> Future for OuterContext<A>
where 
    A: Actor,
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.ctx.handle.update();

        match self.ctx.state() {
            ContextState::Starting => {
                {
                    let guard = self.ctx.handle.notify();

                    self.act.on_spawn(&mut self.ctx);
                    self.ctx.state = ContextState::Started;
                }

                Ok(Async::NotReady)
            },
            ContextState::Stopping => {
                {
                    let guard = self.ctx.handle.notify();

                    self.act.on_stop(&mut self.ctx);
                    self.ctx.state = ContextState::Stopped;
                }

                Ok(Async::NotReady)
            },
            ContextState::Stopped => {
                Ok(Async::Ready(()))
            }
            _ => {
                loop {
                    loop {
                        // technically could pass the actor in here but that might be bad form since the actors should only process messages
                        // thus let the future send a message to self().
                        self.ctx.futures = self.ctx.futures.drain_filter(|future| {
                            match future.poll() {
                                Ok(Async::Ready(_)) => false,
                                Ok(Async::NotReady) => true,
                                Err(_) => false,
                            }
                        }).collect();
                        break;
                    }

                    // TODO: add preemption
                    match self.ctx.incoming_rx.try_recv() {
                        Ok(mut packed_message) => {
                            packed_message.handle(&mut self.act, &mut self.ctx);
                        },
                        _ => break,
                    }
                }

                Ok(Async::NotReady) 
            },
        }
    }
}