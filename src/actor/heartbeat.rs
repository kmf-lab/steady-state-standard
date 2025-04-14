use steady_state::*;

/// by keeping the count in steady state this will not be lost or reset if this actor should panic
pub(crate) struct HeartbeatState {
    pub(crate) count: u64
}

/// this is the normal entry point for our actor in the graph using its normal implementation
pub async fn run(context: SteadyContext, heartbeat_tx: SteadyTx<u64>, state: SteadyState<HeartbeatState>) -> Result<(),Box<dyn Error>> {
    let cmd = context.into_monitor([], [&heartbeat_tx]);
    if cmd.use_internal_behavior {
        internal_behavior(cmd, heartbeat_tx, state).await
    } else {
        cmd.simulated_behavior(vec!(&TestEcho(heartbeat_tx))).await
    }
}

async fn internal_behavior<C: SteadyCommander>(mut cmd: C
                                               , heartbeat_tx: SteadyTx<u64>
                                               , state: SteadyState<HeartbeatState> ) -> Result<(),Box<dyn Error>> {
    let args = cmd.args::<crate::MainArg>().expect("unable to downcast");
    let rate = Duration::from_millis(args.rate_ms);
    let beats = args.beats;
 //   drop(args); //could be done this way
    let mut state = state.lock(|| HeartbeatState{ count: 0}).await;
    let mut heartbeat_tx = heartbeat_tx.lock().await;
    //loop is_running until shutdown signal then we call the closure which closes our outgoing Tx
    while cmd.is_running(|| heartbeat_tx.mark_closed()) {
        //await here until both of these are true
        await_for_all!(cmd.wait_periodic(rate),
                       cmd.wait_vacant(&mut heartbeat_tx, 1));

        let _ = cmd.try_send(&mut heartbeat_tx, state.count);

        state.count += 1;
        if beats == state.count {
            info!("request graph stop");
            cmd.request_graph_stop();
        }
    }
    Ok(())
}

/// Here we test the internal behavior of this actor
#[cfg(test)]
pub(crate) mod tests {
    pub use std::thread::sleep;
    use steady_state::*;
    use crate::arg::MainArg;
    use super::*;

    #[test]
    fn test_heartbeat() {
        let mut graph = GraphBuilder::for_testing().build(MainArg {
            rate_ms: 0,
            beats: 0,
        });
        //default capacity is 64 unless specified
        let (heartbeat_tx, heartbeat_rx) = graph.channel_builder().build();

        let state = new_state();
        graph.actor_builder()
            .with_name("UnitTest")
            .build_spawn(move |context|
                   internal_behavior(context, heartbeat_tx.clone(), state.clone())
            );

        graph.start(); //startup the graph
        sleep(Duration::from_millis(1000 * 3)); //this is the default from args * 3
        graph.request_stop(); //our actor has no input so it immediately stops upon this request
        graph.block_until_stopped(Duration::from_secs(1));
        assert_steady_rx_eq_take!(&heartbeat_rx, vec!(0,1));
    }
}
