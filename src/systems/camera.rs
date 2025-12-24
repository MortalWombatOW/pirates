// Camera-related systems for visual effects.

use bevy::prelude::*;
use crate::components::camera::CameraShake;
use crate::events::CannonFiredEvent;
use crate::plugins::core::MainCamera;

/// System that applies camera shake based on trauma level.
/// Uses noise-based offset for smooth, organic shake.
pub fn camera_shake_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut CameraShake), With<MainCamera>>,
) {
    let Ok((mut transform, mut shake)) = query.get_single_mut() else {
        return;
    };

    // Decay trauma over time
    shake.decay(time.delta_secs());

    // Calculate shake intensity (trauma²)
    let intensity = shake.shake_intensity();
    
    if intensity > 0.001 {
        // Use simple pseudo-random based on noise_time for offset
        let noise_x = (shake.noise_time * 50.0).sin();
        let noise_y = (shake.noise_time * 60.0 + 1.5).cos();
        let noise_rot = (shake.noise_time * 45.0 + 3.0).sin();
        
        // Apply offset based on intensity
        let offset_x = noise_x * shake.max_offset * intensity;
        let offset_y = noise_y * shake.max_offset * intensity;
        let offset_rot = noise_rot * shake.max_rotation * intensity;
        
        // Apply shake offset (this is cumulative, so store base position separately)
        // For simplicity, we directly apply the offset each frame
        // The camera_follow system will reset position, so this is an additive effect
        transform.translation.x += offset_x;
        transform.translation.y += offset_y;
        transform.rotation = Quat::from_rotation_z(offset_rot);
    } else {
        // Reset rotation when not shaking
        transform.rotation = Quat::IDENTITY;
    }
}

/// System that triggers camera shake when cannons are fired.
pub fn trigger_camera_shake_on_fire(
    mut events: EventReader<CannonFiredEvent>,
    mut query: Query<&mut CameraShake, With<MainCamera>>,
) {
    let Ok(mut shake) = query.get_single_mut() else {
        return;
    };

    for _event in events.read() {
        // Add trauma for each cannon fired event
        shake.add_trauma(0.3);
    }
}
