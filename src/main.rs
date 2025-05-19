use steady_state::*;
use arg::MainArg;
mod arg;

pub(crate) mod actor {
   pub(crate) mod heartbeat;
   pub(crate) mod generator;
   pub(crate) mod worker;
   pub(crate) mod logger;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let cli_args = MainArg::parse();
    let _ = init_logging(LogLevel::Info);
    let mut graph = GraphBuilder::default()
           .build(cli_args); //or pass () if no args

    build_graph(&mut graph);

    //startup entire graph
    graph.start();
    // your graph is running here until actor calls graph stop
    graph.block_until_stopped(std::time::Duration::from_secs(1))
}

fn build_graph(graph: &mut Graph) {
    let channel_builder = graph.channel_builder();

    let (heartbeat_tx, heartbeat_rx) = channel_builder.build();
    let (generator_tx, generator_rx) = channel_builder.build();
    let (worker_tx, worker_rx) = channel_builder.build();

    let actor_builder = graph.actor_builder().with_mcpu_avg();

    let state = new_state();
    actor_builder.with_name("heartbeat")
        .build(move |context| { actor::heartbeat::run(context, heartbeat_tx.clone(), state.clone()) }
               , &mut Threading::Spawn);

    let state = new_state();
    actor_builder.with_name("generator")
        .build(move |context| { actor::generator::run(context, generator_tx.clone(), state.clone()) }
               , &mut Threading::Spawn);

    actor_builder.with_name("worker")
        .build(move |context| { actor::worker::run(context, heartbeat_rx.clone(), generator_rx.clone(), worker_tx.clone()) }
               , &mut Threading::Spawn);

    actor_builder.with_name("logger")
        .build(move |context| { actor::logger::run(context, worker_rx.clone()) }
               , &mut Threading::Spawn);
}


#[cfg(test)]
pub(crate) mod main_tests {
    use steady_state::*;
    use crate::actor::worker::FizzBuzzMessage;
    use super::*;

    #[test]
    fn graph_test() -> Result<(), Box<dyn std::error::Error>> {

        let gb = GraphBuilder::for_testing();

        let mut graph = gb.build(MainArg {
            rate_ms: 100,
            beats: 10,
        });

        build_graph(&mut graph);
        
        graph.start();

        let messenger = graph.sidechannel_messenger();

       messenger.call_actor(Box::new(15u64), ActorName::new("generator", None))?;
       messenger.call_actor(Box::new(100u64), ActorName::new("heartbeat", None))?;
      // // 
      // // //  sleep(std::time::Duration::from_millis(100));
      // //   //TODO: we need a better timeout solution
      messenger.call_actor(Box::new(FizzBuzzMessage::FizzBuzz), ActorName::new("logger",None))?;

        
      drop(messenger); //TODO: combine
      graph.request_stop();


      graph.block_until_stopped(std::time::Duration::from_secs(1))
    }
}


//standard needs single message passing
//               graph test
//               actor test
//  demo something not send?
//  demo wait_for_all with multiple channels
//  demo state
//  demo clean shutdown
//  will be common base for the following 3
//  hb & gen ->try worker ->async logger/shutdown


// robust will have
//     panic, peek, dlq, externalAwait?

// performant will have
//     full batch usage, skip iterator ?
//     zero copy???visitor?

// distributed will have
//     stream demo between boxes
//


