use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use bevy_dolly::prelude::*;

#[derive(Component)]
pub struct MainCamera;

pub fn update_camera(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: QuerySet<(
        QueryState<(&mut Transform, With<MainCamera>)>,
        QueryState<&mut CameraRig>,
    )>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_ev: EventReader<MouseWheel>,
) {
    let rot_speed_mult = 0.25;
    let zoom_speed_mult = 0.5;

    let mut q1 = query.q1();
    let mut rig = q1.single_mut();
    let camera_driver = rig.driver_mut::<YawPitch>();

    if keys.just_pressed(KeyCode::Left) {
        camera_driver.rotate_yaw_pitch(-12.0, 0.0);
    }
    if keys.just_pressed(KeyCode::Right) {
        camera_driver.rotate_yaw_pitch(12.0, 0.0);
    }
    if keys.just_pressed(KeyCode::Up) {
        camera_driver.rotate_yaw_pitch(0.0, -12.0);
    }
    if keys.just_pressed(KeyCode::Down) {
        camera_driver.rotate_yaw_pitch(0.0, 12.0);
    }

    if mouse_button_input.pressed(MouseButton::Right) {
        for event in mouse_motion_events.iter() {
            let delta = event.delta;
            camera_driver
                .rotate_yaw_pitch(-1.2 * delta.x * rot_speed_mult, -delta.y * rot_speed_mult);
        }
    }

    if let Some(mouse_wheel) = mouse_wheel_ev.iter().last() {
        if mouse_wheel.y.abs() > 0.0 {
            rig.driver_mut::<Arm>().offset += -glam::Vec3::Z * mouse_wheel.y * zoom_speed_mult;
        }
    }

    let transform = rig.update(time.delta_seconds());
    let mut q0 = query.q0();
    let (mut cam, _) = q0.single_mut();

    cam.update(transform);
}
