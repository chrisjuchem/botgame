use bevy::prelude::*;
use bevy_mod_index::prelude::Index;

use crate::{
    cards_v2::deck::Decklist,
    match_sim::{
        CurrentTurn, Deck, Energy, Hand, MatchId, Minerals, PlayerId, Scrap, UnplayedCard,
    },
};

// ====== Events ======

#[derive(Event, Clone)]
pub struct StartMatchEvent {
    pub match_id: MatchId,
    pub players: Vec<(PlayerId, Decklist)>,
}

#[derive(Event, Clone)]
pub struct NewTurnEvent {
    pub match_id: MatchId,
    pub next_player: PlayerId,
}

#[derive(Event, Clone)]
pub struct AddCardToDeckEvent {
    pub match_id: MatchId,
    pub player_id: PlayerId,
    pub card: UnplayedCard,
}

#[derive(Event, Clone)]
pub struct DrawCardEvent {
    pub match_id: MatchId,
    pub player_id: PlayerId,
    pub card: UnplayedCard,
}

#[derive(Event, Clone)]
pub struct CleanupMatchEvent {
    pub match_id: MatchId,
}

pub fn init_events(app: &mut App) {
    app.add_event::<StartMatchEvent>();
    app.add_event::<NewTurnEvent>();
    app.add_event::<AddCardToDeckEvent>();
    app.add_event::<DrawCardEvent>();
    app.add_event::<CleanupMatchEvent>();
}

// ====== Systems ======

pub fn start_match(mut commands: Commands, mut e: EventReader<StartMatchEvent>) {
    for StartMatchEvent { match_id, players } in e.read() {
        info!("match {match_id:?} started");
        for (player_id, decklist) in players.iter() {
            let p = commands
                .spawn((
                    *match_id,
                    *player_id,
                    Hand(vec![]),
                    Deck(vec![]),
                    Energy { max: 0, available: 0 },
                    Scrap(0),
                    Minerals(0),
                    decklist.clone(),
                    Name::new(format!("player_{:?}", player_id.0)),
                ))
                .id();
        }
    }
}

pub fn add_cards_to_deck(
    mut e: EventReader<AddCardToDeckEvent>,
    mut player_idx: Index<PlayerId>,
    mut decks: Query<&mut Deck>,
) {
    for AddCardToDeckEvent { player_id, card, .. } in e.read() {
        decks.get_mut(player_idx.single(player_id)).unwrap().0.push(card.clone())
    }
}

pub fn draw_cards(
    mut e: EventReader<DrawCardEvent>,
    mut player_idx: Index<PlayerId>,
    mut players: Query<(&mut Deck, &mut Hand)>,
) {
    for DrawCardEvent { player_id, card, .. } in e.read() {
        let (mut deck, mut hand): (Mut<Deck>, Mut<Hand>) =
            players.get_mut(player_idx.single(player_id)).unwrap();
        let i = deck
            .0
            .iter()
            .rev() // probably more efficient when entire deck is Unknown
            .enumerate()
            .find(|(i, deck_card)| *deck_card == card)
            .expect("Couldn't find card to draw!")
            .0;
        hand.0.push(deck.0.remove(i));
    }
}

pub fn next_turn(
    mut commands: Commands,
    mut e: EventReader<NewTurnEvent>,
    players: Query<(Entity, &MatchId, &PlayerId, Has<CurrentTurn>)>,
) {
    for NewTurnEvent { match_id, next_player } in e.read() {
        for (e, m, p, t) in players.iter() {
            if m == match_id {
                if t {
                    commands.entity(e).remove::<CurrentTurn>();
                } else if p == next_player {
                    commands.entity(e).insert(CurrentTurn);
                }
            }
        }
    }
}

pub fn cleanup_match(
    mut commands: Commands,
    mut e: EventReader<CleanupMatchEvent>,
    mut match_index: Index<MatchId>,
) {
    for CleanupMatchEvent { match_id } in e.read() {
        for entity in match_index.lookup(match_id) {
            commands.entity(entity).despawn();
        }
    }
}
