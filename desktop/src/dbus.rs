#![cfg(target_os = "linux")]
//! Types and methods utilized for communicating with D-Bus

use std::mem;
use std::sync::{Arc, Mutex};

use ashpd::desktop::settings::{ColorScheme, Settings};
use futures::Stream;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct FreedesktopSettings {
    proxy: Settings<'static>,
}

impl FreedesktopSettings {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            proxy: Settings::new().await?,
        })
    }

    pub async fn color_scheme(&self) -> Result<ColorScheme> {
        Ok(self.proxy.color_scheme().await?)
    }

    pub async fn watch_color_scheme(&self) -> Result<impl Stream<Item = ColorScheme> + use<>> {
        Ok(self.proxy.receive_color_scheme_changed().await?)
    }
}

pub struct GameModeSession {
    _guard: Arc<Mutex<Option<GameModeGuard>>>,
}

impl GameModeSession {
    pub fn new(enabled: bool) -> Self {
        let guard = Arc::new(Mutex::new(None));
        let guard2 = guard.clone();
        tokio::spawn(async move {
            let game_mode_guard = GameModeGuard::new(enabled).await;
            *guard2.lock().expect("Non-poisoned gamemode guard") = Some(game_mode_guard);
        });
        Self { _guard: guard }
    }
}

struct GameModeGuard {
    gamemode: Option<ashpd::desktop::game_mode::GameMode<'static>>,
}

impl GameModeGuard {
    async fn new(enabled: bool) -> Self {
        if !enabled {
            return Self { gamemode: None };
        }

        let gamemode = ashpd::desktop::game_mode::GameMode::new()
            .await
            .inspect_err(|err| tracing::warn!("Failed to initialize gamemode controller: {}", err))
            .ok();

        if let Some(gamemode) = &gamemode
            && let Err(err) = gamemode.register(std::process::id()).await
        {
            tracing::warn!("Failed to register a game with gamemode: {}", err)
        }

        Self { gamemode }
    }
}

impl Drop for GameModeGuard {
    fn drop(&mut self) {
        if let Some(gamemode) = mem::take(&mut self.gamemode) {
            tokio::spawn(async move {
                if let Err(err) = gamemode.unregister(std::process::id()).await {
                    tracing::warn!("Failed to unregister a game with gamemode: {}", err)
                }
            });
        }
    }
}
