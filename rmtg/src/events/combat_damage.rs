use importer::card::{KeyWord, SubCard};
pub struct CombatData {
    pub damage: i32,
    pub lifegain: u32,
}
impl CombatData {
    pub fn get(attacker: &SubCard, blockers: &[&SubCard]) -> Option<Self> {
        fn strike(
            attacker: &SubCard,
            blockers: &[&SubCard],
            attacker_toughness: &mut u32,
            blockers_toughness: &mut [Vec<u32>],
            can_strike: impl Fn(&SubCard) -> bool,
        ) -> CombatData {
            let mut damage = 0;
            let mut lifegain = 0;
            if *attacker_toughness == 0 {
                return CombatData { damage, lifegain };
            }
            let mut power = if can_strike(attacker) {
                attacker.get_power().unwrap()
            } else {
                0
            };
            if attacker.has(KeyWord::Lifelink) {
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
            if attacker.has(KeyWord::Trample) {
                damage = damage.saturating_add_unsigned(power);
            }
            CombatData { damage, lifegain }
        }
        if !attacker.can_be_in_combat()
            || blockers.iter().any(|c| !c.can_be_in_combat())
            || attacker.get_amount() > 1
        {
            return None;
        }
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
        Some(Self {
            damage: first.damage + normal.damage,
            lifegain: first.lifegain + normal.lifegain,
        })
    }
}
