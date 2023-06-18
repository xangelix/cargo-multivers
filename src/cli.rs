use crate::rustc::Rustc;

#[derive(clap::Parser)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum Cargo {
    #[command(name = "multivers", version, author, about, long_about)]
    Multivers(Args),
}

/// Type of information to print on stdout
#[derive(clap::ValueEnum, Clone, Copy)]
pub enum Print {
    /// Prints the list of CPU features supported by the target
    CpuFeatures,
}

#[derive(clap::Args)]
pub struct Args {
    /// Build for the target triple
    #[clap(long, value_name = "TRIPLE")]
    pub target: Option<String>,

    /// Print information on stdout
    #[clap(long, value_name = "INFORMATION")]
    pub print: Option<Print>,

    /// Comma-separated list of CPUs to use as a target
    #[clap(
        long,
        use_value_delimiter = true,
        value_delimiter = ',',
        value_name = "CPUs"
    )]
    pub cpus: Option<Vec<String>>,

    /// Comma-separated list of CPU features to exclude from the builds
    #[clap(
        long,
        use_value_delimiter = true,
        value_delimiter = ',',
        value_name = "CPU-FEATURES"
    )]
    pub exclude_cpu_features: Option<Vec<String>>,

    #[command(flatten)]
    pub manifest: clap_cargo::Manifest,

    #[command(flatten)]
    pub workspace: clap_cargo::Workspace,

    #[command(flatten)]
    pub features: clap_cargo::Features,

    /// Arguments given to cargo build
    #[clap(raw = true)]
    pub args: Vec<String>,
}

impl Args {
    /// Returns the target given on the command line or the default target that rustc uses to build if none is provided
    pub fn target(&self) -> anyhow::Result<String> {
        self.target
            .as_ref()
            .map_or_else(Rustc::default_target, |target| Ok(target.clone()))
    }
}
