use std::error::Error;
use std::time::Duration;
use log::*;
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


pub async fn run(context: SteadyContext
                 , heartbeat: SteadyRx<u64> //the type can be any struct or primitive or enum...
                 , generator: SteadyRx<u32>
                 , logger: SteadyTx<FizzBuzzMessage>) -> Result<(),Box<dyn Error>> {
    internal_behavior(context.into_monitor([&heartbeat, &generator], [&logger]), heartbeat, generator, logger).await
}

async fn internal_behavior<C: SteadyCommander>(mut cmd: C
                                               , heartbeat: SteadyRx<u64> //the type can be any struct or primitive or enum...
                                               , generator: SteadyRx<u32>
                                               , logger: SteadyTx<FizzBuzzMessage>) -> Result<(),Box<dyn Error>> {

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

#[cfg(test)]
pub(crate) mod worker_tests {
    use std::time::Duration;
    use steady_state::*;
    use super::*;

    #[async_std::test]
    async fn test_worker() {
        let mut graph = GraphBuilder::for_testing()
            .with_telemetry_metric_features(false) //skip this???
            .build(());

        let (generate_tx, generate_rx) = graph.channel_builder()
            .with_capacity(500)
            .build();

        let (hearthbeat_tx, hearthbeat_rx) = graph.channel_builder()
            .with_capacity(500)
            .build::<u32>();

        let (logger_tx, logger_rx) = graph.channel_builder()
            .with_capacity(500)
            .build::<FizzBuzzMessage>();



        //let state = new_state();
        // graph.actor_builder()
        //     .with_name("UnitTest")
        //     .build_spawn(move |context|
        //         internal_behavior(context, generate_tx.clone(), state.clone())
        //     );

        graph.start(); //startup the graph

        Delay::new(Duration::from_millis(100)).await;

        graph.request_stop(); //our actor has no input so it immediately stops upon this request
        graph.block_until_stopped(Duration::from_secs(1));

        let vec:Vec<u32> = generate_rx.testing_take().await;

        assert_eq!(vec[0], 0, "vec: {:?}", vec);
        assert_eq!(vec[1], 1, "vec: {:?}", vec);
    }
}