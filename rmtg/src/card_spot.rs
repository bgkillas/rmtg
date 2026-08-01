use bevy::prelude::{Component, Entity};
#[derive(Debug, PartialEq, Clone)]
pub enum SpotType {
    CommanderMain,
    CommanderAlt,
    Exile,
    Main,
    Graveyard,
}
#[derive(Component, Clone, Debug)]
pub struct CardSpot {
    pub spot_type: SpotType,
    pub ent: Option<Entity>,
}
impl CardSpot {
    #[must_use]
    pub fn new(spot_type: SpotType) -> Self {
        Self {
            spot_type,
            ent: None,
        }
    }
}
