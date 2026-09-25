use crate::card::{Commander, KeyWord, SubCard};
use std::ops::Add;
#[derive(PartialEq, Default, Debug)]
pub struct CombatData {
    pub damage: i32,
    pub commander: u32,
    pub second_commander: u32,
    pub lifegain: u32,
}
impl Add for CombatData {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            damage: self.damage + rhs.damage,
            commander: self.commander + rhs.commander,
            second_commander: self.second_commander + rhs.second_commander,
            lifegain: self.lifegain + rhs.lifegain,
        }
    }
}
impl CombatData {
    pub fn new(damage: i32, lifegain: u32) -> Self {
        Self {
            damage,
            commander: 0,
            second_commander: 0,
            lifegain,
        }
    }
    pub fn get(attacker: &SubCard, blockers: &mut [&SubCard]) -> Option<Self> {
        fn strike(
            attacker: &SubCard,
            blockers: &[&SubCard],
            attacker_toughness: &mut u32,
            blockers_toughness: &mut [Vec<u32>],
            can_strike: impl Fn(&SubCard) -> bool,
        ) -> CombatData {
            let mut damage = 0;
            let mut commander = 0;
            let mut second_commander = 0;
            let mut lifegain = 0;
            if *attacker_toughness == 0 {
                return CombatData {
                    damage,
                    commander,
                    second_commander,
                    lifegain,
                };
            }
            let mut power = if can_strike(attacker) {
                attacker.get_power().unwrap()
            } else {
                0
            };
            if attacker.has(KeyWord::Lifelink)
                && (attacker.has(KeyWord::Trample)
                    || blockers.is_empty()
                    || blockers_toughness.iter().flatten().any(|t| *t != 0))
            {
                lifegain += power;
            }
            for (blocker, toughnesses) in blockers.iter().zip(blockers_toughness) {
                for toughness in toughnesses {
                    if *toughness == 0 {
                        continue;
                    }
                    if can_strike(blocker) {
                        let blocker_power = blocker.get_power().unwrap();
                        if blocker.has(KeyWord::Lifelink) {
                            damage = damage.saturating_sub_unsigned(blocker_power);
                        }
                        if !attacker.has(KeyWord::Indestructible) {
                            let assigned = if blocker.has(KeyWord::Deathtouch) {
                                1
                            } else {
                                *attacker_toughness
                            };
                            if blocker_power >= assigned {
                                *attacker_toughness = 0;
                            } else {
                                *attacker_toughness -= blocker_power;
                            }
                        }
                    }
                    let assigned = if attacker.has(KeyWord::Deathtouch) {
                        1
                    } else {
                        *toughness
                    };
                    if let Some(next) = power.checked_sub(assigned) {
                        power = next;
                        if !blocker.has(KeyWord::Indestructible) {
                            *toughness = 0;
                        }
                    } else {
                        if !blocker.has(KeyWord::Indestructible) {
                            *toughness -= power;
                        }
                        power = 0;
                    }
                }
            }
            if attacker.has(KeyWord::Trample) || blockers.is_empty() {
                damage = damage.saturating_add_unsigned(power);
                match attacker.commander {
                    Commander::Not => {}
                    Commander::First => {
                        commander += power;
                    }
                    Commander::Second => {
                        second_commander += power;
                    }
                }
            }
            CombatData {
                damage,
                commander,
                second_commander,
                lifegain,
            }
        }
        if !attacker.can_be_in_combat()
            || blockers.iter().any(|c| !c.can_be_in_combat())
            || attacker.get_amount() > 1
        {
            return None;
        }
        blockers.sort_by(|a, b| {
            a.has(KeyWord::Indestructible)
                .cmp(&b.has(KeyWord::Indestructible))
                .then(b.get_toughness().cmp(&a.get_toughness()))
        });
        let mut attacker_toughness = attacker.get_toughness().unwrap();
        let mut blockers_toughness = blockers
            .iter()
            .map(|c| vec![c.get_toughness().unwrap(); c.get_amount() as usize])
            .collect::<Vec<_>>();
        let first = strike(
            attacker,
            blockers,
            &mut attacker_toughness,
            &mut blockers_toughness,
            SubCard::has_first_strike_damage,
        );
        let normal = strike(
            attacker,
            blockers,
            &mut attacker_toughness,
            &mut blockers_toughness,
            SubCard::has_normal_strike_damage,
        );
        Some(first + normal)
    }
}
