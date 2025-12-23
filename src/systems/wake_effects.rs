//! Ship Wake Ink Trail Effects
//!
//! GPU particle effects for ship wake trails using bevy_hanabi.

use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::components::ship::Ship;

/// Marker component for entities with wake emitters attached.
#[derive(Component)]
pub struct HasWakeEmitter;

/// Resource holding wake effect assets.
#[derive(Resource)]
pub struct WakeEffectAssets {
    pub wake_effect: Handle<EffectAsset>,
}

/// Create the ship wake particle effect.
pub fn create_wake_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let writer = ExprWriter::new();

    // Particle lifetime: 1.5 seconds for trail length balance
    let lifetime = writer.lit(1.5).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Spawn position: slight random offset for ribbon width
    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        radius: writer.lit(8.0).expr(),
        dimension: ShapeDimension::Surface,
    };

    // Minimal initial velocity (particles stay where spawned)
    let init_vel = SetVelocityCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        speed: writer.lit(2.0).expr(),
    };

    // Ink gradient: sepia to transparent
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.2, 0.15, 0.1, 0.5)); // Dark sepia, semi-transparent
    gradient.add_key(0.5, Vec4::new(0.3, 0.25, 0.2, 0.25));
    gradient.add_key(1.0, Vec4::splat(0.0)); // Fade out

    // Size: small particles (use Vec3 for bevy_hanabi 0.14)
    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(4.0));
    size_gradient.add_key(1.0, Vec3::splat(2.0));

    let module = writer.finish();

    effects.add(
        EffectAsset::new(512, Spawner::rate(25.0.into()), module)
            .with_name("ship_wake")
            .init(init_pos)
            .init(init_vel)
            .init(init_lifetime)
            .render(ColorOverLifetimeModifier { gradient })
            .render(SizeOverLifetimeModifier {
                gradient: size_gradient,
                screen_space_size: false,
            }),
    )
}

/// Initialize wake effect assets on startup.
pub fn setup_wake_effects(mut effects: ResMut<Assets<EffectAsset>>, mut commands: Commands) {
    let wake_effect = create_wake_effect(&mut effects);
    commands.insert_resource(WakeEffectAssets { wake_effect });
}

/// Attach wake emitters to ships that are moving.
pub fn attach_wake_to_moving_ships(
    mut commands: Commands,
    ships_without_wake: Query<Entity, (With<Ship>, Without<HasWakeEmitter>)>,
    wake_assets: Option<Res<WakeEffectAssets>>,
) {
    let Some(assets) = wake_assets else { return };

    for ship_entity in ships_without_wake.iter() {
        // Spawn wake emitter as child of ship
        let wake_entity = commands
            .spawn((
                Name::new("ShipWake"),
                ParticleEffectBundle {
                    effect: ParticleEffect::new(assets.wake_effect.clone()),
                    transform: Transform::from_translation(Vec3::new(0.0, -20.0, -1.0)),
                    ..default()
                },
            ))
            .id();

        commands.entity(ship_entity).add_child(wake_entity);
        commands.entity(ship_entity).insert(HasWakeEmitter);
    }
}
