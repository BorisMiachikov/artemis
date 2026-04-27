use std::collections::HashMap;

use bevy::platform::collections::HashMap as BevyHashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Lang {
    #[default]
    Ru,
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Ru, Lang::En];

    pub fn label(self) -> &'static str {
        match self {
            Lang::Ru => "Русский",
            Lang::En => "English",
        }
    }
}

#[derive(Resource, Default)]
pub struct Translations {
    by_lang: BevyHashMap<Lang, HashMap<String, String>>,
}

impl Translations {
    pub fn get<'a>(&'a self, lang: Lang, key: &'a str) -> &'a str {
        self.by_lang
            .get(&lang)
            .and_then(|d| d.get(key))
            .map(String::as_str)
            .unwrap_or(key)
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Lang>()
        .init_resource::<Translations>()
        .add_systems(PreStartup, load_translations);
}

fn load_translations(mut translations: ResMut<Translations>) {
    for lang in Lang::ALL {
        let path = match lang {
            Lang::Ru => "assets/i18n/ru.ron",
            Lang::En => "assets/i18n/en.ron",
        };
        match std::fs::read_to_string(path) {
            Ok(content) => match ron::from_str::<HashMap<String, String>>(&content) {
                Ok(dict) => {
                    info!("i18n: загружено {} ключей для {:?}", dict.len(), lang);
                    translations.by_lang.insert(lang, dict);
                }
                Err(e) => error!("i18n: ошибка парсинга {}: {}", path, e),
            },
            Err(e) => error!("i18n: не удалось прочитать {}: {}", path, e),
        }
    }
}
