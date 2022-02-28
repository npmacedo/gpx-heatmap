use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use bevy_dolly::prelude::*;

#[derive(Component)]
pub struct MainCamera;

pub fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(0.0, 25.0, -10.0);

    commands.spawn().insert(
        CameraRig::builder()
            .with(YawPitch::new().yaw_degrees(0.0).pitch_degrees(-30.0))
            .with(Position::new(Vec3::ZERO))
            .with(Smooth::new_rotation(1.5))
            .with(Arm::new(Vec3::Z * 15.0))
            .build(),
    );
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(translation.x, translation.y, translation.z)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..Default::default()
        })
        .insert(MainCamera);
}

pub fn update_camera(
    windows: Res<Windows>,
    time: Res<Time>,
    mut camera: Query<(&mut Transform, &mut PerspectiveProjection), With<MainCamera>>,
    mut camera_rig: Query<&mut CameraRig>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_ev: EventReader<MouseWheel>,
) {
    let orbit_button = MouseButton::Left;
    let pan_button = MouseButton::Right;
    let rot_speed_mult = 0.25;
    let zoom_speed_mult = 0.5;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;

    if mouse_button_input.pressed(orbit_button) {
        for ev in mouse_motion_events.iter() {
            rotation_move += ev.delta;
        }
    } else if mouse_button_input.pressed(pan_button) {
        for ev in mouse_motion_events.iter() {
            pan += ev.delta;
        }
    }

    let (mut transform, projection) = camera.get_single_mut().unwrap();
    let mut camera_rig = camera_rig.get_single_mut().unwrap();

    let arm_driver = camera_rig.driver_mut::<Arm>();
    let radius = arm_driver.offset.length();

    if let Some(mouse_wheel) = mouse_wheel_ev.iter().last() {
        if mouse_wheel.y.abs() > 0.0 {
            arm_driver.offset += -glam::Vec3::Z * mouse_wheel.y * zoom_speed_mult;
        }
    }

    let yaw_pitch_driver = camera_rig.driver_mut::<YawPitch>();

    if mouse_button_input.pressed(orbit_button) {
        let delta = rotation_move;
        yaw_pitch_driver
            .rotate_yaw_pitch(-1.2 * delta.x * rot_speed_mult, -delta.y * rot_speed_mult);
    }

    let position_driver = camera_rig.driver_mut::<Position>();

    if mouse_button_input.pressed(pan_button) {
        // make panning distance independent of resolution and FOV,
        let window = get_primary_window_size(&windows);
        pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
        // translate by local axes
        let right = transform.rotation * Vec3::X * -pan.x;
        let up = transform.rotation * Vec3::Y * pan.y;
        // make panning proportional to distance away from focus point
        let translation = (right + up) * radius;
        position_driver.translate(translation);
    }

    transform.update(camera_rig.update(time.delta_seconds()));
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}
