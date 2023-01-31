use crate::debug::avm1_message::Avm1Msg;
use crate::debug::player_message::PlayerMsg;
use crate::debug::targeted_message::TargetedMsg;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DebugMessageIn {
    /// Send a command to the object at the given path
    Targeted { path: String, msg: TargetedMsg },

    /// Send a command to the player
    Player { msg: PlayerMsg },

    /// Send a command to AVM1
    Avm1 { msg: Avm1Msg },
}
