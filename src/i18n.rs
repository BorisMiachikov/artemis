#![allow(dead_code)] // En используется с Фазы 2 (переключатель в меню)

use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum Lang {
    #[default]
    Ru,
    En,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Lang>();
    // Фаза 2: загрузка assets/i18n/{ru,en}.ron, макрос t!(key).
}
