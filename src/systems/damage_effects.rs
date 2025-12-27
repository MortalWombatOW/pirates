//! Damage Visual Effects
//!
//! GPU particle effects for combat damage feedback using bevy_hanabi.

use bevy::prelude::*;
use bevy_hanabi::prelude::*;

// ============================================================================
// Damage Ink Splatter (8.5.4)
// ============================================================================

/// Resource holding splatter effect assets.
#[derive(Resource)]
pub struct SplatterEffectAssets {
    pub splatter_effect: Handle<EffectAsset>,
}

/// Create the damage splatter particle effect.
pub fn create_splatter_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let writer = ExprWriter::new();

    // Particle lifetime: 2.0 seconds for ink stain lingering
    let lifetime = writer.lit(2.0).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Explosive radial position
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(5.0).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Fast initial burst velocity
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(80.0).expr(),
    };

    // Ink splatter gradient: dark sepia to transparent
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.1, 0.08, 0.05, 0.9)); // Very dark ink
    gradient.add_key(0.3, Vec4::new(0.15, 0.12, 0.08, 0.6));
    gradient.add_key(1.0, Vec4::splat(0.0)); // Fade out

    // Size: grows then shrinks (splat spread)
    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(2.0));
    size_gradient.add_key(0.2, Vec3::splat(8.0));
    size_gradient.add_key(1.0, Vec3::splat(0.0));

    // Drag to slow particles down
    let drag = writer.lit(5.0).expr();

    let module = writer.finish();

    // One-shot burst (triggered externally)
    effects.add(
        EffectAsset::new(256, Spawner::once(30.0.into(), false), module)
            .with_name("damage_splatter")
            .init(init_pos)
            .init(init_vel)
            .init(init_lifetime)
            .update(LinearDragModifier::new(drag))
            .render(ColorOverLifetimeModifier { gradient })
            .render(SizeOverLifetimeModifier {
                gradient: size_gradient,
                screen_space_size: false,
            }),
    )
}

/// Initialize splatter effect assets on startup.
pub fn setup_splatter_effects(mut effects: ResMut<Assets<EffectAsset>>, mut commands: Commands) {
    let splatter_effect = create_splatter_effect(&mut effects);
    commands.insert_resource(SplatterEffectAssets { splatter_effect });
}

/// Spawn damage splatter particles on ship hit events.
pub fn spawn_damage_splatter(
    mut commands: Commands,
    mut events: EventReader<crate::events::ShipHitEvent>,
    splatter_assets: Option<Res<SplatterEffectAssets>>,
) {
    let Some(assets) = splatter_assets else { return };

    for event in events.read() {
        // Spawn splatter effect at hit position
        // Note: Particle count is defined in the effect asset (30 particles)
        commands.spawn((
            Name::new("DamageSplatter"),
            ParticleEffectBundle {
                effect: ParticleEffect::new(assets.splatter_effect.clone()),
                transform: Transform::from_translation(event.hit_position.extend(1.0)),
                ..default()
            },
        ));
        
        info!("Spawned damage splatter at {:?} for {:.1} damage", event.hit_position, event.damage);
    }
}
