use std::path::PathBuf;

use clap::Parser;

use crate::utils::version;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
  #[arg(short, long, value_name = "PATH", help = "Path to the project root", default_value = ".")]
  pub project_root: PathBuf,
}
