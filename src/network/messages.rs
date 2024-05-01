use bevy::prelude::UVec2;
use serde::{Deserialize, Serialize};

use crate::{
    cards::{deck::Decklist, Card, Effect},
    make_enum,
    match_sim::{GridLocation, MatchId, PlayerId, UnplayedCard},
};

make_enum! {
    #[derive(Debug, Serialize, Deserialize)]
    pub enum NetworkMessage {
        JoinMatchmakingQueueMessage,
        MatchStartedMessage,
        EffectMessage,
        AddCardToDeckMessage,
        DrawCardMessage,
        NewTurnMessage,
        ActivateAbilityMessage,
        ProtocolErrorMessage,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinMatchmakingQueueMessage {
    pub player_name: String,
    pub decklist: Decklist,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchStartedMessage {
    pub match_id: MatchId,
    pub players: Vec<(PlayerId, Decklist)>,
    pub you: PlayerId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EffectMessage {
    pub match_id: MatchId,
    pub effect: Effect,
    pub targets: Vec<GridLocation>,
    pub source: Option<GridLocation>,
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
pub struct NewTurnMessage {
    pub match_id: MatchId,
    pub next_player: PlayerId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivateAbilityMessage {
    pub match_id: MatchId,
    pub unit_location: UVec2,
    pub ability_idx: usize,
    pub targets: Vec<GridLocation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolErrorMessage {
    pub msg: String,
}
