use crate::*;
use crate::address::message::*;

pub struct OuterContext<A>
where
    A: Actor,
{
    act: A,
    ctx: Context<A>,

    scheduled: Vec<Box<Future<Item=(), Error=()> + 'static + Send>>,
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
            scheduled: Vec::new(),
        }
    }

    pub fn addr(&self) -> Addr<A>
    {
        self.ctx.addr()
    } 
}

#[derive(Clone, Debug)]
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
    queued: Vec<Box<Future<Item=(), Error=()> + 'static + Send>>,
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
            queued: Vec::new(),
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
        self.queued.push(Box::new(future));
    }

}

const MESSAGE_QUANTUM: usize = 2;

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
                        // enable logical separation of the context from the futures
                        self.scheduled.reserve_exact(self.ctx.queued.len());
                        self.scheduled.append(&mut self.ctx.queued);

                        let act = &mut self.act;
                        let ctx = &mut self.ctx;
                        self.scheduled = self.scheduled.drain_filter(|future| {
                            match future.poll() {
                                Ok(Async::Ready(_)) => false,
                                Ok(Async::NotReady) => true,
                                Err(_) => false,
                            }
                        }).collect();
                        break;
                    }

                    // allow a fixed number of messages before yielding to other actors
                    let mut remaining_messages = MESSAGE_QUANTUM;
                    match self.ctx.incoming_rx.try_recv() {
                        Ok(mut packed_message) => {
                            packed_message.handle(&mut self.act, &mut self.ctx);

                            remaining_messages -= 1;
                            if remaining_messages == 0 {
                                let guard = self.ctx.handle.notify();
                                break;
                            }
                        },
                        _ => break,
                    }
                }

                Ok(Async::NotReady) 
            },
        }
    }
}