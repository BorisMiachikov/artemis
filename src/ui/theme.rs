//! Цвета и шрифты HUD/меню — современный минимализм с акцентами NASA.

use bevy_egui::egui::Color32;

/// Официальный синий NASA — основной цвет акцента.
pub const NASA_BLUE: Color32 = Color32::from_rgb(0x0B, 0x3D, 0x91);

/// Оранжевый SLS — критические значения и активные элементы.
pub const SLS_ORANGE: Color32 = Color32::from_rgb(0xFC, 0x3D, 0x21);

/// Тёплый белый для основного текста.
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xE8, 0xEA, 0xF0);

/// Приглушённый текст (подписи, hint'ы).
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x9A, 0xA0, 0xAE);

/// Полупрозрачный фон HUD-панелей.
pub const PANEL_BG: Color32 = Color32::from_rgba_premultiplied(0x0A, 0x0E, 0x1A, 0xC0);

/// Зелёный — успешные показатели.
pub const STATUS_GREEN: Color32 = Color32::from_rgb(0x4C, 0xD9, 0x6A);

/// Жёлтый — допустимые, но неоптимальные показатели.
pub const STATUS_YELLOW: Color32 = Color32::from_rgb(0xFF, 0xD0, 0x44);
