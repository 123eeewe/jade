//! Конфигурация (аналог `jade.toml` / `JadeConfig`).
//!
//! Формат — простой INI-подобный парсер без внешних зависимостей:
//! секции `[hud]`, ключи `key = value`, комментарии `#`.

use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HudConfig {
    pub show_overlay: bool,
    /// Расположение тултипа: left / middle / right.
    pub overlay_pos: String,
    pub hide_from_debug: bool,
    /// Сколько тиков показывать после потери цели (fade-out).
    pub fade_ticks: u32,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub hud: HudConfig,
    /// id компонента -> включён ли.
    pub components: HashMap<String, bool>,
}

impl Default for Config {
    fn default() -> Self {
        let mut components = HashMap::new();
        // Как в Jade: по умолчанию всё включено, кроме uid.
        components.insert("rustjade:uid".to_string(), false);
        Config {
            hud: HudConfig {
                show_overlay: true,
                overlay_pos: "middle".into(),
                hide_from_debug: true,
                fade_ticks: 5,
            },
            components,
        }
    }
}

impl Config {
    /// Загружает конфиг из файла; при отсутствии — дефолт и запись файла.
    pub fn load_or_create(path: &Path) -> Config {
        match std::fs::read_to_string(path) {
            Ok(s) => Config::parse(&s),
            Err(_) => {
                let c = Config::default();
                let _ = std::fs::write(path, c.dump());
                c
            }
        }
    }

    pub fn parse(text: &str) -> Config {
        let mut cfg = Config::default();
        let mut section = String::new();
        for raw in text.lines() {
            let line = raw.split('#').next().unwrap_or("").trim().to_string();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].to_string();
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim().to_string();
                let v = v.trim().to_string();
                match section.as_str() {
                    "hud" => match k.as_str() {
                        "show_overlay" => cfg.hud.show_overlay = parse_bool(&v),
                        "overlay_pos" => cfg.hud.overlay_pos = v,
                        "hide_from_debug" => cfg.hud.hide_from_debug = parse_bool(&v),
                        "fade_ticks" => cfg.hud.fade_ticks = v.parse().unwrap_or(5),
                        _ => {}
                    },
                    "components" => {
                        cfg.components.insert(k, parse_bool(&v));
                    }
                    _ => {}
                }
            }
        }
        cfg
    }

    pub fn dump(&self) -> String {
        let mut s = String::from("# RustJade config (аналог jade.toml)\n\n[hud]\n");
        s.push_str(&format!("show_overlay = {}\n", self.hud.show_overlay));
        s.push_str(&format!("overlay_pos = {}\n", self.hud.overlay_pos));
        s.push_str(&format!("hide_from_debug = {}\n", self.hud.hide_from_debug));
        s.push_str(&format!("fade_ticks = {}\n", self.hud.fade_ticks));
        s.push_str("\n[components]\n");
        let mut keys: Vec<_> = self.components.keys().collect();
        keys.sort();
        for k in keys {
            s.push_str(&format!("{k} = {}\n", self.components[k]));
        }
        s
    }

    pub fn is_component_enabled(&self, id: &str) -> bool {
        *self.components.get(id).unwrap_or(&true)
    }
}

fn parse_bool(v: &str) -> bool {
    matches!(v.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut cfg = Config::default();
        cfg.components.insert("rustjade:health".into(), false);
        let text = cfg.dump();
        let parsed = Config::parse(&text);
        assert!(!parsed.is_component_enabled("rustjade:health"));
        assert!(parsed.is_component_enabled("rustjade:armor"));
        assert!(!parsed.is_component_enabled("rustjade:uid"));
        assert_eq!(parsed.hud.overlay_pos, "middle");
    }
}
