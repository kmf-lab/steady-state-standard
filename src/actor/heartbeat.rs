use std::error::Error;
use std::time::Duration;
use log::info;
use log::error;
use steady_state::*;

/// by keeping the count in steady state this will not be lost or reset if this actor should panic
pub(crate) struct HeartbeatState {
    pub(crate) count: u64
}

/// this is the normal entry point for our actor in the graph using its normal implementation
#[cfg(not(test))]
pub async fn run(context: SteadyContext, heartbeat_tx: SteadyTx<u64>, state: SteadyState<HeartbeatState>) -> Result<(),Box<dyn Error>> {
    internal_behavior(context.into_monitor([], [& heartbeat_tx]), heartbeat_tx, state).await
}

/// this is the test entry point so graph testing can inject values rather than use the normal implementation
#[cfg(test)]
pub async fn run(context: SteadyContext, heartbeat_tx: SteadyTx<u64>, state: SteadyState<HeartbeatState>) -> Result<(),Box<dyn Error>> {
    let mut cmd =  context.into_monitor([], [& heartbeat_tx]);
    if let Some(responder) = cmd.sidechannel_responder() {
        let mut heartbeat_tx = heartbeat_tx.lock().await;
        while cmd.is_running(&mut ||heartbeat_tx.mark_closed()) {
            // in main use graph.sidechannel_director node_call(msg,"heartbeat")
            if !responder.echo_responder(&mut cmd,&mut heartbeat_tx).await {
                //this failure should not happen
                log.error("Unable to send simulated heartbeat for graph test");
            }
        }
    }
    Ok(())
}

async fn internal_behavior<C: SteadyCommander>(mut cmd: C, heartbeat_tx: SteadyTx<u64>, state: SteadyState<HeartbeatState> ) -> Result<(),Box<dyn Error>> {

    let args = cmd.args::<crate::MainArg>().expect("unable to downcast");
    let rate = Duration::from_millis(args.rate_ms);
    let mut state = cmd.steady_state(state, || HeartbeatState{ count: 0});

    let mut heartbeat_tx = heartbeat_tx.lock().await;
    //loop is_running until shutdown signal then we call the closure which closes our outgoing Tx
    while cmd.is_running(|| heartbeat_tx.mark_closed()) {
        await_for_all!(cmd.wait_periodic(rate),
                       cmd.wait_vacant(&mut heartbeat_tx, 1));

        cmd.try_send(&mut heartbeat_tx, state.count )
            .expect("room to write"); //logic error which will not happen due to wait_avail above.

        state.count += 1;
        if  args.beats == state.count {
            log.info("request graph stop");
            cmd.request_graph_stop();
        }
    }
    Ok(())
}


/// Here we test the internal behavior of this actor
#[cfg(test)]
pub(crate) mod tests {
    use std::time::Duration;
    use steady_state::*;
    use super::*;

    #[async_std::test]
    async fn test_simple_process() {
        let mut graph = GraphBuilder::for_testing()
            .with_telemetry_metric_features(false) //skip this???
            .build(());

        let (heartbeat_tx, heartbeat_rx) = graph.channel_builder()
            .with_capacity(500)
            .build();

        graph.actor_builder()
            .with_name("UnitTest")
            .build_spawn(move |context|
                internal_behavior(context, heartbeat_tx.clone())
            );

        graph.start(); //startup the graph

        Delay::new(Duration::from_millis(1000 * 3)).await; //this is the default from args * 3

        graph.request_stop(); //our actor has no input so it immediately stops upon this request
        graph.block_until_stopped(Duration::from_secs(1));

        let vec = heartbeat_rx.testing_take().await;

        assert_eq!(vec[0].value, 0, "vec: {:?}", vec);
        assert_eq!(vec[1].value, 1, "vec: {:?}", vec);
    }
}

