#[derive(Event, Clone)]
pub struct EffectEvent {
    pub match_id: MatchId,
    pub effect: Effect,
    pub targets: Vec<GridLocation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EffectMessage {
    pub match_id: MatchId,
    pub effect: Effect,
    pub targets: Vec<GridLocation>,
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
