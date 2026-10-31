//! RustJade — аналог мода Jade (WAILA-HWT) для Minecraft на чистом Rust.
//!
//! Архитектурно повторяет Jade:
//!  * `core::ProbeComponent`     ~ `snownee.jade.api.IWailaComponent`
//!  * `core::Registry/Prober`    ~ `WailaApi` + pipeline приоритетов
//!  * `components::*`            ~ встроенные компоненты jade:health, contents...
//!  * `config::Config`           ~ jade.toml / JadeConfig
//!  * `hud::print_tooltip`       ~ рендер HUD (в консоли вместо GuiGraphics)
//!
//! Запуск: `cargo run` — интерактивная демка с raytrace "мира".

mod components;
mod config;
mod core;
mod hud;
mod text;
mod world;

use config::Config;
use core::{Prober, Registration, Registry};
use world::{
    BlockData, EntityData, HeldItem, Nbt, ProbeContext, Target, Vec3,
};

use std::collections::HashMap;

/// "Мир" демки: список целей вокруг игрока.
struct DemoWorld {
    targets: Vec<Target>,
}

fn make_world() -> DemoWorld {
    let mut chest_inv = vec![
        world::ItemStack { id: "minecraft:diamond".into(), name: "Diamond".into(), count: 3 },
        world::ItemStack { id: "minecraft:bread".into(), name: "Bread".into(), count: 17 },
        world::ItemStack { id: "minecraft:iron_ingot".into(), name: "Iron Ingot".into(), count: 64 },
    ];
    for i in 0..12 {
        chest_inv.push(world::ItemStack {
            id: format!("minecraft:oak_log_{i}"),
            name: format!("Oak Log #{i}"),
            count: 8,
        });
    }

    let mut furnace_nbt = HashMap::new();
    furnace_nbt.insert("FuelBurnTimeTicks".to_string(), Nbt::Int(140));

    let horse_nbt = Nbt::compound(&[(
        "HorseInventory",
        Nbt::compound(&[
            ("SaddleItem", Nbt::compound(&[
                ("id", Nbt::Str("minecraft:saddle".into())),
                ("Count", Nbt::Int(1)),
            ])),
            ("ItemSlot", Nbt::compound(&[
                ("id", Nbt::Str("minecraft:chest".into())),
                ("Count", Nbt::Int(1)),
            ])),
        ]),
    )]);

    let mut chest_state = HashMap::new();
    chest_state.insert("facing".to_string(), "north".to_string());

    let mut repeater_state = HashMap::new();
    repeater_state.insert("powered".to_string(), "true".to_string());
    repeater_state.insert("delay".to_string(), "3".to_string());

    DemoWorld {
        targets: vec![
            Target::Entity(EntityData {
                id: "minecraft:cow".into(),
                display_name: "Cow".into(),
                health: 6.0,
                max_health: 10.0,
                armor: 0.0,
                is_baby: false,
                is_tamed: false,
                owner_name: None,
                nbt: Nbt::Compound(HashMap::new()),
                pos: Vec3::new(10.4, 64.0, 5.2),
            }),
            Target::Entity(EntityData {
                id: "minecraft:wolf".into(),
                display_name: "Wolf".into(),
                health: 2.5,
                max_health: 8.0,
                armor: 0.0,
                is_baby: false,
                is_tamed: true,
                owner_name: Some("Steve".into()),
                nbt: Nbt::Compound(HashMap::new()),
                pos: Vec3::new(8.0, 64.0, 7.0),
            }),
            Target::Entity(EntityData {
                id: "minecraft:horse".into(),
                display_name: "Horse".into(),
                health: 15.0,
                max_health: 15.0,
                armor: 0.0,
                is_baby: false,
                is_tamed: true,
                owner_name: Some("Alex".into()),
                nbt: horse_nbt,
                pos: Vec3::new(12.0, 64.0, 9.0),
            }),
            Target::Block(BlockData {
                id: "minecraft:chest".into(),
                display_name: "Chest".into(),
                hardness: 2.5,
                block_state: chest_state,
                inventory: chest_inv,
                nbt: Nbt::Compound(HashMap::new()),
                pos: (10, 64, 3),
            }),
            Target::Block(BlockData {
                id: "minecraft:furnace".into(),
                display_name: "Furnace".into(),
                hardness: 3.5,
                block_state: HashMap::new(),
                inventory: Vec::new(),
                nbt: Nbt::Compound(furnace_nbt),
                pos: (9, 64, 2),
            }),
            Target::Block(BlockData {
                id: "minecraft:repeater".into(),
                display_name: "Redstone Repeater".into(),
                hardness: 0.0,
                block_state: repeater_state,
                inventory: Vec::new(),
                nbt: Nbt::Compound(HashMap::new()),
                pos: (7, 64, 4),
            }),
            Target::Block(BlockData {
                id: "minecraft:diamond_ore".into(),
                display_name: "Diamond Ore".into(),
                hardness: 3.0,
                block_state: HashMap::new(),
                inventory: Vec::new(),
                nbt: Nbt::Compound(HashMap::new()),
                pos: (11, 12, 25),
            }),
        ],
    }
}

/// Простейший "raytrace": цель, в конус прицела которой попадает луч
/// заданного направления (имитация `Minecraft.getInstance().hitResult`).
///
/// Возвращает **индекс** цели, чтобы вызывающий код мог взять ровно ту же
/// сущность (в реальном моде hitResult возвращает сам объект мира).
fn ray_trace(world: &DemoWorld, player: &Vec3, look_angle_deg: f64) -> Option<usize> {
    // Направление взгляда в плоскости XZ.
    let rad = look_angle_deg.to_radians();
    let dir = (rad.cos(), rad.sin());
    let mut best: Option<(f64, usize)> = None;
    for (i, t) in world.targets.iter().enumerate() {
        let p = target_pos(t);
        let dx = p.x - player.x;
        let dz = p.z - player.z;
        let dist = (dx * dx + dz * dz).sqrt();
        if dist < 0.1 || dist > 32.0 {
            continue;
        }
        // косинус угла между направлением взгляда и вектором до цели
        let dot = (dx * dir.0 + dz * dir.1) / dist;
        if dot < 0.98 {
            continue; // вне узкого конуса прицела (~±10°)
        }
        let score = dist / dot; // чем меньше — тем "ближе к прицелу"
        if best.is_none_or(|(s, _)| score < s) {
            best = Some((score, i));
        }
    }
    best.map(|(_, i)| i)
}

/// Позиция цели для расчёта угла (используется в демке).
fn target_pos(t: &Target) -> Vec3 {
    match t {
        Target::Entity(e) => e.pos,
        Target::Block(b) => Vec3::new(
            b.pos.0 as f64 + 0.5,
            b.pos.1 as f64 + 0.5,
            b.pos.2 as f64 + 0.5,
        ),
    }
}

/// Угол взгляда на конкретную цель из позиции игрока (для ray_trace).
pub fn angle_to(player: &Vec3, t: &Target) -> f64 {
    let p = target_pos(t);
    let dx = p.x - player.x;
    let dz = p.z - player.z;
    dz.atan2(dx).to_degrees()
}

fn main() {
    println!("\x1b[36mRustJade\x1b[0m — консольная демка аналога мода Jade для Minecraft\n");

    // Конфиг: ./rustjade.toml (создаётся при первом запуске).
    let cfg = Config::load_or_create(std::path::Path::new("rustjade.toml"));

    let mut registry = Registry::new();
    {
        let mut reg = Registration::new(&mut registry);
        components::register_core(&mut reg);
        // Отключаем компоненты согласно конфигу (как Jade).
    }
    for (id, on) in &cfg.components {
        if !*on {
            registry.disable(id);
        }
    }

    let world = make_world();
    let player = Vec3::new(8.0, 64.0, 0.0);
    let prober = Prober::new(&registry);

    let scenarios = [
        ("Корова (survival, без предмета)", "minecraft:cow", false, false, None),
        ("Волк (приручённый, мало HP)", "minecraft:wolf", false, false, None),
        ("Лошадь (с седлом и сундуком)", "minecraft:horse", false, false, None),
        ("Сундук (обычный режим)", "minecraft:chest", false, false, Some(HeldItem {
            id: "minecraft:iron_pickaxe".into(),
            name: "Iron Pickaxe".into(),
            efficiency: 1.0,
        })),
        ("Сундук (SHIFT — сводка)", "minecraft:chest", true, false, None),
        ("Печь (топка активна)", "minecraft:furnace", false, false, None),
        ("Редстоун-повторитель (запитан)", "minecraft:repeater", false, false, None),
        ("Алмазная руда (руками — нельзя)", "minecraft:diamond_ore", false, false, Some(HeldItem {
            id: "minecraft:air".into(),
            name: "Hand".into(),
            efficiency: 0.0,
        })),
        ("Алмазная руда (креатив: ID + кирка)", "minecraft:diamond_ore", false, true, Some(HeldItem {
            id: "minecraft:diamond_pickaxe".into(),
            name: "Diamond Pickaxe".into(),
            efficiency: 4.0,
        })),
    ];

    for (title, target_id, sneak, creative, held) in scenarios {
        // Ищем цель и наводимся точно на неё (ray_trace по вычисленному углу).
        let Some(idx) = world.targets.iter().position(|t| t.id() == target_id) else {
            println!("({title}: цель не найдена)");
            continue;
        };
        let angle = angle_to(&player, &world.targets[idx]);
        // Raytrace подтверждает, что под этим углом мы видим именно нужную цель.
        let hit = ray_trace(&world, &player, angle);
        let Some(hit_idx) = hit.filter(|i| *i == idx) else {
            println!("({title}: raytrace под углом {angle:.1}° попал не в цель: {hit:?})");
            continue;
        };
        let target = world.targets[hit_idx].clone();
        println!("\x1b[1m▶ {title}\x1b[0m");
        let ctx = ProbeContext {
            target: &target,
            player_pos: player,
            held_item: held,
            sneaking: sneak,
            creative,
        };
        let tip = prober.probe(&ctx);
        if cfg.hud.show_overlay {
            hud::print_tooltip(&tip);
        } else {
            println!("(HUD скрыт в конфиге)");
        }
        println!();
    }

    println!("Файл конфига: rustjade.toml (правьте [components], чтобы вкл/выкл строки).");
}

