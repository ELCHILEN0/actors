use crate::*;

// useful messages to simplify some derivations.

impl Message for () {
    type Response = ();
}

impl<T> Handler<()> for T
where
    T: Actor
{
    fn receive(&mut self, msg: (), ctx: &mut Context<Self>)
    { }    
}

pub struct SenderMessage<M>
where
    M: Message,
{
    sender: Recipient<M::Response>,
    msg: M,
}

impl<M> Message for SenderMessage<M>
where
    M: Message,
{
    type Response = M::Response;
}

impl<M> SenderMessage<M>
where
    M: Message,
    <M as Message>::Response: 'static,
{
    fn new(sender: Recipient<M::Response>, msg: M) -> Self
    {
        SenderMessage {
            sender,
            msg,
        }
    }

    pub fn sender(&self) -> Recipient<M::Response>
    {
        self.sender.clone()
    }
}

pub fn from<M>(sender: Recipient<M::Response>, msg: M) -> SenderMessage<M>
where
    M: Message,
    M::Response: 'static,
{
    SenderMessage::new(sender, msg)
}
