// use std::time::Duration;

// use bevy::color::palettes::css::{DARK_GREEN, TAN};
// use bevy::core_pipeline::auto_exposure::AutoExposurePlugin;
// use bevy::prelude::*;

// use bevy_map_camera::look_transform::LookTransformLens;
// use bevy_map_camera::{LookTransform, MapCameraBundle, MapCameraPlugin};
// use bevy_tweening::{Animator, EaseMethod, RepeatCount, RepeatStrategy, Tween};

fn main() {
    // let mut app = App::new();
    // app.add_plugins((DefaultPlugins, AutoExposurePlugin, MapCameraPlugin));

    // app.add_systems(Startup, setup);
    // app.run();
}

// fn setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     // plane
//     commands.spawn(PbrBundle {
//         mesh: meshes.add(Plane3d::default().mesh().size(10., 10.)),
//         material: materials.add(Color::from(DARK_GREEN)),
//         ..Default::default()
//     });

//     let cube_material = materials.add(Color::from(TAN));

//     // cubes
//     for x in -2..=2 {
//         for z in -2..=2 {
//             commands.spawn(PbrBundle {
//                 mesh: meshes.add(Cuboid::from_size(Vec3::splat(0.2))),
//                 material: cube_material.clone(),
//                 transform: Transform::from_xyz((x * 2) as f32, 0.1, (z * 2) as f32),
//                 ..Default::default()
//             });
//         }
//     }

//     // light
//     commands.spawn(PointLightBundle {
//         point_light: PointLight {
//             shadows_enabled: true,
//             ..Default::default()
//         },
//         transform: Transform::from_xyz(4.0, 8.0, 4.0),
//         ..Default::default()
//     });

//     // easing and camera
//     let look_transform_from = LookTransform::new(
//         Vec3 {
//             x: 0.,
//             y: 8.5,
//             z: 10.0,
//         },
//         Vec3::ZERO,
//         Vec3::Y,
//     );

//     let target = LookTransform::new(
//         Vec3 {
//             x: 10.,
//             y: 8.5,
//             z: 0.0,
//         },
//         Vec3::ZERO,
//         Vec3::Y,
//     );

//     let tween = Tween::new(
//         EaseMethod::Linear,
//         Duration::from_secs(5),
//         LookTransformLens {
//             start: look_transform_from,
//             end: target,
//         },
//     )
//     .with_repeat_strategy(RepeatStrategy::MirroredRepeat)
//     .with_repeat_count(RepeatCount::Infinite);

//     commands.spawn((MapCameraBundle::default(), Animator::new(tween)));
// }
