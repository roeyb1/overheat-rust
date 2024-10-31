use bevy::prelude::*;

#[derive(Component)]
pub struct FaceCamera;

#[derive(Component)]
pub struct Animation {
    pub frames: Vec<usize>,
    pub current: usize,
    pub timer: Timer,
}

pub struct OverheatAnimationPlugin;

impl Plugin for OverheatAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update, (
                face_camera,
                animate_sprites,
            )
        );
    }
}

fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut Animation, &mut TextureAtlas)>
) {
    for (mut animation, mut atlas) in query.iter_mut() {
        animation.timer.tick(time.delta());
        if animation.timer.just_finished() {
            atlas.index = animation.frames[animation.current];
            animation.current += 1;
            animation.current %= animation.frames.len();
        }
    }
}

fn face_camera(
    cam_query: Query<&Transform, With<Camera>>, 
    mut query: Query<&mut Transform, (With<FaceCamera>, Without<Camera>)>
) {
    if cam_query.is_empty() { return; }

    let cam_transform = cam_query.single();
    for mut transform in query.iter_mut() {
        transform.rotation = cam_transform.rotation;
    }
}
