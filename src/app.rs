use std::{path::PathBuf, sync::Arc};

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
    confirm_empty_replace_dialog::ConfirmEmptyReplaceDialog,
    confirm_git_dir_dialog::ConfirmGitDirDialog,
    notifications::{NotificationEnum, Notifications},
    preview::Preview,
    replace::Replace,
    search::Search,
    search_result::SearchResult,
    small_help::SmallHelp,
    status::Status,
    Component,
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
      ],
      should_quit: false,
      should_suspend: false,
      config,
      mode,
      last_tick_key_events: Vec::new(),
      project_root,
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    log::info!("Starting app..");
    // log project root
    log::info!("Project root: {:?}", self.project_root);
    let initial_state = State::new(self.project_root.clone());
    log::info!("Initial state: {:?}", initial_state);
    let store = Store::new_with_state(reducer, initial_state).wrap(ThunkMiddleware).await;

    let (action_tx, mut action_rx) = mpsc::unbounded_channel();
    let (redux_action_tx, mut redux_action_rx) = mpsc::unbounded_channel::<AppAction>();

    let mut tui = tui::Tui::new()?;
    // tui.mouse(true);
    tui.enter()?;

    for component in self.components.iter_mut() {
      component.register_action_handler(redux_action_tx.clone())?;
    }

    for component in self.components.iter_mut() {
      component.register_config_handler(self.config.clone())?;
    }

    for component in self.components.iter_mut() {
      component.init(tui.size()?)?;
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
            if let Some(keymap) = self.config.keybindings.get(&self.mode) {
              if let Some(app_action) = keymap.get(&vec![key]) {
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
                if let Some(app_action) = keymap.get(&self.last_tick_key_events) {
                  log::info!("Got action: {app_action:?}");
                  match app_action {
                    AppAction::Tui(action) => action_tx.send(AppAction::Tui(action.clone()))?,
                    AppAction::Action(action) => redux_action_tx.send(AppAction::Action(action.clone()))?,
                    AppAction::Thunk(action) => redux_action_tx.send(AppAction::Thunk(action.clone()))?,
                  }
                }
              }
            };
          },
          _ => {},
        }
        for component in self.components.iter_mut() {
          component.handle_events(Some(e.clone()), &state)?;
        }
      }

      let mut rendered = false;
      while let Ok(action) = action_rx.try_recv() {
        // if action != TuiAction::Tick && action != TuiAction::Render {
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

            // store.dispatch(thunk::ActionOrThunk::Thunk(thunk_impl(action, &action_tx))).await;
            store.dispatch(thunk::ActionOrThunk::Thunk(thunk)).await;
          },
          AppAction::Tui(action) => {
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
