use crate::constants::DEFAULT_HEIGHT_STRING;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    /// Input GLTF file (.glb, .gltf)
    #[arg(long = "in", value_name = "PATH")]
    pub(crate) input: PathBuf,

    /// Output PNG file (.png)
    #[arg(long = "out", value_name = "PATH", default_value = "out.png")]
    pub(crate) output: PathBuf,

    /// Rendered image height
    #[arg(long = "height", default_value = DEFAULT_HEIGHT_STRING)]
    pub(crate) height: String,

    /// Rendered image height
    #[arg(long, short = 'P')]
    pub(crate) stay_after_complete: bool,

    /// Rendered image height
    #[arg(long = "camera", default_value = "no camera")]
    pub(crate) camera_name: String,    
}

pub(crate) fn cli_parse() -> Cli {
    Cli::parse()
}
