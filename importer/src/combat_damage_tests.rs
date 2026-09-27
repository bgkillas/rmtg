use crate::card::{KeyWord, SubCard};
use crate::combat_damage::CombatData;
use std::num::NonZero;
#[test]
pub fn test_combat_damage() {
    let mut attacker = SubCard::default();
    attacker.attributes.power = Some(8u8.into());
    attacker.attributes.toughness = Some(8u8.into());
    let mut blocker1 = SubCard::default();
    blocker1.attributes.power = Some(1u8.into());
    blocker1.attributes.toughness = Some(1u8.into());
    let mut blocker2 = SubCard::default();
    blocker2.attributes.power = Some(2u8.into());
    blocker2.attributes.toughness = Some(2u8.into());
    let mut blocker3 = SubCard::default();
    blocker3.attributes.power = Some(3u8.into());
    blocker3.attributes.toughness = Some(3u8.into());
    assert_eq!(
        CombatData::get(&attacker, &mut []),
        Some(CombatData::new(8, 0))
    );
    *attacker.get_mut(KeyWord::Lifelink) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &mut []),
        Some(CombatData::new(8, 8))
    );
    assert_eq!(
        CombatData::get(&attacker, &mut [&blocker1]),
        Some(CombatData::new(0, 8))
    );
    assert_eq!(
        CombatData::get(&attacker, &mut [&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(0, 8))
    );
    *attacker.get_mut(KeyWord::Trample) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &mut [&blocker1]),
        Some(CombatData::new(7, 8))
    );
    assert_eq!(
        CombatData::get(&attacker, &mut [&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(2, 8))
    );
    *attacker.get_mut(KeyWord::Deathtouch) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &mut [&blocker1]),
        Some(CombatData::new(7, 8))
    );
    assert_eq!(
        CombatData::get(&attacker, &mut [&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(5, 8))
    );
    *attacker.get_mut(KeyWord::DoubleStrike) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &mut [&blocker1]),
        Some(CombatData::new(15, 16))
    );
    assert_eq!(
        CombatData::get(&attacker, &mut [&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(13, 16))
    );
    *blocker2.get_mut(KeyWord::Indestructible) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &mut [&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(12, 16))
    );
    *blocker3.get_mut(KeyWord::Deathtouch) = NonZero::new(1);
    *blocker3.get_mut(KeyWord::FirstStrike) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &mut [&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(5, 8))
    );
    *attacker.get_mut(KeyWord::Indestructible) = NonZero::new(1);
    assert_eq!(
        CombatData::get(&attacker, &mut [&blocker1, &blocker2, &blocker3]),
        Some(CombatData::new(12, 16))
    );
}
