use steady_state::*;
use crate::actor::worker::FizzBuzzMessage;

/// Simple consumer actor demonstrating reactive message processing.
/// Logger actors typically have no outgoing channels and focus on
/// efficient message consumption and external system integration.
pub async fn run(actor: SteadyActorShadow, fizz_buzz_rx: SteadyRx<FizzBuzzMessage>) -> Result<(),Box<dyn Error>> {
    let actor = actor.into_spotlight([&fizz_buzz_rx], []);
    if actor.use_internal_behavior {
        internal_behavior(actor, fizz_buzz_rx).await
    } else { //as with other edge actors, we use simulated behavior to enable testing from main
        actor.simulated_behavior(vec!(&fizz_buzz_rx)).await
    }
}

/// Event-driven processing pattern for immediate message handling.
/// This approach ensures minimal latency between message arrival and processing,
/// making it ideal for logging, monitoring, and real-time notification systems.
async fn internal_behavior<A: SteadyActor>(mut actor: A
                                           , rx: SteadyRx<FizzBuzzMessage>) -> Result<(),Box<dyn Error>> {
    let mut rx = rx.lock().await;
    // Termination condition waits for channel closure and message drainage.
    // This ensures all messages are processed before the actor terminates,
    // preventing data loss during shutdown sequences.
    while actor.is_running(|| rx.is_closed_and_empty()) {
        // This is important as it drops CPU usage to zero if we have no work to do.
        await_for_all!(actor.wait_avail(&mut rx, 1)); //#!#//
        
        // This consumes all the messages in the channel until it is empty
        // Warning: the producer is adding messages at the same time;
        // so we may be here longer than we want. NOTE: is_running() checks
        // for shutdown and relays collected telemetry.
        while let Some(msg) = actor.try_take(&mut rx) { //#!#//
            // Message processing with structured logging integration.
            // The framework automatically handles log formatting, threading,
            // and output routing based on configuration. 
            info!("Msg {:?}", msg );
        }        
    }
    Ok(())
}

/// Testing with log capture demonstrates verification of actor output behavior.
/// This pattern enables testing of actors that primarily produce side effects
/// rather than direct message outputs.
#[test]
fn test_logger() -> Result<(), Box<dyn std::error::Error>> {
    use steady_logger::*;
    
    //in this case, there is no outgoing channel, so we must test against the logs
    let _guard = start_log_capture();  //#!#//

    let mut graph = GraphBuilder::for_testing().build(());
    let (fizz_buzz_tx, fizz_buzz_rx) = graph.channel_builder().build();

    graph.actor_builder().with_name("UnitTest")
        .build(move |context| {
            internal_behavior(context, fizz_buzz_rx.clone())
        }, SoloAct);

    graph.start();
    // Testing infrastructure provides message injection capabilities
    // for precise control over actor input during verification.
    fizz_buzz_tx.testing_send_all(vec![FizzBuzzMessage::Fizz],true);

    graph.request_shutdown();
    graph.block_until_stopped(Duration::from_secs(10000))?;
    // Log assertion macros enable verification of logging behavior
    // across multi-threaded execution environments.
    assert_in_logs!(["Msg Fizz"]); //#!#//

    Ok(())
}
