use tokio::prelude::*;
use actors::*;

struct TestActor;
struct ValidMessageA;
struct ValidMessageB;
struct InvalidMessage;

impl Actor for TestActor {
    fn on_spawn(&mut self, ctx: &mut Context<Self>) {
        println!("test act spawn");
        let mut my_addr = ctx.addr();
        my_addr.send(from(my_addr.recipient(), ValidMessageA));
    }
}

impl Message for ValidMessageA {
    type Response = ValidMessageB;
}

impl Message for ValidMessageB { 
    type Response = ();
}

impl Handler<SenderMessage<ValidMessageA>> for TestActor
{
    fn receive(&mut self, msg: SenderMessage<ValidMessageA>, ctx: &mut Context<Self>)
    {
        // ctx.stop();
        msg.sender().send(ValidMessageB);
        println!("handled A");
    }
}

impl Handler<ValidMessageB> for TestActor
{
    fn receive(&mut self, msg: ValidMessageB, ctx: &mut Context<Self>)
    {
        println!("handled B");
    }
}

fn main() {
    tokio::run(future::lazy(|| {
        let mut addr1 = TestActor{ }.spawn();
        let mut addr2 = TestActor{ }.spawn();

        addr1.send(from(EmptyAddr::new().recipient(), ValidMessageA));
        addr2.send(from(EmptyAddr::new().recipient(), ValidMessageA));

        Ok(())
    }));

    println!("done running");
}