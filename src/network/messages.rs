use serde::{Deserialize, Serialize};

use crate::{
    cards::Card,
    make_enum,
    match_sim::{MatchId, PlayerId},
};

make_enum! {
    #[derive(Debug, Serialize, Deserialize)]
    pub enum NetworkMessage {
        JoinMatchmakingQueue,
        MatchStarted,
        ProtocolError,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinMatchmakingQueue {
    pub player_name: String,
    pub deck: Card,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchStarted {
    pub match_id: MatchId,
    pub players: Vec<(PlayerId, Card)>,
    pub you: PlayerId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolError {
    pub(crate) msg: String,
}
