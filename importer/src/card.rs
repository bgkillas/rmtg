use crate::id::Id;
use bevy::asset::Handle;
use bevy::image::Image;
use bevy::pbr::StandardMaterial;
use bevy::ui::widget::ImageNode;
use bitcode::{Decode, Encode};
use enumset::{EnumSet, EnumSetType};
use std::cmp::Ordering;
use std::mem;
use std::slice::{Iter, IterMut};
rules::generate_types!();
type Value = f64;
#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct Card {
    pub subcard: SubCard,
    pub equiped: Vec<SubCard>,
    pub power: Option<Value>,
    pub toughness: Option<Value>,
    pub counters: Option<Value>,
    pub loyalty: Option<Value>,
    pub misc: Option<Value>,
    pub is_token: bool,
}
#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct SubCard {
    pub id: Id,
    pub tokens: Vec<Id>,
    pub data: CardData,
    pub flipped: bool,
}
#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct CardData {
    pub face: CardInfo,
    pub back: Option<Box<CardInfo>>,
    pub layout: Layout,
}
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
pub enum Layout {
    #[default]
    Normal,
    Flip,
    Room,
}
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
pub struct Cost {
    pub white: u8,
    pub blue: u8,
    pub black: u8,
    pub red: u8,
    pub green: u8,
    pub colorless: u8,
    pub any: u8,
    pub pay: u8,
    pub var: u8,
}
#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct CardInfo {
    pub name: String,
    pub mana_cost: Cost,
    pub type_line: Types,
    pub oracle_text: String,
    pub colors: Colors,
    pub color_identity: Colors,
    pub power: Option<u8>,
    pub toughness: Option<u8>,
    pub loyalty: Option<u8>,
    #[bitcode(skip)]
    pub image: MaybeImage,
}
#[derive(Debug, Clone, Default)]
pub struct MaybeImage {
    pub image: Option<Handle<Image>>,
}
#[derive(Debug, Default, Clone, PartialOrd, Encode, Decode, Eq, PartialEq)]
pub struct Types {
    pub super_type: SuperTypes,
    pub main_type: MainTypes,
    pub sub_type: SubTypes,
}
#[derive(Debug, Default, Clone, PartialOrd, Encode, Decode, Eq, PartialEq)]
pub struct SuperTypes {
    #[bitcode(with = "SuperTypesCoder")]
    pub types: EnumSet<SuperType>,
}
#[derive(Debug, Default, Clone, PartialOrd, Encode, Decode, Eq, PartialEq)]
pub struct MainTypes {
    #[bitcode(with = "MainTypesCoder")]
    pub types: EnumSet<MainType>,
}
#[derive(Debug, Default, Clone, PartialOrd, Encode, Decode, Eq, PartialEq)]
pub struct SubTypes {
    #[bitcode(with = "SubTypesCoder")]
    pub types: EnumSet<SubType>,
}
#[derive(Debug, Clone, Encode, Decode, Eq, PartialEq)]
#[repr(transparent)]
struct SuperTypesCoder {
    bytes: [u8; size_of::<EnumSet<SuperType>>()],
}
#[derive(Debug, Clone, Encode, Decode, Eq, PartialEq)]
#[repr(transparent)]
struct MainTypesCoder {
    bytes: [u8; size_of::<EnumSet<MainType>>()],
}
#[derive(Debug, Clone, Encode, Decode, Eq, PartialEq)]
#[repr(transparent)]
struct SubTypesCoder {
    bytes: [u8; size_of::<EnumSet<SubType>>()],
}
#[derive(Debug, Default, Clone, Copy, PartialOrd, Encode, Decode, PartialEq)]
pub struct Colors {
    #[bitcode(with = "ColorsCoder")]
    pub colors: EnumSet<Color>,
}
#[derive(Debug, Default, Clone, Copy, Encode, Decode, PartialEq)]
#[repr(transparent)]
struct ColorsCoder {
    bytes: [u8; size_of::<EnumSet<Color>>()],
}
#[derive(Debug, Encode, Decode, EnumSetType)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}
pub struct CardIter<'a> {
    pub subcard: &'a SubCard,
    pub equiped: Iter<'a, SubCard>,
    pub started: bool,
}
pub struct CardIterMut<'a> {
    pub subcard: *mut SubCard,
    pub equiped: IterMut<'a, SubCard>,
    pub started: bool,
}
#[derive(Debug)]
pub enum Order {
    Greater,
    Less,
    Equal,
    GreaterEqual,
    LessEqual,
}
#[derive(Debug, Clone, Copy)]
pub enum SearchKey {
    Name,
    Cmc,
    Type,
    SuperType,
    MainType,
    SubType,
    Text,
    Color,
    Identity,
    Power,
    Toughness,
    Loyalty,
}
impl From<SuperTypesCoder> for EnumSet<SuperType> {
    fn from(value: SuperTypesCoder) -> Self {
        unsafe { mem::transmute(value) }
    }
}
impl From<&EnumSet<SuperType>> for SuperTypesCoder {
    fn from(value: &EnumSet<SuperType>) -> Self {
        unsafe { mem::transmute(*value) }
    }
}
impl From<MainTypesCoder> for EnumSet<MainType> {
    fn from(value: MainTypesCoder) -> Self {
        unsafe { mem::transmute(value) }
    }
}
impl From<&EnumSet<MainType>> for MainTypesCoder {
    fn from(value: &EnumSet<MainType>) -> Self {
        unsafe { mem::transmute(*value) }
    }
}
impl From<SubTypesCoder> for EnumSet<SubType> {
    fn from(value: SubTypesCoder) -> Self {
        unsafe { mem::transmute(value) }
    }
}
impl From<&EnumSet<SubType>> for SubTypesCoder {
    fn from(value: &EnumSet<SubType>) -> Self {
        unsafe { mem::transmute(*value) }
    }
}
impl From<ColorsCoder> for EnumSet<Color> {
    fn from(value: ColorsCoder) -> Self {
        unsafe { mem::transmute(value) }
    }
}
impl From<&EnumSet<Color>> for ColorsCoder {
    fn from(value: &EnumSet<Color>) -> Self {
        unsafe { mem::transmute(*value) }
    }
}
impl CardInfo {
    #[must_use]
    pub fn clone_no_image(&self) -> Self {
        Self {
            name: self.name.clone(),
            mana_cost: self.mana_cost,
            type_line: self.type_line.clone(),
            oracle_text: self.oracle_text.clone(),
            colors: self.colors,
            color_identity: self.color_identity,
            power: self.power,
            loyalty: self.loyalty,
            toughness: self.toughness,
            image: MaybeImage::default(),
        }
    }
    #[must_use]
    pub fn clone_image(&self) -> Handle<Image> {
        self.image.clone_handle()
    }
}
impl From<Handle<Image>> for MaybeImage {
    fn from(value: Handle<Image>) -> Self {
        Self { image: Some(value) }
    }
}
impl MaybeImage {
    #[must_use]
    pub fn clone_handle(&self) -> Handle<Image> {
        self.handle().clone()
    }
    #[must_use]
    pub fn handle(&self) -> &Handle<Image> {
        self.image.as_ref().unwrap()
    }
}
impl MainType {
    #[must_use]
    pub fn is_permanent(self) -> bool {
        !matches!(self, Self::Instant | Self::Sorcery)
    }
}
impl Types {
    #[must_use]
    pub fn len(&self) -> usize {
        self.super_type.types.len() + self.main_type.types.len() + self.sub_type.types.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.super_type.types.is_empty()
            && self.main_type.types.is_empty()
            && self.sub_type.types.is_empty()
    }
    #[must_use]
    pub fn is_permanent(&self) -> bool {
        self.main_type.types.iter().any(MainType::is_permanent)
    }
}
impl From<&str> for Types {
    fn from(s: &str) -> Self {
        let mut ret = Self::default();
        for word in s.split(' ') {
            if let Ok(super_type) = SuperType::try_from(word) {
                ret.super_type.types.insert(super_type);
            } else if let Ok(ty) = MainType::try_from(word) {
                ret.main_type.types.insert(ty);
            } else if let Ok(sub_type) = SubType::try_from(word) {
                ret.sub_type.types.insert(sub_type);
            }
        }
        ret
    }
}
impl From<&str> for SuperTypes {
    fn from(s: &str) -> Self {
        let mut ret = Self::default();
        for word in s.split(' ') {
            if let Ok(super_type) = SuperType::try_from(word) {
                ret.types.insert(super_type);
            }
        }
        ret
    }
}
impl From<&str> for MainTypes {
    fn from(s: &str) -> Self {
        let mut ret = Self::default();
        for word in s.split(' ') {
            if let Ok(main_type) = MainType::try_from(word) {
                ret.types.insert(main_type);
            }
        }
        ret
    }
}
impl From<&str> for SubTypes {
    fn from(s: &str) -> Self {
        let mut ret = Self::default();
        for word in s.split(' ') {
            if let Ok(sub_type) = SubType::try_from(word) {
                ret.types.insert(sub_type);
            }
        }
        ret
    }
}
impl TryFrom<&str> for Colors {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut cost = Self::default();
        for c in value.chars() {
            match c {
                'w' => {
                    cost.colors.insert(Color::White);
                }
                'u' => {
                    cost.colors.insert(Color::Blue);
                }
                'b' => {
                    cost.colors.insert(Color::Black);
                }
                'r' => {
                    cost.colors.insert(Color::Red);
                }
                'g' => {
                    cost.colors.insert(Color::Green);
                }
                _ => return Err(()),
            }
        }
        Ok(cost)
    }
}
impl Colors {
    pub fn parse<'a>(value: impl Iterator<Item = &'a str>) -> Self {
        let mut cost = Self::default();
        for c in value {
            match c {
                "W" => {
                    cost.colors.insert(Color::White);
                }
                "U" => {
                    cost.colors.insert(Color::Blue);
                }
                "B" => {
                    cost.colors.insert(Color::Black);
                }
                "R" => {
                    cost.colors.insert(Color::Red);
                }
                "G" => {
                    cost.colors.insert(Color::Green);
                }
                _ => unreachable!(),
            }
        }
        cost
    }
}
impl Colors {
    #[must_use]
    pub fn len(&self) -> usize {
        self.colors.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }
}
impl From<&str> for Cost {
    fn from(value: &str) -> Self {
        let mut cost = Self::default();
        if value.is_empty() {
            return cost;
        }
        let valueinner = &value[1..value.len() - 1];
        for c in valueinner.split("}{") {
            for c in c.split('/') {
                match c {
                    "W" => cost.white += 1,
                    "U" => cost.blue += 1,
                    "B" => cost.black += 1,
                    "R" => cost.red += 1,
                    "G" => cost.green += 1,
                    "C" => cost.colorless += 1,
                    "P" => cost.pay += 1,
                    "X" => cost.var += 1,
                    c => cost.any += c.parse::<u8>().unwrap(),
                }
            }
        }
        cost
    }
}
impl Cost {
    #[must_use]
    pub fn total(&self) -> u8 {
        self.white + self.blue + self.black + self.red + self.green + self.colorless + self.any
    }
}
impl CardData {
    #[must_use]
    pub fn clone_no_image(&self) -> Self {
        Self {
            face: self.face.clone_no_image(),
            back: self
                .back
                .as_ref()
                .map(|c| CardInfo::clone_no_image(c).into()),
            layout: self.layout,
        }
    }
}
impl Card {
    #[must_use]
    pub fn is_modified(&self) -> bool {
        !self.equiped.is_empty() || self.has_counters()
    }
    #[must_use]
    pub fn has_counters(&self) -> bool {
        self.power.is_some()
            || self.toughness.is_some()
            || self.counters.is_some()
            || self.loyalty.is_some()
            || self.misc.is_some()
    }
    #[must_use]
    pub fn clone_no_image(&self) -> Self {
        Self {
            subcard: self.subcard.clone_no_image(),
            equiped: self.equiped.iter().map(SubCard::clone_no_image).collect(),
            power: None,
            toughness: None,
            counters: None,
            loyalty: None,
            misc: None,
            is_token: false,
        }
    }
    #[must_use]
    pub fn filter(&self, text: &str) -> bool {
        self.subcard.filter(text)
    }
    #[must_use]
    pub fn flatten(mut self) -> Vec<SubCard> {
        let mut vec = Vec::with_capacity(self.equiped.len() + 1);
        let drain = mem::take(&mut self.equiped);
        vec.extend(drain);
        vec.push(self.subcard);
        vec
    }
    #[must_use]
    pub fn iter(&self) -> CardIter<'_> {
        CardIter {
            subcard: &self.subcard,
            equiped: self.equiped.iter(),
            started: false,
        }
    }
    #[must_use]
    pub fn iter_mut(&mut self) -> CardIterMut<'_> {
        CardIterMut {
            subcard: &raw mut self.subcard,
            equiped: self.equiped.iter_mut(),
            started: false,
        }
    }
    #[must_use]
    pub fn get(&self, idx: usize) -> Option<&SubCard> {
        if idx == 0 {
            Some(&self.subcard)
        } else {
            self.equiped.get(idx - 1)
        }
    }
    #[must_use]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut SubCard> {
        if idx == 0 {
            Some(&mut self.subcard)
        } else {
            self.equiped.get_mut(idx - 1)
        }
    }
}
impl<'a> IntoIterator for &'a Card {
    type Item = &'a SubCard;
    type IntoIter = CardIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a> IntoIterator for &'a mut Card {
    type Item = &'a mut SubCard;
    type IntoIter = CardIterMut<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
impl<'a> Iterator for CardIter<'a> {
    type Item = &'a SubCard;
    fn next(&mut self) -> Option<Self::Item> {
        if self.started {
            self.equiped.next()
        } else {
            self.started = true;
            Some(self.subcard)
        }
    }
}
impl DoubleEndedIterator for CardIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let back = self.equiped.next_back();
        if back.is_some() {
            back
        } else if self.started {
            None
        } else {
            self.started = true;
            Some(self.subcard)
        }
    }
}
impl ExactSizeIterator for CardIter<'_> {
    fn len(&self) -> usize {
        1 + self.equiped.len()
    }
}
impl ExactSizeIterator for CardIterMut<'_> {
    fn len(&self) -> usize {
        1 + self.equiped.len()
    }
}
impl<'a> Iterator for CardIterMut<'a> {
    type Item = &'a mut SubCard;
    fn next(&mut self) -> Option<Self::Item> {
        if self.started {
            self.equiped.next()
        } else {
            self.started = true;
            unsafe { self.subcard.as_mut() }
        }
    }
}
impl DoubleEndedIterator for CardIterMut<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let back = self.equiped.next_back();
        if back.is_some() {
            back
        } else if self.started {
            None
        } else {
            self.started = true;
            unsafe { self.subcard.as_mut() }
        }
    }
}
impl SubCard {
    #[must_use]
    pub fn clone_no_image(&self) -> Self {
        Self {
            id: self.id,
            tokens: self.tokens.clone(),
            data: self.data.clone_no_image(),
            flipped: self.flipped,
        }
    }
    #[must_use]
    pub fn filter(&self, text: &str) -> bool {
        self.data.face.filter(text) || self.data.back.as_ref().is_some_and(|c| c.filter(text))
    }
    #[must_use]
    pub fn face(&self) -> &CardInfo {
        if self.flipped {
            self.data.back.as_ref().unwrap()
        } else {
            &self.data.face
        }
    }
    #[must_use]
    pub fn back(&self) -> Option<&CardInfo> {
        if self.flipped {
            Some(&self.data.face)
        } else {
            self.data.back.as_deref()
        }
    }
    #[must_use]
    pub fn image_node(&self) -> ImageNode {
        if matches!(self.data.layout, Layout::Flip) && self.flipped {
            ImageNode {
                image: self.data.face.clone_image(),
                flip_x: true,
                flip_y: true,
                ..ImageNode::default()
            }
        } else {
            ImageNode::new(self.face().clone_image())
        }
    }
    #[must_use]
    pub fn material(&self) -> StandardMaterial {
        if matches!(self.data.layout, Layout::Flip) && self.flipped {
            StandardMaterial {
                base_color_texture: Some(self.data.face.clone_image()),
                unlit: true,
                uv_transform: StandardMaterial::FLIP_VERTICAL * StandardMaterial::FLIP_HORIZONTAL,
                ..StandardMaterial::default()
            }
        } else {
            StandardMaterial {
                base_color_texture: Some(self.face().clone_image()),
                unlit: true,
                ..StandardMaterial::default()
            }
        }
    }
}
impl CardInfo {
    #[must_use]
    pub fn filter(&self, text: &str) -> bool {
        let textlower = text.to_ascii_lowercase();
        let texttrim = textlower.trim();
        let pairs = get_pairs(texttrim);
        pairs
            .into_iter()
            .all(|(n, k, v, o)| self.filter_pair(n, k, v, o))
    }
    #[must_use]
    pub fn filter_pair(&self, negate: bool, key: SearchKey, value: &str, ordering: Order) -> bool {
        let res = match key {
            SearchKey::Name => self.name.to_ascii_lowercase().contains(value),
            SearchKey::Cmc => {
                if let Ok(v) = value.parse() {
                    self.mana_cost.total().cmp(&v) == ordering
                } else {
                    return false;
                }
            }
            SearchKey::Type => {
                if let Ok(count) = value.parse::<usize>() {
                    self.type_line.len() == count
                } else if let Some(order) = self.type_line.partial_cmp(&Types::from(value)) {
                    order == ordering
                } else {
                    return false;
                }
            }
            SearchKey::SuperType => {
                if let Ok(count) = value.parse::<usize>() {
                    self.type_line.super_type.types.len() == count
                } else if let Some(order) = self
                    .type_line
                    .super_type
                    .partial_cmp(&SuperTypes::from(value))
                {
                    order == ordering
                } else {
                    return false;
                }
            }
            SearchKey::MainType => {
                if let Ok(count) = value.parse::<usize>() {
                    self.type_line.main_type.types.len() == count
                } else if let Some(order) = self
                    .type_line
                    .main_type
                    .partial_cmp(&MainTypes::from(value))
                {
                    order == ordering
                } else {
                    return false;
                }
            }
            SearchKey::SubType => {
                if let Ok(count) = value.parse::<usize>() {
                    self.type_line.sub_type.types.len() == count
                } else if let Some(order) =
                    self.type_line.sub_type.partial_cmp(&SubTypes::from(value))
                {
                    order == ordering
                } else {
                    return false;
                }
            }
            SearchKey::Text => self.oracle_text.to_ascii_lowercase().contains(value),
            SearchKey::Color => {
                if let Ok(count) = value.parse::<usize>() {
                    self.colors.len() == count
                } else if let Ok(col) = Colors::try_from(value)
                    && let Some(order) = self.colors.partial_cmp(&col)
                {
                    order == ordering
                } else {
                    return false;
                }
            }
            SearchKey::Identity => {
                if let Ok(count) = value.parse::<usize>() {
                    self.color_identity.len() == count
                } else if let Ok(col) = Colors::try_from(value)
                    && let Some(order) = self.color_identity.partial_cmp(&col)
                {
                    order == ordering
                } else {
                    return false;
                }
            }
            SearchKey::Power => {
                if let Some(power) = self.power
                    && let Ok(v) = value.parse()
                {
                    power.cmp(&v) == ordering
                } else {
                    return false;
                }
            }
            SearchKey::Loyalty => {
                if let Some(loyalty) = self.loyalty
                    && let Ok(v) = value.parse()
                {
                    loyalty.cmp(&v) == ordering
                } else {
                    return false;
                }
            }
            SearchKey::Toughness => {
                if let Some(toughness) = self.toughness
                    && let Ok(v) = value.parse()
                {
                    toughness.cmp(&v) == ordering
                } else {
                    return false;
                }
            }
        };
        if negate { !res } else { res }
    }
}
impl PartialEq<Order> for Ordering {
    fn eq(&self, other: &Order) -> bool {
        match other {
            Order::Greater => matches!(self, Ordering::Greater),
            Order::Less => matches!(self, Ordering::Less),
            Order::Equal => matches!(self, Ordering::Equal),
            Order::GreaterEqual => matches!(self, Ordering::Greater | Ordering::Equal),
            Order::LessEqual => matches!(self, Ordering::Less | Ordering::Equal),
        }
    }
}
#[must_use]
fn get_pairs(text: &str) -> Vec<(bool, SearchKey, &str, Order)> {
    let mut quotes = false;
    let mut quoted = false;
    let mut order = None;
    let mut k = 0;
    let mut v = 0;
    let mut pairs = Vec::new();
    let mut key = None;
    let mut negate = false;
    for (i, c) in text.char_indices() {
        match c {
            '!' => negate = true,
            '\"' => {
                quoted = true;
                quotes = !quotes;
            }
            '=' if !quotes => {
                v = i + 1;
                if order.is_none() {
                    key = get_key(&text[if negate { k + 1 } else { k }..i]);
                    if key.is_some() {
                        order = Some(Order::Equal);
                    }
                } else if matches!(order, Some(Order::Greater)) {
                    order = Some(Order::GreaterEqual);
                } else if matches!(order, Some(Order::Less)) {
                    order = Some(Order::LessEqual);
                }
            }
            '<' if !quotes => {
                v = i + 1;
                if order.is_none() {
                    key = get_key(&text[if negate { k + 1 } else { k }..i]);
                    if key.is_some() {
                        order = Some(Order::Less);
                    }
                }
            }
            '>' if !quotes => {
                v = i + 1;
                if order.is_none() {
                    key = get_key(&text[if negate { k + 1 } else { k }..i]);
                    if key.is_some() {
                        order = Some(Order::Greater);
                    }
                }
            }
            ' ' if !quotes => {
                if let Some(order_inner) = order
                    && let Some(key_inner) = key
                {
                    pairs.push((
                        negate,
                        key_inner,
                        if quoted {
                            &text[v + 1..i - 1]
                        } else {
                            &text[v..i]
                        },
                        order_inner,
                    ));
                    k = i + 1;
                }
                order = None;
                quoted = false;
                negate = false;
            }
            _ => {}
        }
    }
    if let Some(order_inner) = order
        && let Some(key_inner) = key
    {
        pairs.push((
            negate,
            key_inner,
            if quoted {
                &text[v + 1..text.len() - 1]
            } else {
                &text[v..]
            },
            order_inner,
        ));
    } else {
        pairs.push((false, SearchKey::Name, &text[k..], Order::Equal));
    }
    pairs
}
fn get_key(key: &str) -> Option<SearchKey> {
    Some(match key {
        "name" | "n" => SearchKey::Name,
        "cmc" | "cost" => SearchKey::Cmc,
        "type" | "t" => SearchKey::Type,
        "super_type" | "ut" => SearchKey::SuperType,
        "main_type" | "mt" => SearchKey::MainType,
        "sub_type" | "st" => SearchKey::SubType,
        "text" | "o" => SearchKey::Text,
        "color" | "c" => SearchKey::Color,
        "power" | "p" => SearchKey::Power,
        "loyalty" | "l" => SearchKey::Loyalty,
        "toughness" | "h" => SearchKey::Toughness,
        _ => return None,
    })
}
impl From<SubCard> for Card {
    fn from(subcard: SubCard) -> Self {
        Self {
            subcard,
            ..Card::default()
        }
    }
}
