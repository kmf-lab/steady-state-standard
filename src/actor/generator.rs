use std::error::Error;
use std::time::Duration;
use log::*;
use steady_state::*;

pub async fn run(context: SteadyContext, generated: SteadyTx<u32>) -> Result<(),Box<dyn Error>> {
    internal_behavior(context.into_monitor([], [&generated]), generated).await
}

async fn internal_behavior<C: SteadyCommander>(mut cmd: C, generated: SteadyTx<u32> ) -> Result<(),Box<dyn Error>> {

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