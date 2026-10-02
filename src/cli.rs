use std::path::PathBuf;

use clap::Parser;

use crate::utils::version;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
  #[arg(short, long, value_name = "PATH", help = "Path to the project root", default_value = ".")]
  pub project_root: PathBuf,

  #[arg(short = 'H', long, help = "Include hidden files in the search (search runs on Enter only)")]
  pub hidden: bool,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hidden_is_off_by_default() {
    let cli = Cli::parse_from(["serpl"]);
    assert!(!cli.hidden);
  }

  #[test]
  fn hidden_flag_long_and_short() {
    assert!(Cli::parse_from(["serpl", "--hidden"]).hidden);
    assert!(Cli::parse_from(["serpl", "-H", "-p", "/tmp"]).hidden);
  }
}
