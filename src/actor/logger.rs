use std::error::Error;
use std::time::Duration;
use log::*;
use steady_state::*;
use crate::actor::worker::FizzBuzzMessage;

pub async fn run(context: SteadyContext, fizz_buzz: SteadyRx<FizzBuzzMessage>) -> Result<(),Box<dyn Error>> {
    internal_behavior(context.into_monitor([&fizz_buzz], []), fizz_buzz).await
}

async fn internal_behavior<C: SteadyCommander>(mut cmd: C, fizz_buzz: SteadyRx<FizzBuzzMessage>) -> Result<(),Box<dyn Error>> {

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