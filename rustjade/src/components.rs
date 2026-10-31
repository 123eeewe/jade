//! Встроенные компоненты — аналог пакета `snownee.jade.component.core`:
//! название, здоровье, броня, владелец питомца, кольты, сундуки,
//! печки, полярность редстоуна, прочность инструмента, координаты и т.д.

use crate::core::{Order, ProbeComponent, ProbeLine, ProbeTargetBuilder, Slot};
use crate::text::{line, Color, Component};
use crate::world::{ProbeContext, Target, TargetKind};

/// Общее: для всех типов целей.
fn all() -> Vec<TargetKind> {
    vec![TargetKind::Entity, TargetKind::Block]
}
fn entities() -> Vec<TargetKind> {
    vec![TargetKind::Entity]
}
fn blocks() -> Vec<TargetKind> {
    vec![TargetKind::Block]
}

// ---------------------------------------------------------------------------
// Здоровье (аналог jade:health)
// ---------------------------------------------------------------------------

pub struct HealthComponent;

impl ProbeComponent for HealthComponent {
    fn id(&self) -> &str { "rustjade:health" }
    fn display_name(&self) -> &str { "Здоровье" }
    fn targets(&self) -> Vec<TargetKind> { entities() }
    fn slot(&self) -> (Slot, Order) { (Slot::Body, 100) }

    fn generate(&self, ctx: &ProbeContext, out: &mut ProbeTargetBuilder) -> bool {
        let Some(e) = ctx.entity() else { return false };
        // Игроки/некоторые мобы скрыты как в Jade — здесь показываем всегда.
        let pct = if e.max_health > 0.0 { e.health / e.max_health } else { 0.0 };
        let color = if pct > 0.5 {
            Color::Green
        } else if pct > 0.25 {
            Color::Gold
        } else {
            Color::Red
        };
        let hearts = render_hearts(e.health, e.max_health);
        let c = line(
            Component::literal("Health").with_color(Color::Gray),
            Component::literal(format!("{:.1} / {:.1}", e.health, e.max_health)).with_color(color).child(hearts),
        );
        out.append(ProbeLine::new(self.id(), c));
        true
    }
}

/// Мини-полоска сердец из юникода: ♥ заполненные, ♡ пустые.
fn render_hearts(cur: f32, max: f32) -> Component {
    let full = cur.floor() as usize;
    let half = (cur - cur.floor()) >= 0.5;
    let total = (max.ceil() as usize + 1) / 2; // ~как сердца ванилы (2 hp за сердце)
    let mut s = String::new();
    for i in 0..total {
        if i < full {
            s.push('♥');
        } else if i == full && half {
            s.push('♥');
        } else {
            s.push('♡');
        }
    }
    Component::literal(s).with_color(Color::Red)
}

// ---------------------------------------------------------------------------
// Броня (аналог jade:armor)
// ---------------------------------------------------------------------------

pub struct ArmorComponent;

impl ProbeComponent for ArmorComponent {
    fn id(&self) -> &str { "rustjade:armor" }
    fn display_name(&self) -> &str { "Броня" }
    fn targets(&self) -> Vec<TargetKind> { entities() }
    fn slot(&self) -> (Slot, Order) { (Slot::Body, 110) }

    fn generate(&self, ctx: &ProbeContext, out: &mut ProbeTargetBuilder) -> bool {
        let Some(e) = ctx.entity() else { return false };
        if e.armor <= 0.0 {
            return false;
        }
        let c = line(
            Component::literal("Armor").with_color(Color::Gray),
            Component::literal(format!("{:.0}", e.armor)).with_color(Color::White),
        );
        out.append(ProbeLine::new(self.id(), c));
        true
    }
}

// ---------------------------------------------------------------------------
// Питомцы/владелец (аналог jade:pet_owner)
// ---------------------------------------------------------------------------

pub struct OwnerComponent;

impl ProbeComponent for OwnerComponent {
    fn id(&self) -> &str { "rustjade:owner" }
    fn display_name(&self) -> &str { "Владелец" }
    fn targets(&self) -> Vec<TargetKind> { entities() }
    fn slot(&self) -> (Slot, Order) { (Slot::AfterName, 0) }

    fn generate(&self, ctx: &ProbeContext, out: &mut ProbeTargetBuilder) -> bool {
        let Some(e) = ctx.entity() else { return false };
        if !e.is_tamed {
            return false;
        }
        let owner = e.owner_name.clone().unwrap_or_else(|| "???".into());
        let c = Component::literal(format!("Tamed by {owner}")).with_color(Color::LightPurple);
        out.add_after_name(ProbeLine::new(self.id(), c));
        true
    }
}

// ---------------------------------------------------------------------------
// Кольты у лошадей (аналог jade:horse_inventory)
// ---------------------------------------------------------------------------

pub struct HorseInventoryComponent;

impl ProbeComponent for HorseInventoryComponent {
    fn id(&self) -> &str { "rustjade:horse_inventory" }
    fn display_name(&self) -> &str { "Инвентарь лошади" }
    fn targets(&self) -> Vec<TargetKind> { entities() }
    fn slot(&self) -> (Slot, Order) { (Slot::Body, 300) }

    fn generate(&self, ctx: &ProbeContext, out: &mut ProbeTargetBuilder) -> bool {
        let Some(e) = ctx.entity() else { return false };
        let mut found = false;
        for key in ["ItemSlot", "SaddleItem"] {
            if let Some(slot) = e.nbt.path(&format!("HorseInventory.{key}")).or_else(|| e.nbt.get(key)) {
                if let Some(name) = slot.path("id").and_then(|n| match n {
                    crate::world::Nbt::Str(s) => Some(s.clone()),
                    _ => None,
                }) {
                    let count = slot.path("Count").and_then(|c| c.as_i64()).unwrap_or(1);
                    let c = Component::literal(format!("📦 {name} ×{count}"));
                    out.append(ProbeLine::new(self.id(), c).with_icon(&name));
                    found = true;
                }
            }
        }
        found
    }
}

// ---------------------------------------------------------------------------
// Содержимое контейнеров (аналог jade:contents)
// ---------------------------------------------------------------------------

pub struct ContentsComponent;

impl ProbeComponent for ContentsComponent {
    fn id(&self) -> &str { "rustjade:contents" }
    fn display_name(&self) -> &str { "Содержимое" }
    fn targets(&self) -> Vec<TargetKind> { blocks() }
    fn slot(&self) -> (Slot, Order) { (Slot::Body, 200) }

    fn generate(&self, ctx: &ProbeContext, out: &mut ProbeTargetBuilder) -> bool {
        let Some(b) = ctx.block() else { return false };
        if b.inventory.is_empty() {
            return false;
        }
        // Sneaking показывает сводку вместо полного списка (фича Jade).
        if ctx.sneaking {
            let total: u32 = b.inventory.iter().map(|i| i.count).sum();
            let kinds = b.inventory.len();
            let c = Component::literal(format!("{kinds} types, {total} items total"))
                .with_color(Color::Gray)
                .italic();
            out.append(ProbeLine::new(self.id(), c));
            return true;
        }
        let max_rows = 9;
        for item in b.inventory.iter().take(max_rows) {
            let c = Component::literal(format!("{} ×{}", item.name, item.count));
            out.append(ProbeLine::new(self.id(), c).with_icon(&item.id));
        }
        if b.inventory.len() > max_rows {
            let more = b.inventory.len() - max_rows;
            out.append(ProbeLine::new(
                self.id(),
                Component::literal(format!("...and {more} more")).with_color(Color::DarkGray),
            ));
        }
        true
    }
}

// ---------------------------------------------------------------------------
// Печки/топка (аналог jade:smelting)
// ---------------------------------------------------------------------------

pub struct SmeltingComponent;

impl ProbeComponent for SmeltingComponent {
    fn id(&self) -> &str { "rustjade:smelting" }
    fn display_name(&self) -> &str { "Плавка" }
    fn targets(&self) -> Vec<TargetKind> { blocks() }
    fn slot(&self) -> (Slot, Order) { (Slot::Body, 250) }

    fn generate(&self, ctx: &ProbeContext, out: &mut ProbeTargetBuilder) -> bool {
        let Some(b) = ctx.block() else { return false };
        let fuel = b.nbt.path("FuelBurnTimeTicks").and_then(|t| t.as_i64()).unwrap_or(0);
        let recipe = b.nbt.path("RecipesUsed");
        if fuel <= 0 && recipe.is_none() {
            return false;
        }
        if fuel > 0 {
            let secs = fuel as f64 / 20.0;
            let c = line(
                Component::literal("Burning").with_color(Color::Gray),
                Component::literal(format!("{secs:.1}s left")).with_color(Color::Gold),
            );
            out.append(ProbeLine::new(self.id(), c).with_icon("🔥"));
        }
        true
    }
}

// ---------------------------------------------------------------------------
// Редстоун-полярность (аналог jade:potion_effects / redstone)
// ---------------------------------------------------------------------------

pub struct RedstoneComponent;

impl ProbeComponent for RedstoneComponent {
    fn id(&self) -> &str { "rustjade:redstone" }
    fn display_name(&self) -> &str { "Редстоун" }
    fn targets(&self) -> Vec<TargetKind> { blocks() }
    fn slot(&self) -> (Slot, Order) { (Slot::Head, 0) }

    fn generate(&self, ctx: &ProbeContext, out: &mut ProbeTargetBuilder) -> bool {
        let Some(b) = ctx.block() else { return false };
        let powered = b.block_state.get("powered").map(|s| s == "true").unwrap_or(false);
        if !powered {
            return false;
        }
        out.add_head(ProbeLine::new(self.id(), Component::literal("⚡ Powered").with_color(Color::Red)));
        true
    }
}

// ---------------------------------------------------------------------------
// Прочность блока / скорость ломания (аналог jade:break_progress + tool)
// ---------------------------------------------------------------------------

pub struct BlockBreakableComponent;

impl ProbeComponent for BlockBreakableComponent {
    fn id(&self) -> &str { "rustjade:block_breakable" }
    fn display_name(&self) -> &str { "Можно сломать" }
    fn targets(&self) -> Vec<TargetKind> { blocks() }
    fn slot(&self) -> (Slot, Order) { (Slot::Tail, 100) }

    fn generate(&self, ctx: &ProbeContext, out: &mut ProbeTargetBuilder) -> bool {
        let Some(b) = ctx.block() else { return false };
        if let Some(held) = &ctx.held_item {
            let eff = held.efficiency;
            let (label, color) = if eff <= 0.0 {
                ("Cannot harvest", Color::Red)
            } else if eff < 1.0 {
                ("Slow to break", Color::Gold)
            } else {
                ("Can harvest", Color::Green)
            };
            let est = if eff > 0.0 { b.hardness / eff } else { f32::INFINITY };
            let text = if est.is_finite() {
                format!("{label} (~{est:.1}s)")
            } else {
                label.to_string()
            };
            out.add_tail(ProbeLine::new(self.id(), Component::literal(text).with_color(color)));
            true
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Координаты блока (аналог jade:block_position)
// ---------------------------------------------------------------------------

pub struct PositionComponent;

impl ProbeComponent for PositionComponent {
    fn id(&self) -> &str { "rustjade:position" }
    fn display_name(&self) -> &str { "Координаты" }
    fn targets(&self) -> Vec<TargetKind> { blocks() }
    fn slot(&self) -> (Slot, Order) { (Slot::Tail, 0) }

    fn generate(&self, ctx: &ProbeContext, out: &mut ProbeTargetBuilder) -> bool {
        let Some(b) = ctx.block() else { return false };
        let (x, y, z) = b.pos;
        let dist = ctx.player_pos.distance(&crate::world::Vec3::new(x as f64, y as f64, z as f64));
        let c = line(
            Component::literal("Position").with_color(Color::Gray),
            Component::literal(format!("{x}, {y}, {z}")).with_color(Color::White)
                .child(Component::literal(format!("  ({dist:.0}m)")).with_color(Color::DarkGray)),
        );
        out.add_tail(ProbeLine::new(self.id(), c));
        true
    }
}

// ---------------------------------------------------------------------------
// ID-хелпер в креативе (аналог jade:uid)
// ---------------------------------------------------------------------------

pub struct UidComponent;

impl ProbeComponent for UidComponent {
    fn id(&self) -> &str { "rustjade:uid" }
    fn display_name(&self) -> &str { "ID цели" }
    fn targets(&self) -> Vec<TargetKind> { all() }
    fn slot(&self) -> (Slot, Order) { (Slot::Tail, 999) }
    fn show_icon(&self) -> bool { false }

    fn generate(&self, ctx: &ProbeContext, out: &mut ProbeTargetBuilder) -> bool {
        if !ctx.creative {
            return false;
        }
        let id = match ctx.target {
            Target::Entity(e) => e.id.clone(),
            Target::Block(b) => b.id.clone(),
        };
        out.add_tail(ProbeLine::new(
            self.id(),
            Component::literal(id).with_color(Color::DarkGray).italic(),
        ));
        true
    }
}

// ---------------------------------------------------------------------------
// Расстояние до цели (аналог jade:distance)
// ---------------------------------------------------------------------------

pub struct DistanceComponent;

impl ProbeComponent for DistanceComponent {
    fn id(&self) -> &str { "rustjade:distance" }
    fn display_name(&self) -> &str { "Дистанция" }
    fn targets(&self) -> Vec<TargetKind> { entities() }
    fn slot(&self) -> (Slot, Order) { (Slot::Tail, 50) }

    fn generate(&self, ctx: &ProbeContext, out: &mut ProbeTargetBuilder) -> bool {
        let Some(e) = ctx.entity() else { return false };
        let d = ctx.player_pos.distance(&e.pos);
        out.add_tail(ProbeLine::new(
            self.id(),
            Component::literal(format!("{d:.1} blocks away")).with_color(Color::DarkGray),
        ));
        true
    }
}

// ---------------------------------------------------------------------------
// Регистрация встроенных компонентов (аналог CorePlugin::register)
// ---------------------------------------------------------------------------

pub fn register_core(reg: &mut crate::core::Registration) {
    reg.add(Box::new(RedstoneComponent)); // Head
    reg.add(Box::new(OwnerComponent)); // AfterName
    reg.add(Box::new(HealthComponent)); // Body 100
    reg.add(Box::new(ArmorComponent)); // Body 110
    reg.add(Box::new(ContentsComponent)); // Body 200
    reg.add(Box::new(SmeltingComponent)); // Body 250
    reg.add(Box::new(HorseInventoryComponent)); // Body 300
    reg.add(Box::new(PositionComponent)); // Tail 0
    reg.add(Box::new(DistanceComponent)); // Tail 50
    reg.add(Box::new(BlockBreakableComponent)); // Tail 100
    reg.add(Box::new(UidComponent)); // Tail 999
}
