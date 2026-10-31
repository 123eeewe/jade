//! Ядро RustJade: компоненты наведения, реестр, приоритеты.
//!
//! Архитектура повторяет Jade (WAILA-HWT):
//!  * `ProbeComponent` — аналог `IWailaComponent` — генерирует строки UI;
//!  * `ComponentPriority` — аналог `WailaPlugins.runCoreInit` c порядком
//!    HEAD / AFTER_NAME / BODY / TAIL;
//!  * `Registration` — аналог `IWailaPlugin::register(IWailaRegistrar)`.

use crate::text::{Color, Component};
use crate::world::{ProbeContext, TargetKind};

/// Приоритет позиции компонента в тултипе (аналог `Priority`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Slot {
    /// Над названием цели (иконки статуса).
    Head,
    /// Сразу после названия.
    AfterName,
    /// Тело тултипа (большинство компонентов).
    Body,
    /// В самом низу (подсказки, ID).
    Tail,
}

/// Порядок внутри слота — как числовые priority в Jade.
pub type Order = i32;

/// Строка результата компонента: текст + опциональная "иконка".
#[derive(Debug, Clone)]
pub struct ProbeLine {
    pub component: Component,
    /// Иконка предмета/текстуры слева от строки (в консоли — символ).
    pub icon: Option<String>,
    /// Компонент-владелец (для фильтрации по toggle'ам).
    pub owner: String,
}

impl ProbeLine {
    pub fn new(owner: &str, c: Component) -> Self {
        ProbeLine { component: c, icon: None, owner: owner.to_string() }
    }
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }
}

/// Результат работы компонента: куда положить строки.
#[derive(Debug, Default)]
pub struct ProbeTargetBuilder {
    pub head: Vec<ProbeLine>,
    pub after_name: Vec<ProbeLine>,
    pub body: Vec<ProbeLine>,
    pub tail: Vec<ProbeLine>,
}

impl ProbeTargetBuilder {
    pub fn prepend(&mut self, line: ProbeLine) {
        self.body.insert(0, line);
    }
    pub fn append(&mut self, line: ProbeLine) {
        self.body.push(line);
    }
    pub fn add_head(&mut self, line: ProbeLine) {
        self.head.push(line);
    }
    pub fn add_after_name(&mut self, line: ProbeLine) {
        self.after_name.push(line);
    }
    pub fn add_tail(&mut self, line: ProbeLine) {
        self.tail.push(line);
    }
}

/// Аналог `IWailaComponent` — один информационный компонент.
pub trait ProbeComponent {
    /// Уникальный ID, напр. "rustjade:health".
    fn id(&self) -> &str;
    /// Человекочитаемое имя для конфига.
    fn display_name(&self) -> &str;
    /// Для каких целей работает.
    fn targets(&self) -> Vec<TargetKind>;
    /// Слот и порядок.
    fn slot(&self) -> (Slot, Order);

    /// Заполняет билдер данными о цели. Возвращает false, если
    /// компонент не применим (аналог `generateTooltip` с early-return).
    fn generate(&self, ctx: &ProbeContext, out: &mut ProbeTargetBuilder) -> bool;

    /// Показывать ли иконку этой строки (как `showIcon` в конфиге Jade).
    fn show_icon(&self) -> bool {
        true
    }
}

/// Тип регистрации плагина (аналог `IWailaPlugin`).
pub type PluginFn = fn(&mut Registration);

/// Реестр компонентов (аналог `IWailaRegistrar`).
pub struct Registry {
    pub components: Vec<Box<dyn ProbeComponent>>,
    /// Выключенные компоненты (из config.toml, key = id).
    pub disabled: Vec<String>,
}

impl Registry {
    pub fn new() -> Self {
        Registry { components: Vec::new(), disabled: Vec::new() }
    }

    pub fn register(&mut self, c: Box<dyn ProbeComponent>) {
        self.components.push(c);
    }

    /// Отключить компонент по ID (аналог `[[component]] enabled=false`).
    pub fn disable(&mut self, id: &str) {
        self.disabled.push(id.to_string());
    }

    pub fn is_enabled(&self, id: &str) -> bool {
        !self.disabled.iter().any(|d| d == id)
    }

    /// Сортировка по (slot, order) — стабильно, как в Jade.
    pub fn sorted(&self) -> Vec<&Box<dyn ProbeComponent>> {
        let mut v: Vec<&Box<dyn ProbeComponent>> =
            self.components.iter().filter(|c| self.is_enabled(c.id())).collect();
        v.sort_by_key(|c| {
            let (slot, order) = c.slot();
            let s = match slot {
                Slot::Head => 0,
                Slot::AfterName => 1,
                Slot::Body => 2,
                Slot::Tail => 3,
            };
            (s, order)
        });
        v
    }
}

/// Объект, который получает плагин при загрузке (аналог registrar).
pub struct Registration<'a> {
    registry: &'a mut Registry,
}

impl<'a> Registration<'a> {
    pub fn new(registry: &'a mut Registry) -> Self {
        Registration { registry }
    }
    pub fn add(&mut self, c: Box<dyn ProbeComponent>) {
        self.registry.register(c);
    }
}

/// Готовый тултип: название + строки по секциям.
#[derive(Debug, Clone)]
pub struct Tooltip {
    pub title: Component,
    pub lines: Vec<ProbeLine>,
}

/// Главный "пробник" (аналог `WailaApi.probe()` + HUD-рендер).
pub struct Prober<'a> {
    pub registry: &'a Registry,
}

impl<'a> Prober<'a> {
    pub fn new(registry: &'a Registry) -> Self {
        Prober { registry }
    }

    /// Собирает тултип для цели.
    pub fn probe(&self, ctx: &ProbeContext) -> Tooltip {
        let mut b = ProbeTargetBuilder::default();
        for comp in self.registry.sorted() {
            if comp.targets().contains(&ctx.target.kind()) {
                comp.generate(ctx, &mut b);
            }
        }

        let title = build_title(ctx);
        let mut lines = Vec::new();
        lines.extend(b.head);
        lines.extend(b.after_name);
        lines.extend(b.body);
        lines.extend(b.tail);
        Tooltip { title, lines }
    }
}

fn build_title(ctx: &ProbeContext) -> Component {
    use crate::world::Target;
    match ctx.target {
        Target::Entity(e) => {
            let mut t = Component::literal(&e.display_name).with_color(Color::Aqua).bold();
            if e.is_baby {
                t = t.child(Component::literal(" (Baby)").italic());
            }
            t
        }
        Target::Block(b) => {
            let mut t = Component::literal(&b.display_name).with_color(Color::Yellow).bold();
            if ctx.creative {
                t = t.child(Component::literal(format!(" [{}]", b.id)).with_color(Color::DarkGray));
            }
            t
        }
    }
}
