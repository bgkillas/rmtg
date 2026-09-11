use importer::card::SubCard;
pub fn get_delta<'a>(
    attacker: &SubCard,
    blockers: impl Iterator<Item = &'a SubCard>,
) -> (u64, u64) {
    let lifegain = 0;
    let damage = 0;
    for blocker in blockers {
        //TODO
        _ = attacker;
        _ = blocker;
    }
    (lifegain, damage)
}
