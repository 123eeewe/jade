//! Текстовые компоненты Minecraft (аналог `Component` из Jade).
//!
//! Поддерживается упрощённое представление текста с форматированием:
//! цвета, жирный/курсив/зачёркнутый, а также вложенные "child"-компоненты.

/// Цвета в стиле Minecraft (§-коды).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl Color {
    pub fn section_code(self) -> char {
        use Color::*;
        match self {
            Black => '0',
            DarkBlue => '1',
            DarkGreen => '2',
            DarkAqua => '3',
            DarkRed => '4',
            DarkPurple => '5',
            Gold => '6',
            Gray => '7',
            DarkGray => '8',
            Blue => '9',
            Green => 'a',
            Aqua => 'b',
            Red => 'c',
            LightPurple => 'd',
            Yellow => 'e',
            White => 'f',
        }
    }
}

/// Стиль текстового компонента.
#[derive(Debug, Clone, Copy, Default)]
pub struct Style {
    pub color: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,
    pub strikethrough: bool,
    pub obfuscated: bool,
}

/// Текстовый компонент — аналог `net.minecraft.network.chat.Component`.
#[derive(Debug, Clone)]
pub struct Component {
    pub text: String,
    pub style: Style,
    pub children: Vec<Component>,
}

impl Component {
    pub fn literal<S: Into<String>>(text: S) -> Self {
        Component {
            text: text.into(),
            style: Style::default(),
            children: Vec::new(),
        }
    }

    /// Пустой компонент (используется как контейнер для children).
    pub fn empty() -> Self {
        Self::literal("")
    }

    pub fn with_color(mut self, c: Color) -> Self {
        self.style.color = Some(c);
        self
    }

    pub fn bold(mut self) -> Self {
        self.style.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.style.italic = true;
        self
    }

    pub fn strikethrough(mut self) -> Self {
        self.style.strikethrough = true;
        self
    }

    pub fn child(mut self, c: Component) -> Self {
        self.children.push(c);
        self
    }

    pub fn append_text(mut self, s: &str) -> Self {
        self.text.push_str(s);
        self
    }

    /// Рендер в строку с §-кодами (для отладки и консоли).
    pub fn render(&self) -> String {
        let mut out = String::new();
        self.render_into(&mut out, None);
        out
    }

    fn render_into(&self, out: &mut String, parent: Option<Style>) {
        let style = self.inherited(parent);
        if let Some(c) = style.color {
            out.push('§');
            out.push(c.section_code());
        }
        if style.bold {
            out.push_str("§l");
        }
        if style.italic {
            out.push_str("§o");
        }
        if style.strikethrough {
            out.push_str("§m");
        }
        out.push_str(&self.text);
        for ch in &self.children {
            ch.render_into(out, Some(style));
        }
        if parent.is_none() {
            out.push_str("§r");
        }
    }

    fn inherited(&self, parent: Option<Style>) -> Style {
        let mut s = self.style;
        if let Some(p) = parent {
            if s.color.is_none() {
                s.color = p.color;
            }
            s.bold |= p.bold;
            s.italic |= p.italic;
            s.strikethrough |= p.strikethrough;
        }
        s
    }

    /// Plain-текст без кодов форматирования (для тестов/доступности).
    pub fn plain(&self) -> String {
        let mut out = self.text.clone();
        for ch in &self.children {
            out.push_str(&ch.plain());
        }
        out
    }
}

/// Удобная сборка строк вида "ключ: значение".
pub fn line(key: Component, value: Component) -> Component {
    Component::empty().child(key).child(Component::literal(": ").with_color(Color::Gray)).child(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_concatenation() {
        let c = Component::literal("Cow")
            .with_color(Color::Aqua)
            .bold()
            .child(Component::literal(" (Baby)")).italic();
        assert_eq!(c.plain(), "Cow (Baby)");
    }

    #[test]
    fn renders_section_codes() {
        let c = Component::literal("Hi").with_color(Color::Red);
        assert_eq!(c.render(), "§cHi§r");
    }
}
