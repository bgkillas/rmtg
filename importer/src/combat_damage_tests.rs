use crate::card::{KeyWord, SubCard};
use crate::combat_damage::CombatData;
use std::num::NonZero;
#[test]
pub fn test_combat_damage() {
    let mut attacker = SubCard::default();
    attacker.attributes.power = Some(8);
    attacker.attributes.toughness = Some(8);
    let mut blocker1 = SubCard::default();
    blocker1.attributes.power = Some(1);
    blocker1.attributes.toughness = Some(1);
    let mut blocker2 = SubCard::default();
    blocker2.attributes.power = Some(2);
    blocker2.attributes.toughness = Some(2);
    let mut blocker3 = SubCard::default();
    blocker3.attributes.power = Some(3);
    blocker3.attributes.toughness = Some(3);
    assert_eq!(CombatData::get(&attacker, &[]), Some(CombatData::new(8, 0)));
    *attacker.get_mut(KeyWord::Lifelink) = NonZero::new(1);
    assert_eq!(CombatData::get(&attacker, &[]), Some(CombatData::new(8, 8)));
    assert_eq!(
        CombatData::get(&attacker, &[&blocker1]),
        Some(CombatData::new(0, 8))
    );
    assert_eq!(
        CombatData::get(&attacker, &[&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(0, 8))
    );
    *attacker.get_mut(KeyWord::Trample) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &[&blocker1]),
        Some(CombatData::new(7, 8))
    );
    assert_eq!(
        CombatData::get(&attacker, &[&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(2, 8))
    );
    *attacker.get_mut(KeyWord::Deathtouch) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &[&blocker1]),
        Some(CombatData::new(7, 8))
    );
    assert_eq!(
        CombatData::get(&attacker, &[&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(5, 8))
    );
    *attacker.get_mut(KeyWord::DoubleStrike) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &[&blocker1]),
        Some(CombatData::new(15, 16))
    );
    assert_eq!(
        CombatData::get(&attacker, &[&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(13, 16))
    );
    *blocker2.get_mut(KeyWord::Indestructible) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &[&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(12, 16))
    );
    *blocker3.get_mut(KeyWord::Deathtouch) = NonZero::new(1);
    *blocker3.get_mut(KeyWord::FirstStrike) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &[&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(5, 8))
    );
    *attacker.get_mut(KeyWord::Indestructible) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &[&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(12, 16))
    );
}
