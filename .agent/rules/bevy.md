# Bevy 0.15 Best Practices

To ensure high performance and maintainable code in Bevy 0.15:

## ECS Optimization
1. **Change Detection**: Use `Changed<T>` and `Added<T>` filters in queries to avoid processing unchanged data.
2. **Minimal Commands**: Prefer direct component mutation (`mut component`) over `Commands` whenever possible. Use `Commands` only for entity spawning/despawning or structural changes (adding/removing components).
3. **Query Filters**: Use `With<T>` and `Without<T>` effectively to narrow down system execution.

## Architectural Patterns
1. **Plugin Pattern**: Encapsulate logic in granular Bevy `Plugin` structs. Every major feature (Combat, Economy, etc.) must be its own plugin.
2. **State Management**: Use `App::init_state::<T>` and `NextState<T>` for game flow. Gate systems with `.run_if(in_state(T::Variant))`.
3. **Resources vs. Components**: Use `Resource` for global singletons (e.g., `WorldClock`) and `Component` for data per entity.
4. **Events**: Use `EventReader` and `EventWriter` for decoupled communication between systems.

## 2D Specifics
1. **Z-Sorting**: Use the `z` coordinate in `Transform` to manage layering of 2D sprites.
2. **Physics**: Use `avian2d` (successor to `bevy_xpbd`). Ensure `RigidBody` and `Collider` components are used correctly.
3. **Camera**: Use the new `Camera2d` component instead of the deprecated `Camera2dBundle`.
