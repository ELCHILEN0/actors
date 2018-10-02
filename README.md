# actors
Typed actor library with support for distributed actors

# Example

```rust
struct TestActor;
struct ValidMessageA;

#[derive(Serialize, Deserialize)]
struct ValidMessageB(usize);
struct InvalidMessage;

impl Message for ValidMessageA {
    type Response = ValidMessageB;
}

impl Message for ValidMessageB { 
    type Response = ();
}

impl Actor for TestActor {
    fn on_spawn(&mut self, ctx: &mut Context<Self>)
    {

        let my_addr = ctx.addr();
        // my_addr.send(from(my_addr.recipient(), ValidMessageA));
        ctx.spawn(future::lazy(move || {
            my_addr.send(from(my_addr.recipient(), ValidMessageA));
            Ok(())
        }));
    }
}

impl Handler<SenderMessage<ValidMessageA>> for TestActor
{
    fn receive(&mut self, msg: SenderMessage<ValidMessageA>, ctx: &mut Context<Self>)
    {
        msg.sender().send(ValidMessageB(2));
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
        let addr1 = TestActor{ }.spawn();
        let addr2 = TestActor{ }.spawn();

        let addr1listener = ListenerActor::<ValidMessageB>::new(
            "127.0.0.1:8080".to_string(),
            addr1.recipient(),
        ).spawn();
        let addr2listener = ListenerActor::<ValidMessageB>::new(
            "127.0.0.1:8081".to_string(),
            addr2.recipient(),
        ).spawn();

        let send1 = ConnectorActor::<ValidMessageB>::new(
            "127.0.0.1:8080".to_string(),
            addr2.recipient(),
        ).spawn();

        let send2 = ConnectorActor::<ValidMessageB>::new(
            "127.0.0.1:8081".to_string(),
            addr1.recipient(),
        ).spawn();

        send2.send(remote(ValidMessageB(1)));
        send2.send(remote(ValidMessageB(1)));
        send2.send(remote(ValidMessageB(1)));
        
        Ok(())
    });
}
```
