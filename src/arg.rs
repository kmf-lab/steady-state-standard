use clap::Parser;

/// Command-line argument structure demonstrating runtime configuration integration.
/// This is normal 'clap' and for more details you should review their documentation.
#[derive(Parser, Debug, PartialEq, Clone)]
pub(crate) struct MainArg {
    /// Timing control parameter for adjusting system responsiveness.
    /// Lower values increase CPU usage but improve reaction time,
    /// while higher values reduce overhead at the cost of latency.
    #[arg(short = 'r', long = "rate", default_value = "1000")]
    pub(crate) rate_ms: u64,

    /// Lifecycle control parameter for automated termination.
    /// This enables demo runs, batch processing limits, and testing scenarios
    /// that need predictable completion behavior.
    #[arg(short = 'b', long = "beats", default_value = "60")]
    pub(crate) beats: u64,
}

/// Default implementation provides fallback values for testing and API usage.
/// This ensures consistent behavior when command-line parsing isn't available
/// or when actors are used programmatically within larger applications.
impl Default for MainArg { //#!#//
    fn default() -> Self {
        MainArg {
            rate_ms: 1000,
            beats: 60,
        }
    }
}
