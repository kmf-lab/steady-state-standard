use std::error::Error;
use std::time::Duration;
use log::*;
use steady_state::*;
use steady_state::simulate_edge::{external_behavior, Simulate};
use crate::actor::heartbeat::HeartbeatState;
use crate::actor::simulate_external_behavior::{external_behavior, Simulate};

pub(crate) struct GeneratorState {
    pub(crate) value: u32
}

#[cfg(not(test))]
pub async fn run(context: SteadyContext, generated_tx: SteadyTx<u32>, state: SteadyState<GeneratorState>) -> Result<(),Box<dyn Error>> {
    internal_behavior(context.into_monitor([], [&generated_tx]), generated_tx, state).await
}

#[cfg(test)]
pub async fn run(context: SteadyContext, generated_tx: SteadyTx<u64>, state: SteadyState<HeartbeatState>) -> Result<(),Box<dyn Error>> {
    external_behavior(context, Simulate::Echo(context.into_monitor([], [&generated_tx]), generated_tx)).await
}

async fn internal_behavior<C: SteadyCommander>(mut cmd: C, generated: SteadyTx<u32>, state: SteadyState<GeneratorState> ) -> Result<(),Box<dyn Error>> {

    let mut state = state.lock(|| GeneratorState {value: 0}).await;
    if let Some(state) = state.as_mut() {
        let mut generated = generated.lock().await;
        while cmd.is_running(|| generated.mark_closed()) {
             //this will await until we have room for this one.
             let _ = cmd.send_async(&mut generated, state.value, SendSaturation::IgnoreAndWait).await;
             state.value += 1;
        }
    }
    Ok(())
}

/// Here we test the internal behavior of this actor
#[cfg(test)]
pub(crate) mod generator_tests {
    use std::time::Duration;
    use steady_state::*;
    use super::*;

    #[async_std::test]
    async fn test_generator() {
        let mut graph = GraphBuilder::for_testing()
            .with_telemetry_metric_features(false) //skip this???
            .build(());

        let (generate_tx, generate_rx) = graph.channel_builder()
            .with_capacity(500)
            .build();

        let state = new_state();
        graph.actor_builder()
            .with_name("UnitTest")
            .build_spawn(move |context|
                internal_behavior(context, generate_tx.clone(), state.clone())
            );

        graph.start(); //startup the graph

        Delay::new(Duration::from_millis(100)).await;

        graph.request_stop(); //our actor has no input so it immediately stops upon this request
        graph.block_until_stopped(Duration::from_secs(1));

        let vec = generate_rx.testing_take().await;

        assert_eq!(vec[0], 0, "vec: {:?}", vec);
        assert_eq!(vec[1], 1, "vec: {:?}", vec);
    }
}