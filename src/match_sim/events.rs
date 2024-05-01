use bevy::{
    ecs::event::{Event, ManualEventReader},
    prelude::{
        debug, info, App, Commands, DespawnRecursiveExt, Entity, EventReader, EventWriter, Events,
        Has, Local, Mut, Name, Query, ResMut,
    },
};
use bevy_mod_index::prelude::Index;

use crate::{
    cards::{deck::Decklist, Ability, Effect, ImplicitTargetRules, PassiveAbility, PassiveEffect},
    match_sim::{
        Cards, CardsMut, CommandExts, CurrentTurn, Deck, Energy, GridLocation, Hand, MatchId,
        PlayerId, UnplayedCard,
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
pub struct EffectEvent {
    pub match_id: MatchId,
    pub effect: Effect,
    pub targets: Vec<GridLocation>,
    pub source: Option<GridLocation>,
}

#[derive(Event, Clone)]
pub struct CleanupMatchEvent {
    match_id: MatchId,
}

pub fn init_events(app: &mut App) {
    app.add_event::<StartMatchEvent>();
    app.add_event::<NewTurnEvent>();
    app.add_event::<AddCardToDeckEvent>();
    app.add_event::<DrawCardEvent>();
    //shuffle deck (seed)
    app.add_event::<EffectEvent>();
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
                    decklist.clone(),
                    Name::new(format!("player_{:?}", player_id.0)),
                ))
                .id();
        }
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

pub fn client_effects(mut e: EventReader<EffectEvent>) {
    for EffectEvent { match_id, effect, targets, source } in e.read() {}
}

pub fn common_effects(
    mut commands: Commands,
    mut e: EventReader<EffectEvent>,
    mut cards: CardsMut,
    mut loc_idx: Index<GridLocation>,
) {
    for EffectEvent { match_id, effect, targets, source } in e.read() {
        debug!("effect {effect:?} with targets {targets:?}");
        match effect {
            Effect::SummonRobot { robot } => {
                for t in targets {
                    commands.spawn_robot(robot.clone(), *match_id, *t)
                }
            },
            Effect::GrantAbilities { abilities } => {
                for t in targets {
                    cards
                        .get_mut(loc_idx.single(t))
                        .unwrap()
                        .abilities
                        .0
                        .extend_from_slice(abilities);
                }
            },
            Effect::ChangeHp { amount } => {
                for t in targets {
                    cards.get_mut(loc_idx.single(t)).unwrap().health.0 += amount;
                }
            },
            Effect::ChangeEnergy { amount } => {
                todo!();
                // for t in targets {
                //     let mut e = cards.get_mut(loc_idx.single(t)).unwrap().energy;
                //     e.current = (e.current as i32 + amount).clamp(0, e.max as i32) as u32;
                // }
            },
            Effect::DestroyCard => {
                // triggering abilities handled by server effects
                for t in targets {
                    println!("despawn {t:?}");
                    commands.entity(loc_idx.single(t)).despawn_recursive();
                }
            },
            Effect::Attack { .. } | Effect::MultipleEffects { .. } => {
                // Handled by server_effects
            },
        }
    }
}

// todo read from a stack instead of using event queue
pub fn server_effects(
    mut e: ResMut<Events<EffectEvent>>,
    mut e_reader: Local<ManualEventReader<EffectEvent>>,
    cards: Cards,
    mut loc_idx: Index<GridLocation>,
    mut match_idx: Index<MatchId>,
) {
    let mut new_events = vec![];
    for EffectEvent { match_id, effect, targets, source } in e_reader.read(&*e) {
        match effect {
            Effect::Attack => {
                for t in targets {
                    // let mut final_factor = 1.;
                    // for (ability, ability_source_loc) in cards
                    //     .iter_many(match_idx.lookup(match_id))
                    //     .flat_map(|card| card.abilities.0.iter().map(|a| (a, *card.grid_loc)))
                    // {
                    //     if let Ability::Passive { passive_effect, target_filter } = ability {
                    //         if target_filter.validate(t, &mut loc_idx, &cards, &ability_source_loc)
                    //         {
                    //             match passive_effect {
                    //                 PassiveEffect::DamageResistance {
                    //                     effect_type: effect_filter,
                    //                     factor,
                    //                 } => {
                    //                     if *effect_type == *effect_filter {
                    //                         final_factor *= factor;
                    //                     }
                    //                 },
                    //                 PassiveEffect::WhenHit { effect, target_rules } => {
                    //                     let target = match target_rules {
                    //                         ImplicitTargetRules::ThisUnit => ability_source_loc,
                    //                         ImplicitTargetRules::ThatUnit => *t,
                    //                     };
                    //
                    //                     new_events.push(EffectEvent {
                    //                         match_id: *match_id,
                    //                         effect: effect.clone(),
                    //                         targets: vec![target],
                    //                     })
                    //                 },
                    //                 PassiveEffect::WhenDies { .. } => {},
                    //             }
                    //         }
                    //     }
                    // }
                    //
                    // let final_dmg = *damage as f32 * final_factor;

                    let src_card = cards.get(loc_idx.single(&source.unwrap())).unwrap();
                    let target_card = cards.get(loc_idx.single(t)).unwrap();

                    new_events.push(EffectEvent {
                        match_id: *match_id,
                        effect: Effect::ChangeHp { amount: -(src_card.attack.0 as i32) },
                        targets: vec![*t],
                        source: *source,
                    });
                    new_events.push(EffectEvent {
                        match_id: *match_id,
                        effect: Effect::ChangeHp { amount: -(target_card.attack.0 as i32) },
                        targets: vec![source.unwrap()],
                        source: Some(*t),
                    });
                }
            },
            Effect::MultipleEffects { effects } => {
                for e in effects {
                    new_events.push(EffectEvent {
                        match_id: *match_id,
                        effect: e.clone(),
                        targets: targets.clone(),
                        source: *source,
                    })
                }
            },
            Effect::DestroyCard => {
                // despawning handled by common effects
                for t in targets {
                    for (ability, ability_source_loc) in cards
                        .iter_many(match_idx.lookup(match_id))
                        .flat_map(|card| card.abilities.0.iter().map(|a| (a, *card.grid_loc)))
                    {
                        if let Ability::Passive(PassiveAbility { passive_effect, target_filter }) =
                            ability
                        {
                            if target_filter.validate(t, &mut loc_idx, &cards, &ability_source_loc)
                            {
                                if let PassiveEffect::WhenDies { effect, target_rules } =
                                    passive_effect
                                {
                                    let target = match target_rules {
                                        ImplicitTargetRules::ThisUnit => ability_source_loc,
                                        ImplicitTargetRules::ThatUnit => *t,
                                    };

                                    new_events.push(EffectEvent {
                                        match_id: *match_id,
                                        effect: effect.clone(),
                                        targets: vec![target],
                                        source: None,
                                    })
                                }
                            }
                        }
                    }
                }
            },
            Effect::SummonRobot { .. }
            | Effect::GrantAbilities { .. }
            | Effect::ChangeHp { .. }
            | Effect::ChangeEnergy { .. } => {
                // Handled by common_effects
            },
        }
    }
    for ev in new_events.into_iter() {
        e.send(ev);
    }
}

pub fn server_state_based(mut e: EventWriter<EffectEvent>, cards: Cards) {
    for card in &cards {
        if card.health.0 <= 0 {
            e.send(EffectEvent {
                match_id: *card.match_id,
                effect: Effect::DestroyCard,
                targets: vec![*card.grid_loc],
                source: None,
            });
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
