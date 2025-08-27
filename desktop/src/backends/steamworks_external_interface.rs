use ruffle_core::context::UpdateContext;
use ruffle_core::external::{ExternalInterfaceProvider, Value as ExternalValue};
use std::cell::RefCell;
use steamworks::{AppId, Client};

#[derive(Default)]
pub struct SteamWorksExternalInterfaceProvider {
    pub client: RefCell<Option<Client>>,
}

impl ExternalInterfaceProvider for SteamWorksExternalInterfaceProvider {
    fn call_method(
        &self,
        _context: &mut UpdateContext<'_>,
        name: &str,
        args: &[ExternalValue],
    ) -> ExternalValue {
        let Some(ref client) = *self.client.borrow() else {
            if name == "steamworks.client.init" {
                let client = if let [ExternalValue::Number(id)] = args {
                    Client::init_app(AppId(*id as u32))
                } else {
                    Client::init()
                };

                match client {
                    Ok(client) => {
                        self.client.replace(Some(client));
                    }
                    Err(err) => {
                        tracing::warn!("Client::init failed: {err}");
                        return ExternalValue::String(err.to_string());
                    }
                }
            } else {
                tracing::warn!(
                    "Steamworks client not initialized! Expected call to steamworks.client.init."
                );
            }

            return ExternalValue::Undefined;
        };

        // API is heavily inspired by https://github.com/ceifa/steamworks.js/

        match name {
            "steamworks.utils.getAppId" => ExternalValue::Number(client.utils().app_id().0 as f64),
            "steamworks.utils.isSteamRunningOnSteamDeck" => {
                ExternalValue::Bool(client.utils().is_steam_running_on_steam_deck())
            }

            "steamworks.localplayer.getSteamId" => {
                ExternalValue::String(client.user().steam_id().steamid32())
            }
            "steamworks.localplayer.getName" => ExternalValue::String(client.friends().name()),
            "steamworks.localplayer.getLevel" => {
                ExternalValue::Number(client.user().level() as f64)
            }
            "steamworks.localplayer.getIpCountry" => {
                ExternalValue::String(client.utils().ip_country())
            }

            "steamworks.achievement.activate" => {
                let [ExternalValue::String(achievement)] = args else {
                    tracing::warn!("steamworks.achievement.activate: Expected string argument");
                    return ExternalValue::Undefined;
                };

                ExternalValue::Bool(
                    client
                        .user_stats()
                        .achievement(achievement)
                        .set()
                        .and_then(|_| client.user_stats().store_stats())
                        .is_ok(),
                )
            }
            "steamworks.achievement.isActivated" => {
                let [ExternalValue::String(achievement)] = args else {
                    tracing::warn!("steamworks.achievement.isActivated: Expected string argument");
                    return ExternalValue::Undefined;
                };

                ExternalValue::Bool(
                    client
                        .user_stats()
                        .achievement(achievement)
                        .get()
                        .unwrap_or(false),
                )
            }
            "steamworks.achievement.clear" => {
                let [ExternalValue::String(achievement)] = args else {
                    tracing::warn!("steamworks.achievement.clear: Expected string argument");
                    return ExternalValue::Undefined;
                };

                ExternalValue::Bool(
                    client
                        .user_stats()
                        .achievement(achievement)
                        .set()
                        .and_then(|_| client.user_stats().store_stats())
                        .is_ok(),
                )
            }

            _ => {
                tracing::warn!(
                    "Trying to call unknown SteamWorksExternalInterfaceProvider method: {name}"
                );
                ExternalValue::Undefined
            }
        }
    }

    fn on_callback_available(&self, _name: &str) {}

    fn get_id(&self) -> Option<String> {
        None
    }
}
