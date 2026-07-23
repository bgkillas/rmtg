use crate::assets::Asset;
use crate::physics::physics_base;
use avian3d::prelude::Collider;
use bevy::ecs::children;
use bevy::mesh::Mesh3d;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::{Bundle, Component, Transform};
use bitcode::{Decode, Encode};
use importer::card::{Card, CardIter, CardIterMut, SubCard};
use importer::is_reversed;
use itertools::Either;
use rand::make_rng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom as _;
use std::cmp::Ordering;
use std::ops::{Bound, RangeBounds};
use std::slice::{Iter, IterMut};
use std::{iter, mem};
#[derive(Component, Default, Clone, Debug, Encode, Decode)]
pub enum Pile {
    Multiple(Vec<SubCard>),
    Single(Box<Card>),
    #[default]
    Empty,
}
impl Pile {
    #[must_use]
    pub fn bundle(self, asset: &mut Asset) -> impl Bundle {
        (
            children![
                self.left(asset),
                self.right(asset),
                self.up(asset),
                self.down(asset),
                self.front(asset),
                self.back(asset),
            ],
            self.collider(),
            self,
            physics_base(),
        )
    }
    #[must_use]
    pub fn left(&self, asset: &Asset) -> impl Bundle + use<> {
        (
            Transform::default(),
            MeshMaterial3d(asset.card.color.clone()),
        )
    }
    #[must_use]
    pub fn right(&self, asset: &Asset) -> impl Bundle + use<> {
        (
            Transform::default(),
            MeshMaterial3d(asset.card.color.clone()),
        )
    }
    #[must_use]
    pub fn up(&self, asset: &mut Asset) -> impl Bundle + use<> {
        (
            Transform::default(),
            MeshMaterial3d(self.first().face().material()),
            Mesh3d(asset.card.stock.clone()),
        )
    }
    #[must_use]
    pub fn down(&self, asset: &Asset) -> impl Bundle + use<> {
        (
            Transform::default(),
            MeshMaterial3d(asset.card.back.clone()),
            Mesh3d(asset.card.stock.clone()),
        )
    }
    #[must_use]
    pub fn front(&self, asset: &Asset) -> impl Bundle + use<> {
        (
            Transform::default(),
            MeshMaterial3d(asset.card.color.clone()),
        )
    }
    #[must_use]
    pub fn back(&self, asset: &Asset) -> impl Bundle + use<> {
        (
            Transform::default(),
            MeshMaterial3d(asset.card.color.clone()),
        )
    }
    #[must_use]
    pub fn collider(&self) -> Collider {
        todo!()
    }
    pub fn sort_by<F>(&mut self, sort: F)
    where
        F: FnMut(&SubCard, &SubCard) -> Ordering,
    {
        if let Pile::Multiple(v) = self {
            v.sort_by(sort);
        }
    }
    #[must_use]
    pub fn new(mut v: Vec<SubCard>) -> Self {
        if v.len() == 1 {
            Self::Single(Box::new(Card::from(v.remove(0))))
        } else {
            Self::Multiple(v)
        }
    }
    #[allow(clippy::must_use_candidate)]
    pub fn equip(&mut self) -> bool {
        match self {
            s @ Pile::Multiple(_) => {
                let subcard = s.pop();
                let Pile::Multiple(equiped) = mem::take(s) else {
                    unreachable!();
                };
                *s = Pile::Single(Box::new(Card {
                    subcard,
                    equiped,
                    power: None,
                    toughness: None,
                    counters: None,
                    loyalty: None,
                    misc: None,
                    is_token: false,
                }));
                true
            }
            s @ Pile::Single(_) => {
                if let Pile::Single(c) = &s
                    && !c.equiped.is_empty()
                {
                    let Pile::Single(cards) = mem::take(s) else {
                        unreachable!();
                    };
                    *s = Pile::Multiple(cards.flatten());
                }
                false
            }
            Pile::Empty => {
                unreachable!()
            }
        }
    }
    #[must_use]
    pub fn is_equiped(&self) -> bool {
        if let Pile::Single(s) = self {
            !s.equiped.is_empty()
        } else {
            false
        }
    }
    #[must_use]
    pub fn is_modified(&self) -> bool {
        if let Pile::Single(s) = self {
            s.is_modified()
        } else {
            false
        }
    }
    #[must_use]
    pub fn has_counters(&self) -> bool {
        if let Pile::Single(s) = self {
            s.has_counters()
        } else {
            false
        }
    }
    pub fn merge(&mut self, to: Self) {
        let Pile::Single(mut top) = to else {
            unreachable!()
        };
        if !self.is_equiped() {
            self.equip();
        }
        let Pile::Single(s) = self else {
            unreachable!()
        };
        mem::swap(s, &mut top);
        s.equiped.splice(0..0, top.flatten());
    }
    #[must_use]
    pub fn clone_no_image(&self) -> Self {
        match self {
            Pile::Multiple(v) => Pile::Multiple(v.iter().map(SubCard::clone_no_image).collect()),
            Pile::Single(s) => Pile::Single(s.clone_no_image().into()),
            Pile::Empty => Pile::Empty,
        }
    }
    #[must_use]
    pub fn get_card(&self, transform: &Transform) -> &SubCard {
        if is_reversed(transform) {
            self.first()
        } else {
            self.last()
        }
    }
    #[must_use]
    pub fn get_mut_card(&mut self, transform: &Transform) -> &mut SubCard {
        if is_reversed(transform) {
            self.first_mut()
        } else {
            self.last_mut()
        }
    }
    #[must_use]
    pub fn get(&self, idx: usize) -> Option<&SubCard> {
        match self {
            Pile::Multiple(v) => v.get(idx),
            Pile::Single(s) => s.get(idx),
            Pile::Empty => unreachable!(),
        }
    }
    #[must_use]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut SubCard> {
        match self {
            Pile::Multiple(v) => v.get_mut(idx),
            Pile::Single(s) => s.get_mut(idx),
            Pile::Empty => unreachable!(),
        }
    }
    pub fn set_single(&mut self) {
        if self.len() == 1 {
            *self = Pile::Multiple(vec![self.pop()]);
        }
    }
    #[must_use]
    pub fn take_card(&mut self, transform: &Transform) -> SubCard {
        let ret = if is_reversed(transform) {
            self.remove(0)
        } else {
            self.pop()
        };
        self.set_single();
        ret
    }
    #[must_use]
    pub fn take_n_card(&mut self, transform: &Transform, n: usize) -> Vec<SubCard> {
        let ret = if is_reversed(transform) {
            self.drain(0..n.min(self.len())).collect()
        } else {
            self.drain(self.len().saturating_sub(n)..self.len())
                .rev()
                .collect()
        };
        self.set_single();
        ret
    }
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Pile::Multiple(v) => v.len(),
            Pile::Single(_) => 1,
            Pile::Empty => 0,
        }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Pile::Multiple(v) => v.is_empty(),
            Pile::Single(_) => false,
            Pile::Empty => true,
        }
    }
    #[must_use]
    pub fn last(&self) -> &SubCard {
        match self {
            Pile::Multiple(v) => v.last().unwrap(),
            Pile::Single(s) => &s.subcard,
            Pile::Empty => unreachable!(),
        }
    }
    #[must_use]
    pub fn pop(&mut self) -> SubCard {
        match self {
            Pile::Multiple(v) => {
                let ret = v.pop().unwrap();
                self.set_single();
                ret
            }
            se @ Pile::Single(_) => {
                let Pile::Single(s) = mem::take(se) else {
                    unreachable!()
                };
                s.subcard
            }
            Pile::Empty => unreachable!(),
        }
    }
    #[must_use]
    pub fn first(&self) -> &SubCard {
        match self {
            Pile::Multiple(v) => &v[0],
            Pile::Single(s) => &s.subcard,
            Pile::Empty => unreachable!(),
        }
    }
    #[must_use]
    pub fn last_mut(&mut self) -> &mut SubCard {
        match self {
            Pile::Multiple(v) => v.last_mut().unwrap(),
            Pile::Single(s) => &mut s.subcard,
            Pile::Empty => unreachable!(),
        }
    }
    #[must_use]
    pub fn first_mut(&mut self) -> &mut SubCard {
        match self {
            Pile::Multiple(v) => &mut v[0],
            Pile::Single(s) => &mut s.subcard,
            Pile::Empty => unreachable!(),
        }
    }
    pub fn extend(&mut self, other: Self) {
        match (self, other) {
            (Pile::Multiple(a), Pile::Multiple(b)) => a.extend(b),
            (Pile::Multiple(a), Pile::Single(b)) => a.extend(b.flatten()),
            (se @ Pile::Single(_), o) => {
                let Pile::Single(s) = mem::take(se) else {
                    unreachable!()
                };
                let mut vec = s.flatten();
                match o {
                    Pile::Multiple(v) => vec.extend(v),
                    Pile::Single(s) => vec.extend(s.flatten()),
                    Pile::Empty => unreachable!(),
                }
                *se = Pile::Multiple(vec);
            }
            _ => unreachable!(),
        }
    }
    pub fn extend_start(&mut self, other: Self) {
        match (self, other) {
            (Pile::Multiple(a), Pile::Multiple(b)) => {
                a.splice(0..0, b);
            }
            (Pile::Multiple(a), Pile::Single(b)) => {
                a.splice(0..0, b.flatten());
            }
            (se @ Pile::Single(_), o) => {
                let Pile::Single(s) = mem::take(se) else {
                    unreachable!()
                };
                let mut vec = s.flatten();
                match o {
                    Pile::Multiple(v) => vec.splice(0..0, v),
                    Pile::Single(s) => vec.splice(0..0, s.flatten()),
                    Pile::Empty => unreachable!(),
                };
                *se = Pile::Multiple(vec);
            }
            _ => unreachable!(),
        }
    }
    pub fn splice_at(&mut self, at: usize, other: Self) {
        match (self, other) {
            (Pile::Multiple(a), Pile::Multiple(b)) => {
                a.splice(at..at, b);
            }
            (Pile::Multiple(a), Pile::Single(b)) => {
                a.splice(at..at, b.flatten());
            }
            (se @ Pile::Single(_), o) => {
                let Pile::Single(s) = mem::take(se) else {
                    unreachable!()
                };
                let mut vec = s.flatten();
                match o {
                    Pile::Multiple(v) => vec.splice(at..at, v),
                    Pile::Single(s) => vec.splice(at..at, s.flatten()),
                    Pile::Empty => unreachable!(),
                };
                *se = Pile::Multiple(vec);
            }
            _ => unreachable!(),
        }
    }
    pub fn shuffle(&mut self) {
        if let Pile::Multiple(v) = self {
            v.shuffle(&mut make_rng::<StdRng>());
        }
    }
    #[must_use]
    pub fn remove(&mut self, n: usize) -> SubCard {
        match self {
            Pile::Multiple(v) => {
                let ret = v.remove(n);
                self.set_single();
                ret
            }
            se @ Pile::Single(_) => {
                let Pile::Single(s) = mem::take(se) else {
                    unreachable!()
                };
                s.subcard
            }
            Pile::Empty => unreachable!(),
        }
    }
    pub fn insert(&mut self, n: usize, card: SubCard) {
        match self {
            Pile::Multiple(v) => v.insert(n, card),
            se @ Pile::Single(_) => {
                let Pile::Single(s) = mem::take(se) else {
                    unreachable!()
                };
                let mut v = s.flatten();
                if n == 0 {
                    v.insert(0, card);
                } else if n == 1 {
                    v.push(card);
                } else {
                    panic!();
                }
                *se = Pile::Multiple(v);
            }
            Pile::Empty => unreachable!(),
        }
    }
    #[must_use]
    pub fn drain<R>(
        &mut self,
        range: R,
    ) -> Either<impl DoubleEndedIterator<Item = SubCard>, impl DoubleEndedIterator<Item = SubCard>>
    where
        R: RangeBounds<usize>,
    {
        match self {
            Pile::Multiple(v) => Either::Left(v.drain(range)),
            se @ Pile::Single(_) => {
                if matches!(range.start_bound(), Bound::Included(&0) | Bound::Unbounded)
                    && matches!(
                        range.end_bound(),
                        Bound::Included(&0) | Bound::Excluded(&1) | Bound::Unbounded
                    )
                {
                    let Pile::Single(s) = mem::take(se) else {
                        unreachable!()
                    };
                    Either::Right(iter::once(s.subcard))
                } else {
                    unreachable!()
                }
            }
            Pile::Empty => unreachable!(),
        }
    }
    #[must_use]
    pub fn iter(&self) -> Either<Iter<'_, SubCard>, CardIter<'_>> {
        match self {
            Pile::Multiple(v) => Either::Left(v.iter()),
            Pile::Single(s) => Either::Right(s.iter()),
            Pile::Empty => unreachable!(),
        }
    }
    pub fn iter_equipment(&self) -> Iter<'_, SubCard> {
        match self {
            Pile::Single(s) => s.equiped.iter(),
            Pile::Multiple(_) | Pile::Empty => unreachable!(),
        }
    }
    pub fn iter_mut(&mut self) -> Either<IterMut<'_, SubCard>, CardIterMut<'_>> {
        match self {
            Pile::Multiple(v) => Either::Left(v.iter_mut()),
            Pile::Single(s) => Either::Right(s.iter_mut()),
            Pile::Empty => unreachable!(),
        }
    }
}
impl<'a> IntoIterator for &'a Pile {
    type Item = &'a SubCard;
    type IntoIter = Either<Iter<'a, SubCard>, CardIter<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a> IntoIterator for &'a mut Pile {
    type Item = &'a mut SubCard;
    type IntoIter = Either<IterMut<'a, SubCard>, CardIterMut<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
