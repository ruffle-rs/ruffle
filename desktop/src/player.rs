use ruffle_core::Player;
use std::sync::{Arc, Mutex, MutexGuard};

pub struct PlayerController {
    player: Arc<Mutex<Player>>,
}

impl PlayerController {
    pub fn new(player: Arc<Mutex<Player>>) -> Self {
        Self { player }
    }

    pub fn get(&self) -> Option<MutexGuard<Player>> {
        // We don't want to return None when the lock fails to grab as that's a fatal error, not a lack of player
        Some(
            self.player
                .try_lock()
                .expect("Player lock must be available"),
        )
    }
}
