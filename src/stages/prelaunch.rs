use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::assets::GameAssets;
use crate::camera::{CameraMode, GameCamera, PlayerVehicle};
use crate::i18n::{Lang, Translations};
use crate::physics::rocket::Rocket;
use crate::states::MissionStage;
use crate::stages::launch_pad;

// ---------------------------------------------------------------------------
// Ресурс кат-сцены
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct PrelaunchCutscene {
    elapsed:  f32,
    duration: f32,
    pub done: bool,
}

impl Default for PrelaunchCutscene {
    fn default() -> Self {
        Self { elapsed: 0.0, duration: 12.0, done: false }
    }
}

// ---------------------------------------------------------------------------
// Маркерные компоненты
// ---------------------------------------------------------------------------

#[derive(Component)]
struct CrawlerOnApproach;

#[derive(Component)]
struct SlsOnCrawler;

// ---------------------------------------------------------------------------
// Константы движения
// ---------------------------------------------------------------------------

const CRAWLER_START: Vec3 = Vec3::new(0.0,   0.0, -120.0);
const CRAWLER_END:   Vec3 = Vec3::ZERO;
const SLS_START:     Vec3 = Vec3::new(-3.0,  4.0, -120.0);
const SLS_END:       Vec3 = Vec3::new(-3.0,  4.0,    0.0);
const CAM_START:     Vec3 = Vec3::new(60.0, 12.0,  -80.0);
const CAM_END:       Vec3 = Vec3::new(40.0, 20.0,   60.0);
const LOOK_START:    Vec3 = Vec3::new(0.0,   8.0, -120.0);
const LOOK_END:      Vec3 = Vec3::new(0.0,  12.0,    0.0);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(MissionStage::Prelaunch),
        (cleanup_stale_vehicles, setup_scene).chain(),
    )
    .add_systems(
        Update,
        tick_cutscene.run_if(in_state(MissionStage::Prelaunch)),
    )
    .add_systems(
        EguiPrimaryContextPass,
        draw_skip_hint.run_if(in_state(MissionStage::Prelaunch)),
    );
}

/// Страховка от «двух ракет на стартовом столе» после рестарта проваленной миссии:
/// убираем любые `PlayerVehicle`, `Rocket` и GLB‑сцены (`SceneRoot`) от прежней
/// стадии до того как Prelaunch положит свои Crawler+SLS+Gantry. `DespawnOnExit`
/// обычно справляется, но при отдельных путях перехода (Splashdown→Prelaunch,
/// gameover из середины Launch и т.д.) сущности утекают, и мы видим дубль.
///
/// Это безопасно: ландшафт launch_pad (трава, холмы, ёлки) использует `Mesh3d`,
/// а не `SceneRoot`, поэтому не попадает под чистку. Свет `DirectionalLight`
/// тоже не задевается. Все `SceneRoot`‑сущности заново спавнятся в `setup_scene`.
#[allow(clippy::type_complexity)]
fn cleanup_stale_vehicles(
    mut commands: Commands,
    stale: Query<Entity, Or<(With<PlayerVehicle>, With<Rocket>, With<SceneRoot>)>>,
) {
    let count = stale.iter().count();
    if count > 0 {
        info!("prelaunch: cleanup despawning {count} stale scene entities");
    }
    for e in &stale {
        commands.entity(e).despawn();
    }
}

// ---------------------------------------------------------------------------
// Системы
// ---------------------------------------------------------------------------

fn setup_scene(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut cam_mode: ResMut<CameraMode>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(PrelaunchCutscene::default());

    // Cockpit-режим → manage_pan_orbit снимет PanOrbitCamera на следующем кадре.
    *cam_mode = CameraMode::Cockpit;

    // Окружение стартового комплекса (трава, бетон, холмы, посадка).
    launch_pad::spawn_environment(
        &mut commands,
        meshes.as_mut(),
        materials.as_mut(),
        MissionStage::Prelaunch,
    );

    // Солнце: яркий направленный свет с тенями.
    commands.spawn((
        DirectionalLight {
            illuminance: 50_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 12.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(MissionStage::Prelaunch),
    ));

    // Gantry — неподвижна у стартового стола.
    commands.spawn((
        SceneRoot(assets.gantry.clone()),
        Transform::from_xyz(0.0, 4.0, 0.0),
        DespawnOnExit(MissionStage::Prelaunch),
    ));

    // Crawler — начинает далеко и едет к столу.
    commands.spawn((
        SceneRoot(assets.crawler.clone()),
        Transform::from_translation(CRAWLER_START),
        CrawlerOnApproach,
        DespawnOnExit(MissionStage::Prelaunch),
    ));

    // SLS — верхом на Crawler, движется вместе с ним.
    commands.spawn((
        SceneRoot(assets.sls.clone()),
        Transform::from_translation(SLS_START),
        SlsOnCrawler,
        DespawnOnExit(MissionStage::Prelaunch),
    ));
}

#[allow(clippy::type_complexity)]
fn tick_cutscene(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut cutscene: ResMut<PrelaunchCutscene>,
    mut cam_mode: ResMut<CameraMode>,
    mut crawlers: Query<
        &mut Transform,
        (With<CrawlerOnApproach>, Without<SlsOnCrawler>, Without<GameCamera>),
    >,
    mut slses: Query<
        &mut Transform,
        (With<SlsOnCrawler>, Without<CrawlerOnApproach>, Without<GameCamera>),
    >,
    mut cameras: Query<
        &mut Transform,
        (With<GameCamera>, Without<CrawlerOnApproach>, Without<SlsOnCrawler>),
    >,
) {
    if cutscene.done {
        return;
    }

    cutscene.elapsed += time.delta_secs();

    let skip     = keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter);
    let finished = skip || cutscene.elapsed >= cutscene.duration;
    let t        = if finished { 1.0 } else { smooth_step(cutscene.elapsed / cutscene.duration) };

    if let Ok(mut tr) = crawlers.single_mut() {
        tr.translation = CRAWLER_START.lerp(CRAWLER_END, t);
    }
    if let Ok(mut tr) = slses.single_mut() {
        tr.translation = SLS_START.lerp(SLS_END, t);
    }
    if let Ok(mut tr) = cameras.single_mut() {
        let pos  = CAM_START.lerp(CAM_END, t);
        let look = LOOK_START.lerp(LOOK_END, t);
        *tr = Transform::from_translation(pos).looking_at(look, Vec3::Y);
    }

    if finished {
        cutscene.done = true;
        // External → manage_pan_orbit вернёт PanOrbitCamera на следующем кадре.
        *cam_mode = CameraMode::External;
    }
}

fn draw_skip_hint(
    mut ctx: EguiContexts,
    cutscene: Option<Res<PrelaunchCutscene>>,
    lang: Res<Lang>,
    t: Res<Translations>,
) -> Result {
    let Some(cs) = cutscene else { return Ok(()) };
    if cs.done { return Ok(()); }

    let c = ctx.ctx_mut()?;
    egui::Area::new("skip_hint".into())
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -24.0])
        .show(c, |ui| {
            ui.label(
                egui::RichText::new(t.get(*lang, "prelaunch.skip_hint"))
                    .color(egui::Color32::from_rgba_unmultiplied(200, 200, 200, 180)),
            );
        });
    Ok(())
}

// ---------------------------------------------------------------------------
// Вспомогательное
// ---------------------------------------------------------------------------

fn smooth_step(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
