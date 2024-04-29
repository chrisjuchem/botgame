use bevy::prelude::*;
use bevy_mod_index::prelude::*;
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    cards_v2::deck::Decklist,
    match_sim::{
        events::{AddCardToDeckEvent, DrawCardEvent, StartMatchEvent},
        PlayerId, UnplayedCard,
    },
};

pub fn fill_decks_and_hands(
    mut trigger: EventReader<StartMatchEvent>, // todo wait until "sideboard cards" chosen
    mut spawns: EventWriter<AddCardToDeckEvent>,
    mut draws: EventWriter<DrawCardEvent>,
) {
    let mut rng = thread_rng();

    for StartMatchEvent { match_id, players } in trigger.read() {
        for (pid, decklist) in players {
            let mut deck = vec![];
            deck.extend_from_slice(&decklist.cards);
            deck.extend_from_slice(&decklist.cards);
            deck.extend_from_slice(&decklist.cards);
            deck.shuffle(&mut rng);

            for card in deck.iter().rev().take(2) {
                draws.send(DrawCardEvent {
                    match_id: *match_id,
                    player_id: *pid,
                    card: UnplayedCard::Known(card.clone()),
                });
            }

            for card in deck {
                spawns.send(AddCardToDeckEvent {
                    match_id: *match_id,
                    player_id: *pid,
                    card: UnplayedCard::Known(card),
                });
            }
        }
    }
}
