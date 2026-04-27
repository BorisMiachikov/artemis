use bevy::prelude::*;

use crate::events::MissionEvent;
use crate::states::MissionStage;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, handle_launch_input);
}

fn handle_launch_input(
    keys: Res<ButtonInput<KeyCode>>,
    stage: Res<State<MissionStage>>,
    mut next_stage: ResMut<NextState<MissionStage>>,
    mut events: MessageWriter<MissionEvent>,
) {
    // В Prelaunch — Space запускает миссию (дублирует кнопку в меню).
    if matches!(stage.get(), MissionStage::Prelaunch) && keys.just_pressed(KeyCode::Space) {
        events.write(MissionEvent::Liftoff);
        next_stage.set(MissionStage::Launch);
    }
}
