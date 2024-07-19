#![cfg(target_os = "linux")]
//! Types and methods utilized for communicating with D-Bus

use futures::StreamExt;
use zbus::export::futures_core::Stream;
use zbus::zvariant::{OwnedValue, Value};
use zbus::{proxy, Connection};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop",
    gen_blocking = false
)]
trait FreedesktopSettingsInterface {
    fn read(&self, namespace: &str, key: &str) -> Result<OwnedValue>;

    #[zbus(signal)]
    fn setting_changed(&self, namespace: &str, key: &str, value: Value<'_>);
}

pub enum ColorScheme {
    Default,
    PreferLight,
    PreferDark,
}

#[derive(thiserror::Error, Debug)]
pub enum FreedesktopSettingsError {
    #[error("Unexpected value for color scheme: {0}")]
    UnexpectedColorScheme(Value<'static>),
}

pub struct FreedesktopSettings<'p> {
    proxy: FreedesktopSettingsInterfaceProxy<'p>,
}

impl<'p> FreedesktopSettings<'p> {
    const COLOR_SCHEME: (&'static str, &'static str) =
        ("org.freedesktop.appearance", "color-scheme");

    pub async fn new(connection: &Connection) -> Result<Self> {
        Ok(Self {
            proxy: FreedesktopSettingsInterfaceProxy::new(connection).await?,
        })
    }

    pub async fn color_scheme(&self) -> Result<ColorScheme> {
        let (namespace, key) = Self::COLOR_SCHEME;
        let value = self.proxy.read(namespace, key).await?;
        Self::parse_color_scheme(value.into())
    }

    pub async fn watch_color_scheme(&self) -> Result<impl Stream<Item = Result<ColorScheme>>> {
        let stream: SettingChangedStream = self.proxy.receive_setting_changed().await?;
        Ok(stream.filter_map(|value| async move {
            let args = value.args().ok()?;
            if (args.namespace, args.key) == Self::COLOR_SCHEME {
                Some(Self::parse_color_scheme(args.value))
            } else {
                None
            }
        }))
    }

    fn parse_color_scheme(mut value: Value<'_>) -> Result<ColorScheme> {
        while let Value::Value(inner_value) = value {
            value = *inner_value;
        }

        match value {
            Value::U32(0) => Ok(ColorScheme::Default),
            Value::U32(1) => Ok(ColorScheme::PreferDark),
            Value::U32(2) => Ok(ColorScheme::PreferLight),
            value => Err(FreedesktopSettingsError::UnexpectedColorScheme(
                value.try_to_owned()?.into(),
            ))?,
        }
    }
}
