use steady_state::*;

/// State structure that persists across actor restarts and panics.
/// Unlike local variables, SteadyState survives actor failures and maintains
/// consistency across the entire application lifecycle.
pub(crate) struct GeneratorState {
    pub(crate) value: u64
}

/// Public entry point that demonstrates dual-mode operation pattern.
/// This allows the same actor to run in production mode (internal_behavior)
/// or testing mode (simulated_behavior) based on the execution context.
pub async fn run(context: SteadyContext, generated_tx: SteadyTx<u64>, state: SteadyState<GeneratorState>) -> Result<(),Box<dyn Error>> {
    let cmd = context.into_monitor([], [&generated_tx]);
    if cmd.use_internal_behavior {
        internal_behavior(cmd, generated_tx, state).await
    } else {
        cmd.simulated_behavior(vec!(&generated_tx)).await
    }
}

/// Internal behavior demonstrates continuous data production with backpressure handling.
/// This pattern is common for data sources that need to produce at maximum safe rate
/// while respecting downstream capacity constraints.
async fn internal_behavior<C: SteadyCommander>(mut cmd: C, generated: SteadyTx<u64>, state: SteadyState<GeneratorState> ) -> Result<(),Box<dyn Error>> {

    // State locking provides thread-safe access with automatic initialization.
    // The closure runs only if no state exists, ensuring consistent startup behavior.
    let mut state = state.lock(|| GeneratorState {value: 0}).await;
    let mut generated = generated.lock().await;

    // Shutdown coordination: mark_closed() signals downstream actors that no more data will come.
    // This enables clean pipeline termination without dropping messages in transit.
    while cmd.is_running(|| generated.mark_closed()) {
        // SendSaturation::AwaitForRoom provides automatic backpressure management.
        // The actor will pause here if the receiving channel is full, preventing memory exhaustion
        // while maintaining data ordering and system stability.
        match cmd.send_async(&mut generated, state.value, SendSaturation::AwaitForRoom).await {
            SendOutcome::Success => state.value += 1,
            SendOutcome::Blocked(_value) => {} // Graceful handling of shutdown scenarios
        };
    }
    Ok(())
}

/// Unit test demonstrates isolated actor testing without requiring a full graph.
/// This pattern enables rapid development cycles and precise behavioral verification.
#[cfg(test)]
pub(crate) mod generator_tests {
    use steady_state::*;
    use crate::arg::MainArg;
    use super::*;

    #[test]
    fn test_generator() -> Result<(), Box<dyn Error>> {
        let mut graph = GraphBuilder::for_testing().build(MainArg::default());
        let (generate_tx, generate_rx) = graph.channel_builder().build();

        let state = new_state();
        graph.actor_builder()
            .with_name("UnitTest")
            .build_spawn(move |context| internal_behavior(context, generate_tx.clone(), state.clone()) );

        graph.start();
        // Timing-based testing requires careful coordination between test duration
        // and expected actor behavior to ensure deterministic results.
        std::thread::sleep(Duration::from_millis(100));
        graph.request_shutdown();

        graph.block_until_stopped(Duration::from_secs(1))?;

        // Deterministic testing: even in multi-threaded environments,
        // actor isolation ensures predictable message sequences.
        assert_steady_rx_eq_take!(generate_rx,vec!(0,1));
        Ok(())
    }
}
