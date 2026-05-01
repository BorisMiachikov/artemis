use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};

use crate::assets::GameAssets;
use crate::audio::{MusicVolume, SfxVolume};
use crate::camera::{CameraMode, GameCamera};
use crate::config::Difficulty;
use crate::i18n::{Lang, Translations};
use crate::save::{LoadRequested, SaveSlot};
use crate::stages::launch_pad;
use crate::states::MissionStage;
use crate::ui::theme;

/// Параметры медленного облёта камеры в главном меню.
#[derive(Resource)]
struct MenuCameraOrbit {
    angle: f32,
    radius: f32,
    height: f32,
    target: Vec3,
}

impl Default for MenuCameraOrbit {
    fn default() -> Self {
        Self {
            angle: 0.0,
            radius: 70.0,
            height: 22.0,
            target: Vec3::new(0.0, 12.0, 0.0),
        }
    }
}

/// Фоновый слайд‑шоу в главном меню. Хэндлы загружаются через AssetServer
/// лениво — если файлы отсутствуют, остаётся 3D‑сцена.
#[derive(Resource, Default)]
struct MenuBackgrounds {
    handles: Vec<Handle<Image>>,
    elapsed: f32,
}

const BG_VISIBLE_S: f32 = 7.0;
const BG_FADE_S: f32 = 1.5;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(MissionStage::MainMenu), setup_scene)
        .add_systems(
            Update,
            (menu_camera_orbit, advance_background_timer)
                .run_if(in_state(MissionStage::MainMenu)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (draw_background, draw_main_menu)
                .chain()
                .run_if(in_state(MissionStage::MainMenu)),
        );
}

#[allow(clippy::too_many_arguments)]
fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets: Res<GameAssets>,
    mut cam_mode: ResMut<CameraMode>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cameras: Query<&mut Transform, With<GameCamera>>,
    mut time: ResMut<Time<Virtual>>,
    mut clear: ResMut<ClearColor>,
) {
    // Снять паузу если пришли из gameplay.
    time.unpause();

    // Светлое небо на время меню.
    clear.0 = launch_pad::SKY_COLOR;

    // Cockpit-режим снимает PanOrbitCamera, а отсутствие `PlayerVehicle`
    // оставляет управление трансформом нашему `menu_camera_orbit`.
    *cam_mode = CameraMode::Cockpit;
    let orbit = MenuCameraOrbit::default();
    if let Ok(mut tr) = cameras.single_mut() {
        let pos = orbit.target
            + Vec3::new(
                orbit.radius * orbit.angle.cos(),
                orbit.height,
                orbit.radius * orbit.angle.sin(),
            );
        *tr = Transform::from_translation(pos).looking_at(orbit.target, Vec3::Y);
    }
    commands.insert_resource(orbit);

    // Солнце.
    commands.spawn((
        DirectionalLight {
            illuminance: 50_000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.96, 0.90),
            ..default()
        },
        Transform::from_xyz(8.0, 12.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(MissionStage::MainMenu),
    ));

    // Тёплая подсветка металла башни (rim light у основания).
    commands.spawn((
        PointLight {
            intensity: 600_000.0,
            range: 60.0,
            color: Color::srgb(1.0, 0.85, 0.65),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-12.0, 6.0, 8.0),
        DespawnOnExit(MissionStage::MainMenu),
    ));

    // Окружение стартового комплекса.
    launch_pad::spawn_environment(
        &mut commands,
        meshes.as_mut(),
        materials.as_mut(),
        MissionStage::MainMenu,
    );

    // Стартовая башня.
    commands.spawn((
        SceneRoot(assets.gantry.clone()),
        Transform::from_xyz(0.0, 4.0, 0.0),
        DespawnOnExit(MissionStage::MainMenu),
    ));

    // Ракета SLS на стартовом столе.
    commands.spawn((
        SceneRoot(assets.sls.clone()),
        Transform::from_xyz(-3.0, 4.0, 0.0),
        DespawnOnExit(MissionStage::MainMenu),
    ));

    // Фоновое слайд‑шоу. Файлы лежат в assets/images/main_menu/*.jpg,
    // загружаются лениво: если их нет — фоном остаётся 3D‑сцена.
    let backgrounds = [
        "images/main_menu/01.jpg",
        "images/main_menu/02.jpg",
    ];
    let handles = backgrounds
        .iter()
        .map(|p| asset_server.load(*p))
        .collect::<Vec<_>>();
    commands.insert_resource(MenuBackgrounds {
        handles,
        elapsed: 0.0,
    });
}

/// Медленно вращает камеру вокруг стартового стола (≈3°/с) пока активно меню.
fn menu_camera_orbit(
    time: Res<Time>,
    mut orbit: Option<ResMut<MenuCameraOrbit>>,
    mut cameras: Query<&mut Transform, With<GameCamera>>,
) {
    let Some(orbit) = orbit.as_deref_mut() else {
        return;
    };
    orbit.angle += 0.05 * time.delta_secs();
    let Ok(mut tr) = cameras.single_mut() else {
        return;
    };
    let pos = orbit.target
        + Vec3::new(
            orbit.radius * orbit.angle.cos(),
            orbit.height,
            orbit.radius * orbit.angle.sin(),
        );
    *tr = Transform::from_translation(pos).looking_at(orbit.target, Vec3::Y);
}

fn advance_background_timer(time: Res<Time>, mut bg: Option<ResMut<MenuBackgrounds>>) {
    if let Some(bg) = bg.as_deref_mut() {
        bg.elapsed += time.delta_secs();
    }
}

/// Альфы для двух соседних слайдов на момент `t`. Цикл «полный показ → fade».
fn slide_alphas(t: f32, n: usize) -> (usize, f32, usize, f32) {
    if n == 0 {
        return (0, 0.0, 0, 0.0);
    }
    if n == 1 {
        return (0, 1.0, 0, 0.0);
    }
    let slot = BG_VISIBLE_S + BG_FADE_S;
    let cycle = slot * n as f32;
    let local = t.rem_euclid(cycle);
    let idx_a = (local / slot) as usize % n;
    let in_slot = local - slot * idx_a as f32;
    if in_slot < BG_VISIBLE_S {
        (idx_a, 1.0, (idx_a + 1) % n, 0.0)
    } else {
        let p = ((in_slot - BG_VISIBLE_S) / BG_FADE_S).clamp(0.0, 1.0);
        (idx_a, 1.0 - p, (idx_a + 1) % n, p)
    }
}

fn draw_background(
    mut contexts: EguiContexts,
    bg: Option<Res<MenuBackgrounds>>,
    images: Res<Assets<Image>>,
) -> Result {
    let Some(bg) = bg else { return Ok(()) };
    if bg.handles.is_empty() {
        return Ok(());
    }
    // Регистрируем хэндлы как egui-текстуры (только если bevy уже загрузил).
    let mut tex_ids: Vec<Option<egui::TextureId>> = Vec::with_capacity(bg.handles.len());
    for h in &bg.handles {
        if images.get(h).is_some() {
            tex_ids.push(Some(
                contexts.add_image(EguiTextureHandle::Strong(h.clone())),
            ));
        } else {
            tex_ids.push(None);
        }
    }

    let ctx = contexts.ctx_mut()?;
    let screen = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        "main_menu_bg".into(),
    ));

    let (i_a, a_a, i_b, a_b) = slide_alphas(bg.elapsed, bg.handles.len());
    for (i, alpha) in [(i_a, a_a), (i_b, a_b)] {
        if alpha <= 0.0 {
            continue;
        }
        if let Some(Some(id)) = tex_ids.get(i) {
            let a = (alpha * 255.0) as u8;
            painter.image(
                *id,
                screen,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, a),
            );
        }
    }

    // Лёгкая тёмная вуаль поверх фона — чтобы белые надписи лучше читались.
    painter.rect_filled(
        screen,
        0.0,
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 70),
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn draw_main_menu(
    mut contexts: EguiContexts,
    mut next_stage: ResMut<NextState<MissionStage>>,
    slot: Res<SaveSlot>,
    mut load_req: ResMut<LoadRequested>,
    mut lang: ResMut<Lang>,
    t: Res<Translations>,
    mut difficulty: ResMut<Difficulty>,
    mut music_vol: ResMut<MusicVolume>,
    mut sfx_vol: ResMut<SfxVolume>,
    mut show_settings: Local<bool>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let cur_lang = *lang;

    // ── Главная панель ────────────────────────────────────────────────────────
    egui::Window::new("main_menu")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::LEFT_CENTER, [16.0, 0.0])
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(24.0))
        .show(ctx, |ui| {
            ui.set_width(320.0);

            ui.vertical_centered(|ui| {
                ui.colored_label(
                    theme::SLS_ORANGE,
                    egui::RichText::new(t.get(cur_lang, "menu.title"))
                        .size(32.0)
                        .strong(),
                );
                ui.colored_label(
                    theme::TEXT_MUTED,
                    egui::RichText::new(t.get(cur_lang, "menu.subtitle")).size(14.0),
                );
            });

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(12.0);

            // Новая миссия
            let new_btn = egui::Button::new(
                egui::RichText::new(t.get(cur_lang, "menu.new_mission"))
                    .size(18.0)
                    .strong()
                    .color(theme::TEXT_PRIMARY),
            )
            .min_size(egui::vec2(272.0, 42.0))
            .fill(theme::NASA_BLUE);

            if ui.add(new_btn).clicked() {
                next_stage.set(MissionStage::Prelaunch);
            }

            ui.add_space(8.0);

            // Продолжить (только если есть прогресс)
            if slot.has_progress() {
                let stage_name = format!("{:?}", slot.mission_stage);
                let label = t
                    .get(cur_lang, "menu.continue_with_stage")
                    .replace("{stage}", &stage_name);
                let cont_btn = egui::Button::new(
                    egui::RichText::new(label)
                        .size(15.0)
                        .color(theme::STATUS_GREEN),
                )
                .min_size(egui::vec2(272.0, 36.0))
                .fill(theme::PANEL_BG)
                .stroke(egui::Stroke::new(1.0, theme::STATUS_GREEN));

                if ui.add(cont_btn).clicked() {
                    load_req.0 = true;
                }
                ui.add_space(8.0);
            }

            ui.separator();
            ui.add_space(8.0);

            // Настройки (toggle)
            let settings_label = if *show_settings {
                t.get(cur_lang, "menu.settings_close")
            } else {
                t.get(cur_lang, "menu.settings_open")
            };
            let settings_btn = egui::Button::new(
                egui::RichText::new(settings_label)
                    .size(14.0)
                    .color(theme::TEXT_MUTED),
            )
            .min_size(egui::vec2(272.0, 30.0))
            .fill(egui::Color32::TRANSPARENT);

            if ui.add(settings_btn).clicked() {
                *show_settings = !*show_settings;
            }

            if *show_settings {
                ui.add_space(8.0);
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 200))
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.set_width(248.0);

                        // Громкость музыки
                        ui.horizontal(|ui| {
                            ui.colored_label(theme::TEXT_MUTED, t.get(cur_lang, "menu.music"));
                            ui.add(egui::Slider::new(&mut music_vol.0, 0.0..=1.0).show_value(false));
                        });

                        // Громкость эффектов
                        ui.horizontal(|ui| {
                            ui.colored_label(theme::TEXT_MUTED, t.get(cur_lang, "menu.sfx"));
                            ui.add(egui::Slider::new(&mut sfx_vol.0, 0.0..=1.0).show_value(false));
                        });

                        ui.add_space(6.0);

                        // Язык
                        ui.horizontal(|ui| {
                            ui.colored_label(theme::TEXT_MUTED, t.get(cur_lang, "menu.language"));
                            egui::ComboBox::from_id_salt("mm_lang")
                                .selected_text(lang.label())
                                .show_ui(ui, |ui| {
                                    for opt in Lang::ALL {
                                        ui.selectable_value(&mut *lang, opt, opt.label());
                                    }
                                });
                        });

                        // Сложность
                        ui.horizontal(|ui| {
                            ui.colored_label(theme::TEXT_MUTED, t.get(cur_lang, "menu.difficulty"));
                            egui::ComboBox::from_id_salt("mm_diff")
                                .selected_text(match *difficulty {
                                    Difficulty::Story => t.get(cur_lang, "difficulty.story"),
                                    Difficulty::Realistic => {
                                        t.get(cur_lang, "difficulty.realistic")
                                    }
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut *difficulty,
                                        Difficulty::Story,
                                        t.get(cur_lang, "difficulty.story"),
                                    );
                                    ui.selectable_value(
                                        &mut *difficulty,
                                        Difficulty::Realistic,
                                        t.get(cur_lang, "difficulty.realistic"),
                                    );
                                });
                        });
                    });
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            // Выход
            let exit_btn = egui::Button::new(
                egui::RichText::new(t.get(cur_lang, "menu.exit_btn"))
                    .size(13.0)
                    .color(theme::TEXT_MUTED),
            )
            .min_size(egui::vec2(272.0, 26.0))
            .fill(egui::Color32::TRANSPARENT);

            if ui.add(exit_btn).clicked() {
                std::process::exit(0);
            }
        });

    // Версия билда — мелкая мета-плашка в правом нижнем углу.
    egui::Area::new("build_version".into())
        .anchor(egui::Align2::RIGHT_BOTTOM, [-12.0, -12.0])
        .show(ctx, |ui| {
            ui.colored_label(
                theme::TEXT_MUTED,
                egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).size(11.0),
            );
        });

    Ok(())
}
