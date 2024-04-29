use bevy::prelude::UVec2;
use serde::{Deserialize, Serialize};

use crate::{
    cards_v2::deck::Decklist,
    make_enum,
    match_sim::{GridLocation, MatchId, PlayerId, UnplayedCard},
};

make_enum! {
    #[derive(Debug, Serialize, Deserialize)]
    pub enum NetworkMessage {
        JoinMatchmakingQueueMessage,
        MatchStartedMessage,
        AddCardToDeckMessage,
        DrawCardMessage,
        ProtocolErrorMessage,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinMatchmakingQueueMessage {
    pub player_name: String,
    pub deck: Decklist,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchStartedMessage {
    pub match_id: MatchId,
    pub players: Vec<(PlayerId, Decklist)>,
    pub you: PlayerId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddCardToDeckMessage {
    pub match_id: MatchId,
    pub player_id: PlayerId,
    pub card: UnplayedCard,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DrawCardMessage {
    pub match_id: MatchId,
    pub player_id: PlayerId,
    pub card: UnplayedCard,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolErrorMessage {
    pub msg: String,
}
