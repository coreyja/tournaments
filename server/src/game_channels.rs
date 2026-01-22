use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

/// Notification sent when a turn completes
#[derive(Debug, Clone)]
pub struct TurnNotification {
    pub game_id: Uuid,
    pub turn_number: i32,
}

/// Manages broadcast channels for live game updates
/// One broadcast channel per active game, subscribers receive turn notifications
#[derive(Debug, Clone)]
pub struct GameChannels {
    /// Map from game_id to broadcast sender for that game
    channels: Arc<RwLock<HashMap<Uuid, broadcast::Sender<TurnNotification>>>>,
}

impl Default for GameChannels {
    fn default() -> Self {
        Self::new()
    }
}

impl GameChannels {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a broadcast channel for a game
    /// Returns a receiver that will receive turn notifications
    pub async fn subscribe(&self, game_id: Uuid) -> broadcast::Receiver<TurnNotification> {
        let mut channels = self.channels.write().await;

        if let Some(sender) = channels.get(&game_id) {
            sender.subscribe()
        } else {
            // Create new channel with buffer of 256 turns
            // This should be enough to handle temporary slowdowns
            let (sender, receiver) = broadcast::channel(256);
            channels.insert(game_id, sender);
            receiver
        }
    }

    /// Send a turn notification to all subscribers for a game
    pub async fn notify(&self, notification: TurnNotification) {
        let channels = self.channels.read().await;

        if let Some(sender) = channels.get(&notification.game_id) {
            // Ignore errors - they mean no receivers are listening
            let _ = sender.send(notification);
        }
    }

    /// Clean up a game's channel if no receivers are listening
    /// Call this periodically or when a game ends
    pub async fn cleanup(&self, game_id: Uuid) {
        let mut channels = self.channels.write().await;

        if let Some(sender) = channels.get(&game_id)
            && sender.receiver_count() == 0
        {
            channels.remove(&game_id);
            tracing::debug!(game_id = %game_id, "Removed game channel (no subscribers)");
        }
    }

    /// Remove a game's channel entirely (call when game ends)
    pub async fn remove(&self, game_id: Uuid) {
        let mut channels = self.channels.write().await;
        channels.remove(&game_id);
        tracing::debug!(game_id = %game_id, "Removed game channel");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_creates_channel() {
        let channels = GameChannels::new();
        let game_id = Uuid::new_v4();

        let _receiver = channels.subscribe(game_id).await;

        // Channel should exist now
        assert!(channels.channels.read().await.contains_key(&game_id));
    }

    #[tokio::test]
    async fn test_notify_sends_to_subscribers() {
        let channels = GameChannels::new();
        let game_id = Uuid::new_v4();

        let mut receiver = channels.subscribe(game_id).await;

        channels
            .notify(TurnNotification {
                game_id,
                turn_number: 5,
            })
            .await;

        let notification = receiver.recv().await.unwrap();
        assert_eq!(notification.game_id, game_id);
        assert_eq!(notification.turn_number, 5);
    }

    #[tokio::test]
    async fn test_cleanup_removes_empty_channels() {
        let channels = GameChannels::new();
        let game_id = Uuid::new_v4();

        // Create and then drop a subscriber
        {
            let _receiver = channels.subscribe(game_id).await;
        }

        // Channel should still exist but with no receivers
        channels.cleanup(game_id).await;

        // Channel should be removed
        assert!(!channels.channels.read().await.contains_key(&game_id));
    }
}
