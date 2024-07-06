use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
  sync::Arc,
};

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use redux_rs::{
  middlewares::thunk::{self, ThunkMiddleware},
  Selector, Store, StoreApi,
};
use serde::{Deserialize, Serialize};
use strum::{EnumCount, IntoEnumIterator};
use tokio::sync::mpsc;

use crate::{
  action::{AppAction, TuiAction},
  components::{
    confirm_empty_replace_dialog::ConfirmEmptyReplaceDialog, confirm_git_dir_dialog::ConfirmGitDirDialog, help_dialog::HelpDialog, notifications::{NotificationEnum, Notifications}, preview::Preview, replace::Replace, search::Search, search_result::SearchResult, small_help::SmallHelp, status::Status, Component
  },
  config::Config,
  mode::Mode,
  redux::{
    action::Action,
    reducer::reducer,
    state::State,
    thunk::{thunk_impl, ThunkAction},
  },
  tabs::Tab,
  tui,
};

const FILE_COUNT_THRESHOLD: usize = 1000;

pub struct App {
  pub config: Config,
  pub tick_rate: f64,
  pub frame_rate: f64,
  pub components: Vec<Box<dyn Component>>,
  pub should_quit: bool,
  pub should_suspend: bool,
  pub mode: Mode,
  pub last_tick_key_events: Vec<KeyEvent>,
  pub project_root: PathBuf,
}

impl App {
  pub fn new(project_root: PathBuf) -> Result<Self> {
    let config = Config::new()?;
    let mode = Mode::Normal;

    let search = Search::new();
    let replace = Replace::new();
    let search_result = SearchResult::new();
    let preview = Preview::new();
    let notification = Notifications::new();
    let small_help = SmallHelp::default();
    let confirm_git_dir_dialog = ConfirmGitDirDialog::default();
    let confirm_empty_replace_dialog = ConfirmEmptyReplaceDialog::default();
    let help_dialog = HelpDialog::default();
    let status = Status::default();
    Ok(Self {
      tick_rate: 4.0,
      frame_rate: 24.0,
      components: vec![
        Box::new(search),
        Box::new(replace),
        Box::new(search_result),
        Box::new(preview),
        Box::new(notification),
        Box::new(small_help),
        Box::new(status),
        Box::new(confirm_git_dir_dialog),
        Box::new(confirm_empty_replace_dialog),
        Box::new(help_dialog),
      ],
      should_quit: false,
      should_suspend: false,
      config,
      mode,
      last_tick_key_events: Vec::new(),
      project_root,
    })
  }

  fn is_large_folder(path: &Path) -> bool {
    let output =
      Command::new("rg").args(["--files", "--count-matches", "--max-count", "1", path.to_str().unwrap_or("")]).output();

    match output {
      Ok(output) => {
        if output.status.success() {
          let file_count = String::from_utf8_lossy(&output.stdout).lines().count();
          log::info!("File count: {}", file_count);
          file_count > FILE_COUNT_THRESHOLD
        } else {
          log::error!("ripgrep command failed: {}", String::from_utf8_lossy(&output.stderr));
          false
        }
      },
      Err(e) => {
        log::error!("Failed to execute ripgrep: {}", e);
        false
      },
    }
  }

  pub async fn run(&mut self) -> Result<()> {
    log::info!("Starting app..");
    let initial_state = State::new(self.project_root.clone());
    let mut state = initial_state.clone();

    let (action_tx, mut action_rx) = mpsc::unbounded_channel();
    let (redux_action_tx, mut redux_action_rx) = mpsc::unbounded_channel::<AppAction>();

    let mut tui = tui::Tui::new()?;
    // tui.mouse(true);
    tui.enter()?;

    for component in self.components.iter_mut() {
      component.init(tui.size()?)?;
    }

    // handle big folders
    let is_large_folder = Self::is_large_folder(&self.project_root);
    state.is_large_folder = is_large_folder;
    let store = Store::new_with_state(reducer, state).wrap(ThunkMiddleware).await;
    if is_large_folder {
      let search_text_action = AppAction::Tui(TuiAction::Notify(NotificationEnum::Info(
        "This is a large folder. click 'Enter' to search".to_string(),
      )));
      action_tx.send(search_text_action)?;
    }

    for component in self.components.iter_mut() {
      component.register_action_handler(redux_action_tx.clone())?;
    }

    for component in self.components.iter_mut() {
      component.register_config_handler(self.config.clone())?;
    }

    loop {
      let state = store.state_cloned().await;
      if let Some(e) = tui.next().await {
        match e {
          tui::Event::Quit => action_tx.send(AppAction::Tui(TuiAction::Quit))?,
          tui::Event::Tick => action_tx.send(AppAction::Tui(TuiAction::Tick))?,
          tui::Event::Render => action_tx.send(AppAction::Tui(TuiAction::Render))?,
          tui::Event::Resize(x, y) => action_tx.send(AppAction::Tui(TuiAction::Resize(x, y)))?,
          tui::Event::Key(key) => {
            if let Some(app_action) = self.config.keybindings.get(&vec![key]) {
              log::info!("Got action: {app_action:?}");
              match app_action {
                AppAction::Tui(action) => action_tx.send(AppAction::Tui(action.clone()))?,
                AppAction::Action(action) => redux_action_tx.send(AppAction::Action(action.clone()))?,
                AppAction::Thunk(action) => redux_action_tx.send(AppAction::Thunk(action.clone()))?,
              }
            } else {
              // If the key was not handled as a single key action,
              // then consider it for multi-key combinations.
              self.last_tick_key_events.push(key);

              // Check for multi-key combinations
              if let Some(app_action) = self.config.keybindings.get(&self.last_tick_key_events) {
                log::info!("Got action: {app_action:?}");
                match app_action {
                  AppAction::Tui(action) => action_tx.send(AppAction::Tui(action.clone()))?,
                  AppAction::Action(action) => redux_action_tx.send(AppAction::Action(action.clone()))?,
                  AppAction::Thunk(action) => redux_action_tx.send(AppAction::Thunk(action.clone()))?,
                }
              }
            }
          },
          _ => {},
        }
        for component in self.components.iter_mut() {
          component.handle_events(Some(e.clone()), &state)?;
        }
      }

      let mut rendered = false;
      while let Ok(action) = action_rx.try_recv() {
        if action != AppAction::Tui(TuiAction::Tick) && action != AppAction::Tui(TuiAction::Render) {
          log::debug!("{action:?}");
        }
        match action {
          AppAction::Tui(TuiAction::Tick) => {
            self.last_tick_key_events.drain(..);
          },
          AppAction::Tui(TuiAction::Quit) => self.should_quit = true,
          AppAction::Tui(TuiAction::Suspend) => self.should_suspend = true,
          AppAction::Tui(TuiAction::Resume) => self.should_suspend = false,
          AppAction::Tui(TuiAction::Resize(w, h)) => {
            tui.resize(Rect::new(0, 0, w, h))?;
            tui.draw(|f| {
              for component in self.components.iter_mut() {
                let r = component.draw(f, f.size(), &state);
                if let Err(e) = r {
                  action_tx.send(AppAction::Tui(TuiAction::Error(format!("Failed to draw: {:?}", e)))).unwrap();
                }
              }
            })?;
          },
          AppAction::Tui(TuiAction::Render) => {
            if !rendered {
              rendered = true;
              tui.draw(|f| {
                for component in self.components.iter_mut() {
                  let r = component.draw(f, f.size(), &state);
                  if let Err(e) = r {
                    action_tx.send(AppAction::Tui(TuiAction::Error(format!("Failed to draw: {:?}", e)))).unwrap();
                  }
                }
              })?;
            }
          },
          _ => {},
        }
        for component in self.components.iter_mut() {
          if let Some(action) = component.update(action.clone())? {
            action_tx.send(action)?
          };
        }
      }

      while let Ok(action) = redux_action_rx.try_recv() {
        match action {
          AppAction::Action(action) => {
            log::debug!("Redux action: {action:?}");
            store.dispatch(thunk::ActionOrThunk::Action(action)).await;
          },
          AppAction::Thunk(action) => {
            log::debug!("Thunk action: {action:?}");
            let action_tx_arc = Arc::new(action_tx.clone());
            let thunk = thunk_impl(action, action_tx_arc);

            store.dispatch(thunk::ActionOrThunk::Thunk(thunk)).await;
          },
          AppAction::Tui(action) => {
            log::debug!("Tui action: {action:?}");
            action_tx.send(AppAction::Tui(action.clone()))?;
          },
        }
      }

      if self.should_suspend {
        tui.suspend()?;
        action_tx.send(AppAction::Tui(TuiAction::Resume))?;
        tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()?;
      } else if self.should_quit {
        tui.stop()?;
        break;
      }
    }
    tui.exit()?;
    Ok(())
  }
}
