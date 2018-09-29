#![feature(futures_api)]

use tokio::prelude::*;
use tokio::prelude::task::Task;

use std::sync::*;

mod address;
mod context;
mod messages;
mod handle;

pub use crate::address::*;
pub use crate::context::*;
pub use crate::messages::*;
pub use crate::handle::*;

pub trait Actor: Sized + Send + Sync + 'static {
    fn spawn(self) -> Addr<Self>
    {
        let context = OuterContext::new(self, Context::new());

        let addr = context.addr();
        tokio::executor::spawn(context);
        addr
    }

    fn on_spawn(&mut self, ctx: &mut Context<Self>)
    { }

    fn on_stop(&mut self, ctx: &mut Context<Self>)
    { }
}

pub trait Message {
    type Response: Message + Send + Sync;
}

pub trait Handler<M>
where
    Self: Actor,
    M: Message,
{
    // fn sender(&self) -> Recipient<M::Response>
    // {
    //     unimplemented!()
    // }

    fn receive(&mut self, msg: M, ctx: &mut Context<Self>);
}