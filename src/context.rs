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