use steady_state::*;

/// Persistent counter-state that survives actor restarts.
/// Heartbeat actors maintain timing consistency across failures.
pub(crate) struct HeartbeatState {
    pub(crate) count: u64
}

/// Entry point demonstrating simulation conditional for full graph testing
pub async fn run(actor: SteadyActorShadow
                 , heartbeat_tx: SteadyTx<u64>
                 , state: SteadyState<HeartbeatState>) -> Result<(),Box<dyn Error>> {
    let actor = actor.into_spotlight([], [&heartbeat_tx]);
    if actor.use_internal_behavior {
        internal_behavior(actor, heartbeat_tx, state).await
    } else {
        actor.simulated_behavior(vec!(&heartbeat_tx)).await
    }
}

/// Periodic signal generation with coordinated shutdown capabilities.
/// This pattern enables time-based coordination across multiple actors
/// while maintaining precise timing control and graceful termination.
async fn internal_behavior<A: SteadyActor>(mut actor: A
                                               , heartbeat_tx: SteadyTx<u64>
                                               , state: SteadyState<HeartbeatState> ) -> Result<(),Box<dyn Error>> {
    // Runtime argument access allows dynamic behavior configuration.
    // This enables the same actor code to work across different deployment scenarios
    // without recompilation or environment-specific builds.
    let args = actor.args::<crate::MainArg>().expect("unable to downcast");
    let rate = Duration::from_millis(args.rate_ms);
    let beats = args.beats;

    // lock our state and init if it has not been initialized yet
    // upon panic and restart this same state with no data loss will be restored
    let mut state = state.lock(|| HeartbeatState{ count: 0}).await;
    let mut heartbeat_tx = heartbeat_tx.lock().await;

    // Shutdown coordination with proper channel cleanup signaling.
    while actor.is_running(|| heartbeat_tx.mark_closed()) {
        // Synchronized waiting demonstrates multi-condition coordination.
        // await_for_all! it ensures both timing requirements and channel capacity
        // are satisfied before proceeding, preventing timing drift and overflow.
        await_for_all!(actor.wait_periodic(rate),
                       actor.wait_vacant(&mut heartbeat_tx, 1));

        // since we used actor.wait_vacant() above we know this try will never fail
        assert!(actor.try_send(&mut heartbeat_tx, state.count).is_sent(),"unable to send");//#!#//
        //OR:
        //actor.try_send(&mut heartbeat_tx, state.count).expect("unable to send");

        state.count += 1;
        // Self-terminating behavior allows actors to control the application lifecycle.
        if beats == state.count {
            actor.request_shutdown().await;
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

        // Requires state so we create one here.
        let state = new_state();
        graph.actor_builder()
            .with_name("UnitTest")
            .build(move |context|
                //As always, use the internal behavior for testing
                internal_behavior(context, heartbeat_tx.clone(), state.clone()), SoloAct
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
