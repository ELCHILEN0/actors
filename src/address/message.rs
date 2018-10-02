use crate::*;

pub struct PackedMessage<A>
{
    inner: Box<MessageProxy<Actor = A> + Send>,
}

pub struct PackedMessageProxy<A, M>
where
    M: Message
{
    msg: Option<M>,
    act: std::marker::PhantomData<A>,
}

pub trait MessageProxy
{
    type Actor: Actor;

    fn handle(&mut self, act: &mut Self::Actor, ctx: &mut Context<Self::Actor>);
}

pub trait ToPackedMessage<A, M>
{
    fn pack(msg: M) -> PackedMessage<A>;
}

impl<A, M> ToPackedMessage<A, M> for Addr<A>
where
    M: Message + Send + 'static,
    A: Actor + Handler<M>
{
    fn pack(msg: M) -> PackedMessage<A>
    {
        PackedMessage {
            inner: Box::new(PackedMessageProxy{ 
                msg: Some(msg),
                act: std::marker::PhantomData,
            }),
        }
    }
}

impl<A, M> MessageProxy for PackedMessageProxy<A, M>
where
    M: Message + Send + 'static,
    A: Actor + Handler<M>,
{
    type Actor = A;

    fn handle(&mut self, act: &mut Self::Actor, ctx: &mut Context<Self::Actor>)
    {
        if let Some(msg) = self.msg.take() {
            act.receive(msg, ctx);
        }
    }
}

impl<A> MessageProxy for PackedMessage<A>
where
    A: Actor,
{
    type Actor = A;

    fn handle(&mut self, act: &mut Self::Actor, ctx: &mut Context<Self::Actor>)
    {
        self.inner.handle(act, ctx)
    }
}