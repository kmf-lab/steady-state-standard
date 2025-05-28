use steady_state::*;

/// Persistent counter state that survives actor restarts.
/// Heartbeat actors often need to maintain timing consistency across failures,
/// making persistent state essential for reliable scheduling.
pub(crate) struct HeartbeatState {
    pub(crate) count: u64
}

/// Entry point demonstrating command-line argument integration.
/// Heartbeat actors commonly need runtime configuration for timing parameters,
/// deployment flexibility, and operational tuning.
pub async fn run(context: SteadyContext, heartbeat_tx: SteadyTx<u64>, state: SteadyState<HeartbeatState>) -> Result<(),Box<dyn Error>> {
    let cmd = context.into_monitor([], [&heartbeat_tx]);
    if cmd.use_internal_behavior {
        internal_behavior(cmd, heartbeat_tx, state).await
    } else {
        cmd.simulated_behavior(vec!(&heartbeat_tx)).await
    }
}

/// Periodic signal generation with coordinated shutdown capabilities.
/// This pattern enables time-based coordination across multiple actors
/// while maintaining precise timing control and graceful termination.
async fn internal_behavior<C: SteadyCommander>(mut cmd: C
                                               , heartbeat_tx: SteadyTx<u64>
                                               , state: SteadyState<HeartbeatState> ) -> Result<(),Box<dyn Error>> {
    // Runtime argument access allows dynamic behavior configuration.
    // This enables the same actor code to work across different deployment scenarios
    // without recompilation or environment-specific builds.
    let args = cmd.args::<crate::MainArg>().expect("unable to downcast");
    let rate = Duration::from_millis(args.rate_ms);
    let beats = args.beats;

    let mut state = state.lock(|| HeartbeatState{ count: 0}).await;
    let mut heartbeat_tx = heartbeat_tx.lock().await;

    // Shutdown coordination with proper channel cleanup signaling.
    while cmd.is_running(|| heartbeat_tx.mark_closed()) {
        // Synchronized waiting demonstrates multi-condition coordination.
        // await_for_all! ensures both timing requirements and channel capacity
        // are satisfied before proceeding, preventing timing drift and overflow.
        await_for_all!(cmd.wait_periodic(rate),
                       cmd.wait_vacant(&mut heartbeat_tx, 1));

        let _ = cmd.try_send(&mut heartbeat_tx, state.count);

        state.count += 1;
        // Self-terminating behavior allows actors to control application lifecycle.
        // This pattern is useful for batch jobs, scheduled tasks, or demo applications
        // that need to terminate after completing their work.
        if beats == state.count {
            cmd.request_shutdown().await;
        }
    }
    Ok(())
}

/// Testing with timing validation demonstrates how to verify periodic behavior.
/// This pattern ensures heartbeat actors maintain correct timing characteristics
/// under various load and configuration conditions.
#[cfg(test)]
pub(crate) mod heartbeat_tests {
    use steady_state::*;
    use crate::arg::MainArg;
    use super::*;

    #[test]
    fn test_heartbeat() -> Result<(), Box<dyn Error>> {
        let mut graph = GraphBuilder::for_testing().build(MainArg::default());
        let (heartbeat_tx, heartbeat_rx) = graph.channel_builder().build();

        let state = new_state();
        graph.actor_builder()
            .with_name("UnitTest")
            .build_spawn(move |context|
                internal_behavior(context, heartbeat_tx.clone(), state.clone())
            );

        graph.start();
        // Timing-based testing requires careful coordination between test duration
        // and expected actor behavior to ensure deterministic results.
        std::thread::sleep(Duration::from_millis(1000 * 3));
        graph.request_shutdown();
        graph.block_until_stopped(Duration::from_secs(1))?;
        assert_steady_rx_eq_take!(&heartbeat_rx, vec!(0,1));
        Ok(())
    }
}
