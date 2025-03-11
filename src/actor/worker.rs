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
    internal_behavior(context.into_monitor([], [])).await
}

async fn internal_behavior<C: SteadyCommander>(mut cmd: C) -> Result<(),Box<dyn Error>> {

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