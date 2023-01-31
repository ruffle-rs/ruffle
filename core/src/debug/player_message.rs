use serde::{Deserialize, Serialize};

/// Debug messages that are handled in the player
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum PlayerMsg {
    /// Pause the player
    Pause,

    /// Resume the player
    Play,
}
