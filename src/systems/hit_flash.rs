// Hit flash visual effect systems.

use bevy::prelude::*;
use crate::components::hit_flash::HitFlash;
use crate::components::Ship;
use crate::events::ShipHitEvent;

/// Bright red flash color for damage feedback.
/// Using red instead of white because sprites default to white color.
const FLASH_COLOR: Color = Color::srgb(1.0, 0.3, 0.3);

/// System that triggers hit flash effect when ships take damage.
/// Immediately sets the sprite to bright red.
pub fn trigger_hit_flash_system(
    mut commands: Commands,
    mut events: EventReader<ShipHitEvent>,
    mut query: Query<(Entity, &mut Sprite, Option<&Name>), (With<Ship>, Without<HitFlash>)>,
) {
    for event in events.read() {
        // Check if the hit ship exists and doesn't already have a flash
        if let Ok((entity, mut sprite, name)) = query.get_mut(event.ship_entity) {
            // Store original color and add flash component
            let original_color = sprite.color;
            
            // Immediately set sprite to flash color
            sprite.color = FLASH_COLOR;
            
            commands.entity(entity).insert(HitFlash::new(
                HitFlash::DEFAULT_DURATION,
                original_color,
            ));
            
            let ship_name = name.map(|n| n.as_str()).unwrap_or("Ship");
            info!("Hit flash triggered on {} (original color: {:?})", ship_name, original_color);
        }
    }
}

/// System that updates hit flash effect, interpolating color back to original.
pub fn update_hit_flash_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut HitFlash)>,
) {
    for (entity, mut sprite, mut hit_flash) in &mut query {
        hit_flash.timer.tick(time.delta());

        // Calculate lerp progress (0 = start of flash, 1 = end)
        let progress = hit_flash.timer.fraction();

        // Lerp from flash color to original color
        // At progress 0: full flash color, at progress 1: original color
        let lerped = lerp_color(FLASH_COLOR, hit_flash.original_color, progress);
        sprite.color = lerped;

        // Remove component when flash completes
        if hit_flash.timer.finished() {
            sprite.color = hit_flash.original_color;
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

/// Linearly interpolates between two colors.
fn lerp_color(from: Color, to: Color, t: f32) -> Color {
    let from_srgba = from.to_srgba();
    let to_srgba = to.to_srgba();
    
    Color::srgba(
        from_srgba.red + (to_srgba.red - from_srgba.red) * t,
        from_srgba.green + (to_srgba.green - from_srgba.green) * t,
        from_srgba.blue + (to_srgba.blue - from_srgba.blue) * t,
        from_srgba.alpha + (to_srgba.alpha - from_srgba.alpha) * t,
    )
}
