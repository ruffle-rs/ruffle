use ruffle_core::Player;
use std::sync::{Arc, Mutex, MutexGuard};

pub struct PlayerController {
    player: Arc<Mutex<Player>>,
}

impl PlayerController {
    pub fn new(player: Arc<Mutex<Player>>) -> Self {
        Self { player }
    }

    pub fn get(&self) -> MutexGuard<Player> {
        self.player
            .try_lock()
            .expect("Player lock must be available")
    }
}
