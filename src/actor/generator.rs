use steady_state::*;

pub(crate) struct GeneratorState {
    pub(crate) value: u64
}

pub async fn run(context: SteadyContext, generated_tx: SteadyTx<u64>, state: SteadyState<GeneratorState>) -> Result<(),Box<dyn Error>> {
    let cmd = context.into_monitor([], [&generated_tx]);
    if cmd.use_internal_behavior {
        internal_behavior(cmd, generated_tx, state).await
    } else {
        cmd.simulated_behavior(vec!(&TestEcho(generated_tx))).await
    }
}

async fn internal_behavior<C: SteadyCommander>(mut cmd: C, generated: SteadyTx<u64>, state: SteadyState<GeneratorState> ) -> Result<(),Box<dyn Error>> {

    let mut state = state.lock(|| GeneratorState {value: 0}).await;
    let mut generated = generated.lock().await;

    while cmd.is_running(|| generated.mark_closed()) {
         //this will await until we have room for this one.
         let _ = cmd.send_async(&mut generated, state.value, SendSaturation::IgnoreAndWait).await;
         state.value += 1;
    }
    Ok(())
}

/// Here we test the internal behavior of this actor
#[cfg(test)]
pub(crate) mod generator_tests {
    use std::thread::sleep;
    use steady_state::*;
    use super::*;

    #[test]
    fn test_generator() {
        let mut graph = GraphBuilder::for_testing().build(());
        let (generate_tx, generate_rx) = graph.channel_builder().build();

        let state = new_state();
        graph.actor_builder()
            .with_name("UnitTest")
            .build_spawn(move |context| internal_behavior(context, generate_tx.clone(), state.clone()) );

        graph.start();
        sleep(Duration::from_millis(100));
        graph.request_stop();

        graph.block_until_stopped(Duration::from_secs(1));

        assert_steady_rx_eq_take!(generate_rx,vec!(0,1));
    }
}