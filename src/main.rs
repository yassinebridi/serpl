#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod action;
pub mod app;
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

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::Result;
use log::LevelFilter;

use crate::{
  app::App,
  utils::{initialize_logging, initialize_panic_handler, version},
};

async fn tokio_main() -> Result<()> {
  // let _ = simple_logging::log_to_file("serpl.log", LevelFilter::Info);

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
