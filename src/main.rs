#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod action;
pub mod app;
pub mod astgrep;
pub mod cli;
pub mod components;
pub mod config;
pub mod layout;
pub mod macros;
pub mod mode;
pub mod redux;
pub mod ripgrep;
pub mod tabs;
pub mod tui;
pub mod ui;
pub mod utils;

use std::process::Command;

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::{eyre, Result};
use log::LevelFilter;

use crate::{
  app::App,
  utils::{initialize_logging, initialize_panic_handler, version},
};

fn check_dependency(command: &str) -> bool {
  Command::new(command).arg("--version").output().is_ok()
}

async fn tokio_main() -> Result<()> {
  // let _ = simple_logging::log_to_file("serpl.log", LevelFilter::Info);

  if !check_dependency("rg") {
    eprintln!("\x1b[31mError: ripgrep (rg) is not installed. Please install it to use serpl.\x1b[0m");
    return Err(eyre!("ripgrep is not installed"));
  }

  #[cfg(feature = "ast_grep")]
  if !check_dependency("ast-grep") {
    eprintln!("\x1b[31mError: ast-grep is not installed. Please install it to use serpl with AST features.\x1b[0m");
    return Err(eyre!("ast-grep is not installed"));
  }
  initialize_panic_handler()?;

  let args = Cli::parse();
  let mut app = App::new(args.project_root)?;
  app.run().await?;

  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  if let Err(e) = tokio_main().await {
    eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
    Err(e)
  } else {
    Ok(())
  }
}
