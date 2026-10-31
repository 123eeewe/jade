//! Модели данных игрового мира: сущности, блоки, NBT.
//!
//! В реальном моде эти данные приходят из API Minecraft (Fabric/Forge).
//! Здесь они смоделированы так, чтобы вся логика Jade (компоненты
//! наведения, приоритеты, сортировка) работала один-в-один.

use std::collections::HashMap;

/// Упрощённый аналог NBT-тега (String / Int / Float / Compound / List).
#[derive(Debug, Clone, PartialEq)]
pub enum Nbt {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Compound(HashMap<String, Nbt>),
    List(Vec<Nbt>),
}

impl Nbt {
    pub fn compound(pairs: &[(&str, Nbt)]) -> Nbt {
        let mut m = HashMap::new();
        for (k, v) in pairs {
            m.insert(k.to_string(), v.clone());
        }
        Nbt::Compound(m)
    }

    pub fn get(&self, key: &str) -> Option<&Nbt> {
        match self {
            Nbt::Compound(m) => m.get(key),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        if let Nbt::Int(v) = self { Some(*v) } else { None }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Nbt::Float(v) => Some(*v),
            Nbt::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Nbt::Bool(v) = self { Some(*v) } else { None }
    }

    /// Рекурсивно ищет тег по пути "A.B.C" (аналог Path в NBT API).
    pub fn path(&self, path: &str) -> Option<&Nbt> {
        let mut cur = self;
        for seg in path.split('.') {
            cur = cur.get(seg)?;
        }
        Some(cur)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }
    pub fn distance(&self, o: &Vec3) -> f64 {
        ((self.x - o.x).powi(2) + (self.y - o.y).powi(2) + (self.z - o.z).powi(2)).sqrt()
    }
}

/// Тип цели наведения (аналог `WailaTargetType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    Entity,
    Block,
}

/// Сущность, на которую смотрит игрок.
#[derive(Debug, Clone)]
pub struct EntityData {
    /// Идентификатор типа, напр. "minecraft:cow".
    pub id: String,
    pub display_name: String,
    pub health: f32,
    pub max_health: f32,
    pub armor: f32,
    pub is_baby: bool,
    pub is_tamed: bool,
    pub owner_name: Option<String>,
    /// Доп. NBT-данные (кольты, предметы в руке и т.п.).
    pub nbt: Nbt,
    pub pos: Vec3,
}

/// Блок, на который смотрит игрок.
#[derive(Debug, Clone)]
pub struct BlockData {
    /// Идентификатор блока, напр. "minecraft:chest".
    pub id: String,
    pub display_name: String,
    /// Прочность/полезность блока для предмета в руке (0..1).
    pub hardness: f32,
    pub block_state: HashMap<String, String>,
    /// Contents для контейнеров (сундуки, печки...).
    pub inventory: Vec<ItemStack>,
    pub nbt: Nbt,
    pub pos: (i32, i32, i32),
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub id: String,
    pub name: String,
    pub count: u32,
}

/// Предмет в руке игрока.
#[derive(Debug, Clone)]
pub struct HeldItem {
    pub id: String,
    pub name: String,
    /// Эффективность по данному блоку (1.0 = идеально, 0.0 = не может сломать).
    pub efficiency: f32,
}

/// Единая цель наведения.
#[derive(Debug, Clone)]
pub enum Target {
    Entity(EntityData),
    Block(BlockData),
}

impl Target {
    pub fn kind(&self) -> TargetKind {
        match self {
            Target::Entity(_) => TargetKind::Entity,
            Target::Block(_) => TargetKind::Block,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Target::Entity(e) => &e.id,
            Target::Block(b) => &b.id,
        }
    }
}

/// Контекст выполнения компонента (аналог `WailaContext`).
pub struct ProbeContext<'a> {
    pub target: &'a Target,
    pub player_pos: Vec3,
    pub held_item: Option<HeldItem>,
    /// Sneaking — переключает режим отображения (как в Jade).
    pub sneaking: bool,
    /// Creative mode — показывает ID-хелперы.
    pub creative: bool,
}

impl<'a> ProbeContext<'a> {
    pub fn entity(&self) -> Option<&'a EntityData> {
        if let Target::Entity(e) = self.target { Some(e) } else { None }
    }
    pub fn block(&self) -> Option<&'a BlockData> {
        if let Target::Block(b) = self.target { Some(b) } else { None }
    }
}
