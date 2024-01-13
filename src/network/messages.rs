use serde::{Deserialize, Serialize};

use crate::{
    cards::{Card, Effect, Target},
    make_enum,
    match_sim::{MatchId, PlayerId},
};

make_enum! {
    #[derive(Debug, Serialize, Deserialize)]
    pub enum NetworkMessage {
        JoinMatchmakingQueueMessage,
        MatchStartedMessage,
        EffectMessage,
        ProtocolErrorMessage,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinMatchmakingQueueMessage {
    pub player_name: String,
    pub deck: Card,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchStartedMessage {
    pub match_id: MatchId,
    pub players: Vec<PlayerId>,
    pub you: PlayerId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EffectMessage {
    pub match_id: MatchId,
    pub effect: Effect,
    pub target: Target,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolErrorMessage {
    pub(crate) msg: String,
}
