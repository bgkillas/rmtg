use crate::card::CardInfo;
use bevy::ecs::children;
use bevy::prelude::Bundle;
impl CardInfo {
    #[must_use]
    pub fn get_oracle(&self) -> impl Bundle + use<> {
        children![]
    }
}
