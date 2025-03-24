use std::error::Error;
use std::time::Duration;
use log::*;
use steady_state::*;
use crate::actor::worker::FizzBuzzMessage;


pub async fn run(context: SteadyContext, fizz_buzz_rx: SteadyRx<FizzBuzzMessage>) -> Result<(),Box<dyn Error>> {
    let cmd = context.into_monitor([&fizz_buzz_rx], []);
    if cfg!(not(test)) {
        internal_behavior(cmd, fizz_buzz_rx).await
    } else {
        cmd.simulated_behavior(vec!(&TestEquals(fizz_buzz_rx))).await
    }
}


async fn internal_behavior<C: SteadyCommander>(mut cmd: C, fizz_buzz: SteadyRx<FizzBuzzMessage>) -> Result<(),Box<dyn Error>> {
    let args = cmd.args::<crate::MainArg>().expect("unable to downcast");
    let rate = Duration::from_millis(args.rate_ms);

    let mut count = args.beats;
    while cmd.is_running(|| true) {
        await_for_all!(cmd.wait_periodic(rate));
        info!("Heartbeat {} {:?}", count, rate );
        count -= 1;
        if  count == 0 {
            cmd.request_graph_stop();
        }
    }
    Ok(())
}

#[async_std::test]
async fn test_logger() {

    let mut graph = GraphBuilder::for_testing().build(());

    let (fizz_buzz_tx, fizz_buzz_rx) = graph.channel_builder()
        .with_capacity(500) // default this?
        .build();

    graph.actor_builder()
        .with_name("UnitTest")
        .build(move |context|
            internal_behavior(context, fizz_buzz_rx.clone())
        , &mut Threading::Spawn);

    graph.start(); //startup the grap

    let _ = fizz_buzz_tx.testing_send_all(vec![FizzBuzzMessage::Fizz],true);

    Delay::new(Duration::from_millis(300)).await;

    graph.request_stop(); //our actor has no input so it immediately stops upon this request
    graph.block_until_stopped(Duration::from_secs(1));

    //TODO: fizz_buzz_tx.testing_is_empty();
    // TODO: logger test??

 }
