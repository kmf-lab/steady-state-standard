
use steady_state::*;

// over designed this enum is. much to learn here we have.
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
    pub fn new(value: u64) -> Self {
        match (value % 3, value % 5) {
            (0, 0) => FizzBuzzMessage::FizzBuzz,    // Multiple of 15
            (0, _) => FizzBuzzMessage::Fizz,        // Multiple of 3, not 5
            (_, 0) => FizzBuzzMessage::Buzz,        // Multiple of 5, not 3
            _      => FizzBuzzMessage::Value(value), // Neither
        }
    }
}

pub async fn run(context: SteadyContext
                 , heartbeat: SteadyRx<u64> //the type can be any struct or primitive or enum...
                 , generator: SteadyRx<u64>
                 , logger: SteadyTx<FizzBuzzMessage>) -> Result<(),Box<dyn Error>> {
    internal_behavior(context.into_monitor([&heartbeat, &generator], [&logger]), heartbeat, generator, logger).await
}

async fn internal_behavior<C: SteadyCommander>(mut cmd: C
                                               , heartbeat: SteadyRx<u64> //the type can be any struct or primitive or enum...
                                               , generator: SteadyRx<u64>
                                               , logger: SteadyTx<FizzBuzzMessage>) -> Result<(),Box<dyn Error>> {

    let mut heartbeat = heartbeat.lock().await;
    let mut generator = generator.lock().await;
    let mut logger = logger.lock().await;

    while cmd.is_running(|| true) {
        let _clean = await_for_all!(cmd.wait_avail(&mut heartbeat,1)
                                  , cmd.wait_avail(&mut generator,1));

        if let Some(h) = cmd.try_take(&mut heartbeat) {
            //for each beat we empty the generated data
            for item in cmd.take_into_iter(&mut generator) {
                //note: SendSaturation tells the async call to just wait if the outgoing channel
                //      is full. Another popular choice is Warn so it logs if it gets filled.
                cmd.send_async(&mut logger, FizzBuzzMessage::new(item)
                                          , SendSaturation::IgnoreAndWait).await;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod worker_tests {
    use std::thread::sleep;
    use steady_state::*;
    use super::*;

    #[test]
    fn test_worker() {
        let mut graph = GraphBuilder::for_testing().build(());
        let (generate_tx, generate_rx) = graph.channel_builder().build();
        let (heartbeat_tx, heartbeat_rx) = graph.channel_builder().build();
        let (logger_tx, logger_rx) = graph.channel_builder().build::<FizzBuzzMessage>();

        graph.actor_builder().with_name("UnitTest")
             .build(move |context| internal_behavior(context
                                                             , heartbeat_rx.clone()
                                                             , generate_rx.clone()
                                                             , logger_tx.clone())
                 , &mut Threading::Spawn
             );

        generate_tx.testing_send_all(vec![0,1,2,3,4,5], true);
        heartbeat_tx.testing_send_all(vec![0], true);
        graph.start();

        sleep(Duration::from_millis(100));

        graph.request_stop();
        graph.block_until_stopped(Duration::from_secs(1));
        assert_steady_rx_eq_take!(&logger_rx, [FizzBuzzMessage::FizzBuzz
                                              ,FizzBuzzMessage::Value(1)
                                              ,FizzBuzzMessage::Value(2)
                                              ,FizzBuzzMessage::Fizz
                                              ,FizzBuzzMessage::Value(4)
                                              ,FizzBuzzMessage::Buzz]);
    }
}