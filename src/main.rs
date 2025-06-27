use steady_state::*;
use arg::MainArg;
mod arg;

/// Actor module organization demonstrates scalable code structure.
/// This pattern enables clean separation of concerns while maintaining
/// visibility and reusability across different deployment configurations.
pub(crate) mod actor {//#!#//
    pub(crate) mod heartbeat;
    pub(crate) mod generator;
    pub(crate) mod worker;
    pub(crate) mod logger;
}

/// Application entry point demonstrating production-ready initialization patterns.
/// This includes command-line processing, logging setup, graph construction,
/// and lifecycle management with proper error handling and resource cleanup.
fn main() -> Result<(), Box<dyn Error>> {

    let cli_args = MainArg::parse();
    init_logging(LogLevel::Info)?;
    let mut graph = GraphBuilder::default()
        // The default and minimum telemetry frame rate is 40ms. It works well for most cases.
        //.with_telemtry_production_rate_ms(200) //You can slow it down with this  //#!#//
        .build(cli_args);

    build_graph(&mut graph);

    // Synchronous startup ensures all actors are ready before proceeding.
    // This prevents race conditions during initialization and provides
    // predictable system behavior from the start.
    graph.start();
    // Blocking wait with timeout prevents infinite hangs while allowing
    // graceful shutdown completion. The timeout you set should be larger than
    // the expected cleanup duration for all actors to avoid premature termination.
    graph.block_until_stopped(Duration::from_secs(5))
}

/// Actor name constants enable refactoring safety and consistent identification.
/// This pattern prevents typos in string literals while providing a central
/// location for actor naming conventions and namespace management.
const NAME_HEARTBEAT: &str = "HEARTBEAT";
const NAME_GENERATOR: &str = "GENERATOR";
const NAME_WORKER: &str = "WORKER";
const NAME_LOGGER: &str = "LOGGER";

/// Graph construction function demonstrates systematic actor system assembly.
/// This pattern separates topology definition from application logic,
/// enabling easier testing, configuration management, and deployment flexibility.
fn build_graph(graph: &mut Graph) {

    // Channel builder configuration applies consistent monitoring across all channels.
    // This provides uniform observability and alerting behavior without requiring
    // individual channel configuration or runtime performance analysis.
    let channel_builder = graph.channel_builder()
        // Threshold-based alerting enables proactive monitoring of system health.
        // Red alerts indicate critical congestion requiring immediate attention,
        // while orange alerts provide early warning of developing bottlenecks.
        .with_filled_trigger(Trigger::AvgAbove(Filled::p90()), AlertColor::Red) //#!#//
        .with_filled_trigger(Trigger::AvgAbove(Filled::p60()), AlertColor::Orange)
        // Percentile monitoring provides statistical insight into channel utilization.
        .with_filled_percentile(Percentile::p80());

    // The builder is used to build the channels. Note that we do NOT require any type information.
    let (heartbeat_tx, heartbeat_rx) = channel_builder.build();
    let (generator_tx, generator_rx) = channel_builder.build();
    let (worker_tx, worker_rx) = channel_builder.build();

    // Actor builder configuration provides consistent performance monitoring.
    // Load averaging shows relative resource consumption across actors,
    // while CPU monitoring tracks absolute resource utilization per actor.
    let actor_builder = graph.actor_builder()
        // Load distribution metrics enable capacity planning and bottleneck identification.
        // This shows which actors consume the most resources relative to graph capacity.
        .with_load_avg()//#!#//
        // CPU utilization tracking provides absolute performance measurement.
        // Values are normalized to 1024 units per core for consistent cross-platform metrics.
        .with_mcpu_avg();//#!#//

    // State management demonstrates persistent actor behavior across restarts.
    // Each actor maintains independent state that survives crashes, enabling
    // fault-tolerant operation without external persistence mechanisms.
    let state = new_state();
    actor_builder.with_name(NAME_HEARTBEAT)
        // It is a very normal pattern to see every channel and state cloned here. This enables us
        // to keep an Arc here for recovery should this actor panic.  //#!#//
        .build(move |actor| { actor::heartbeat::run(actor, heartbeat_tx.clone(), state.clone()) }
               , SoloAct); 

    // NOTE: that no type information is needed for state.
    let state = new_state();
    actor_builder.with_name(NAME_GENERATOR)
        .build(move |actor| { actor::generator::run(actor, generator_tx.clone(), state.clone()) }
               , SoloAct);

    // Multi-input actors demonstrate complex data flow coordination.
    // The worker receives timing signals from heartbeat and data from generator,
    // enabling controlled batch processing with predictable timing behavior.
    actor_builder.with_name(NAME_WORKER)
        .build(move |actor| { actor::worker::run(actor, heartbeat_rx.clone(), generator_rx.clone(), worker_tx.clone()) }
               , SoloAct);

    // Terminal actors focus on external system integration and side effects.
    // Loggers typically have no outgoing channels but provide essential
    // observability and debugging capabilities for system operation.
    actor_builder.with_name(NAME_LOGGER)
        .build(move |actor| { actor::logger::run(actor, worker_rx.clone()) }
               , SoloAct);
}

/// Integration testing module demonstrates end-to-end system validation.
/// This pattern verifies complete actor system behavior including complex
/// multi-actor interactions and message flow coordination.
#[cfg(test)]
pub(crate) mod main_tests {
    use steady_state::*;
    use steady_state::graph_testing::*;
    use crate::actor::worker::FizzBuzzMessage;
    use super::*;

    #[test]
    fn graph_test() -> Result<(), Box<dyn Error>> {

        //Must build graph for testing or stage_manager will not be available.
        let mut graph = GraphBuilder::for_testing() //#!#//
                         .build(MainArg::default());

        // We call the same production code to build the graph here for testing
        build_graph(&mut graph);
        graph.start();

        // Stage management provides orchestrated testing of multi-actor scenarios.
        // This enables precise control over actor behavior and verification of
        // complex system interactions without manual coordination complexity.
        let stage_manager = graph.stage_manager(); //#!#//
        // This makes use of the "simulated" actors to mock what they send or expect to receive.
        stage_manager.actor_perform(NAME_GENERATOR, StageDirection::Echo(15u64))?;
        stage_manager.actor_perform(NAME_HEARTBEAT, StageDirection::Echo(100u64))?;
        stage_manager.actor_perform(NAME_LOGGER,    StageWaitFor::Message(FizzBuzzMessage::FizzBuzz
                                                                          , Duration::from_secs(2)))?;
        // Must stop stage manager which has been communicating to our simulated actors.
        stage_manager.final_bow(); //#!#//

        graph.request_shutdown();

        graph.block_until_stopped(Duration::from_secs(5))

    }
}
