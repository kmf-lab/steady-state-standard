use steady_state::*;

// over designed this enum is. much to learn here we have.
// Memory-efficient message design using discriminant encoding for compact representation.
// The repr(u64) attribute enables the entire enum to fit within 8 bytes, improving
// cache performance and reducing memory allocation overhead in high-throughput scenarios.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
#[repr(u64)] // Pack everything into 8 bytes
pub(crate) enum FizzBuzzMessage {
    #[default]
    FizzBuzz = 15,         // Discriminant is 15 - could have been any valid FizzBuzz
    Fizz = 3,              // Discriminant is 3 - could have been any valid Fizz
    Buzz = 5,              // Discriminant is 5 - could have been any valid Buzz
    Value(u64),            // Store u64 directly, use the fact that FizzBuzz/Fizz/Buzz only occupy small values
}
impl FizzBuzzMessage {
    /// Business logic encapsulation within message constructors promotes consistency
    /// and keeps domain-specific rules close to the data structures they operate on.
    pub fn new(value: u64) -> Self {
        match (value % 3, value % 5) {
            (0, 0) => FizzBuzzMessage::FizzBuzz,    // Multiple of 15
            (0, _) => FizzBuzzMessage::Fizz,        // Multiple of 3, not 5
            (_, 0) => FizzBuzzMessage::Buzz,        // Multiple of 5, not 3
            _      => FizzBuzzMessage::Value(value), // Neither
        }
    }
}

/// Multi-input coordination actor demonstrating complex data flow patterns.
/// Worker actors commonly integrate multiple data streams with different timing
/// characteristics while maintaining processing order and system responsiveness.
pub async fn run(actor: SteadyActorShadow
                 , heartbeat: SteadyRx<u64> //the type can be any struct or primitive or enum...
                 , generator: SteadyRx<u64>
                 , logger: SteadyTx<FizzBuzzMessage>) -> Result<(),Box<dyn Error>> {
    //this is NOT on the edge of the graph so we do not want to simulate it as it will be tested by its simulated neighbors
    internal_behavior(actor.into_spotlight([&heartbeat, &generator], [&logger]), heartbeat, generator, logger).await
}

/// Batch processing pattern triggered by external timing signals enables efficient
/// bulk operations while maintaining responsive timing control and proper resource
/// utilization across variable load conditions.
async fn internal_behavior<A: SteadyActor>(mut actor: A
                                               , heartbeat: SteadyRx<u64> //the type can be any struct or primitive or enum...
                                               , generator: SteadyRx<u64>
                                               , logger: SteadyTx<FizzBuzzMessage>) -> Result<(),Box<dyn Error>> {

    let mut heartbeat = heartbeat.lock().await;
    let mut generator = generator.lock().await;
    let mut logger = logger.lock().await;

    // When a shutdown is requested, is_running will call the closure to determine if this actor will accept or veto the shutdown.
    // If the closure returns true then the shutdown was accepted, and we will exit the while loop.  It is typical to use
    // short circuit boolean logic to confirm all the required conditions for our actor to shut down. In order to help
    // debug 'why' a actor might refuse to shut down we put 'eyes' the i! macro around each boolean.  The i! macros will simply
    // pass thru the boolean value but also capture and reports which one returned false in the event of an unclean shutdown.

    while actor.is_running(|| i!(heartbeat.is_closed_and_empty()) && i!(generator.is_closed_and_empty()) && i!(logger.mark_closed())) {

        // There are many ways to design an actor, but this is the standard approach to use as the default.
        // Put all the required needs into a single await_for macro call, we have 3 different macros to choose from,
        // and the macros can be nested as needed by using 'wait' editions inside 'await' editions.
        //    ie        await_for_any!(wait_for_all!(...), wait_for_all!(...))
        //
        // await_for_all!:  calls await on every future passed in and then continue after they are all complete.
        // await_for_any!:  calls await on every future passed in and then continue after one of them has completed.
        // await_for_all_or_proceed_upon!: same as await_for_all except that if the first item is done, it immediately continues.
        //
        // reminder: Ihis is all single threaded, as each future makes progress it does so while the other futures await.
        //           In general, all the futures are probably spending most of their time awaiting something external
        //
        // The await_for macros all return a boolean 'clean' which is true if all the conditions were met, this will be
        // false if it had to exit early due to a shutdown in progress.

        let _clean = await_for_all!(actor.wait_avail(&mut heartbeat,1)
                                  , actor.wait_avail(&mut generator,1)
                                  , actor.wait_vacant(&mut logger, 1)
        );

        //if we have a heartbeat or a stop request then we need to process some work
        if actor.try_take(&mut heartbeat).is_some() || actor.is_liveliness_stop_requested() {
            //check for how much work and how much room we have before we begin
            let mut items = actor.avail_units(&mut generator).min(actor.vacant_units(&mut logger));           
            while items>0 {                
                let item = actor.try_take(&mut generator).expect("internal error");
                actor.try_send(&mut logger, FizzBuzzMessage::new(item)).expect("internal error");
                items -= 1;
            }
        }
    }
    Ok(())
}

/// Integration testing demonstrates multi-actor coordination verification across
/// multiple threads and channels, ensuring correct behavior under realistic conditions.
#[cfg(test)]
pub(crate) mod worker_tests {

    use steady_state::*;
    use super::*;

    #[test]
    fn test_worker() -> Result<(), Box<dyn Error>> {
        let mut graph = GraphBuilder::for_testing().build(());
        let (generate_tx, generate_rx) = graph.channel_builder().build();
        let (heartbeat_tx, heartbeat_rx) = graph.channel_builder().build();
        let (logger_tx, logger_rx) = graph.channel_builder().build::<FizzBuzzMessage>();

        graph.actor_builder().with_name("UnitTest")
            .build(move |context| internal_behavior(context
                                                    , heartbeat_rx.clone()
                                                    , generate_rx.clone()
                                                    , logger_tx.clone())
                   , SoloAct
            );

        
        generate_tx.testing_send_all(vec![0,1,2,3,4,5], true);
        heartbeat_tx.testing_send_all(vec![0], true);
        graph.start();
        // because shutdown waits for closed and empty, it does not happen until our test data is digested. 
        graph.request_shutdown();
        graph.block_until_stopped(Duration::from_secs(1))?;
        assert_steady_rx_eq_take!(&logger_rx, [FizzBuzzMessage::FizzBuzz
                                              ,FizzBuzzMessage::Value(1)
                                              ,FizzBuzzMessage::Value(2)
                                              ,FizzBuzzMessage::Fizz
                                              ,FizzBuzzMessage::Value(4)
                                              ,FizzBuzzMessage::Buzz]);
        Ok(())
    }
}
