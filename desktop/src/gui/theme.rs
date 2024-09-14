#[cfg(target_os = "linux")]
use crate::dbus::{ColorScheme, FreedesktopSettings};
use crate::preferences::GlobalPreferences;
use egui::Context;
use futures::StreamExt;
use std::error::Error;
use std::str::FromStr;
use std::sync::{Arc, Weak};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{Mutex, MutexGuard};
use winit::window::{Theme, Window};

struct ThemeControllerData {
    window: Weak<Window>,
    egui_ctx: Context,
    theme_preference: ThemePreference,

    #[cfg(target_os = "linux")]
    zbus_connection: Option<zbus::Connection>,
}

#[derive(Clone)]
pub struct ThemeController(Arc<Mutex<ThemeControllerData>>);

impl ThemeController {
    pub async fn new(
        window: Arc<Window>,
        preferences: GlobalPreferences,
        egui_ctx: Context,
    ) -> Self {
        let this = Self(Arc::new(Mutex::new(ThemeControllerData {
            window: Arc::downgrade(&window),
            egui_ctx,
            theme_preference: Default::default(), // Will be set later
            #[cfg(target_os = "linux")]
            zbus_connection: zbus::Connection::session()
                .await
                .inspect_err(|err| tracing::warn!("Failed to connect to D-Bus: {err}"))
                .ok(),
        })));

        #[cfg(target_os = "linux")]
        this.start_dbus_theme_watcher_linux().await;
        this.start_theme_preference_watcher(&preferences);

        this.set_theme_preference(preferences.theme_preference())
            .await;

        this
    }

    fn start_theme_preference_watcher(&self, preferences: &GlobalPreferences) {
        let mut theme_preference_watcher = preferences.theme_preference_watcher();
        let this = self.clone();
        tokio::spawn(Box::pin(async move {
            loop {
                match theme_preference_watcher.recv().await {
                    Ok(new_theme_preference) => {
                        this.set_theme_preference(new_theme_preference).await;
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
        }));
    }

    #[cfg(target_os = "linux")]
    async fn start_dbus_theme_watcher_linux(&self) {
        async fn start_inner(this: &ThemeController) -> Result<(), Box<dyn Error>> {
            let Some(ref connection) = this.data().zbus_connection else {
                return Ok(());
            };

            let settings = FreedesktopSettings::new(connection).await?;

            let mut stream = Box::pin(settings.watch_color_scheme().await?);

            let this2 = this.clone();
            tokio::spawn(Box::pin(async move {
                while let Some(scheme) = stream.next().await {
                    match scheme {
                        Ok(scheme) => {
                            this2.set_theme(scheme_to_theme(scheme));
                        }
                        Err(err) => {
                            tracing::warn!(
                                "Error while watching for color scheme changes: {}",
                                err
                            );
                        }
                    }
                }
            }));

            Ok(())
        }

        if let Err(err) = start_inner(self).await {
            tracing::warn!("Error registering theme watcher: {}", err);
        }
    }

    fn data(&self) -> MutexGuard<'_, ThemeControllerData> {
        self.0.try_lock().expect("Non-reentrant data mutex")
    }

    pub fn set_theme(&self, theme: Theme) {
        let data = self.data();
        if data.theme_preference != ThemePreference::System {
            // Cannot change theme when there's a preference.
            return;
        }
        self.set_theme_internal(data, theme);
    }

    async fn set_theme_preference(&self, theme_preference: ThemePreference) {
        let theme = match theme_preference {
            ThemePreference::System => {
                if let Ok(theme) = self
                    .get_system_theme()
                    .await
                    .inspect_err(|err| tracing::warn!("Unable to read system theme: {err}"))
                {
                    theme
                } else {
                    return;
                }
            }
            ThemePreference::Light => Theme::Light,
            ThemePreference::Dark => Theme::Dark,
        };
        let mut data = self.data();
        data.theme_preference = theme_preference;
        self.set_theme_internal(data, theme);
    }

    fn set_theme_internal(&self, data: MutexGuard<'_, ThemeControllerData>, theme: Theme) {
        data.egui_ctx.set_visuals(match theme {
            Theme::Light => egui::Visuals::light(),
            Theme::Dark => egui::Visuals::dark(),
        });
        if let Some(window) = data.window.upgrade() {
            window.request_redraw();
        }
    }

    #[cfg(target_os = "linux")]
    async fn get_system_theme(&self) -> Result<Theme, Box<dyn Error>> {
        let Some(ref connection) = self.data().zbus_connection else {
            return Ok(Theme::Dark);
        };
        let settings = FreedesktopSettings::new(connection).await?;
        let scheme = settings.color_scheme().await?;

        Ok(scheme_to_theme(scheme))
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn get_system_theme(&self) -> Result<Theme, Box<dyn Error>> {
        #[derive(thiserror::Error, Debug)]
        #[error("Unsupported operation")]
        struct UnsupportedOperationError;
        self.data()
            .window
            .upgrade()
            .and_then(|w| w.theme())
            .ok_or(Box::new(UnsupportedOperationError))
    }
}

#[cfg(target_os = "linux")]
fn scheme_to_theme(color_scheme: ColorScheme) -> Theme {
    use crate::dbus::ColorScheme;
    match color_scheme {
        ColorScheme::Default => Theme::Light,
        ColorScheme::PreferLight => Theme::Light,
        ColorScheme::PreferDark => Theme::Dark,
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub fn as_str(&self) -> Option<&'static str> {
        match self {
            ThemePreference::System => None,
            ThemePreference::Light => Some("light"),
            ThemePreference::Dark => Some("dark"),
        }
    }
}

impl FromStr for ThemePreference {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "light" => Ok(ThemePreference::Light),
            "dark" => Ok(ThemePreference::Dark),
            _ => Err(()),
        }
    }
}
