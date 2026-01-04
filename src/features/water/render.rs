use bevy::prelude::*;
use bevy::sprite::{Material2d, Material2dPlugin};
use bevy::render::render_resource::{AsBindGroup, ShaderRef, PrimitiveTopology};
use bevy::render::mesh::Indices;
use crate::features::water::quadtree::OceanQuadtree;
use crate::features::water::morton::morton_decode;
use crate::components::CombatEntity;
use crate::plugins::core::GameState;

#[derive(Default)]
pub struct OceanRenderPlugin;

impl Plugin for OceanRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<WaterMaterial>::default())
           .add_systems(OnEnter(GameState::Combat), spawn_water_mesh)
           .add_systems(Update, update_water_mesh.run_if(in_state(GameState::Combat)));
    }
}

/// Custom Material for Stylized Water
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(0)]
    pub time: f32,
}

impl Material2d for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water_v3.wgsl".into()
    }
    // Use default vertex shader for now, or custom if needed for displacement.
    // Given the mesh is 2D, displacement might be tricky without custom vertex shader.
    // For now, let's stick to fragment coloring based on vertex attributes.
}

/// Marker component for the water mesh entity.
#[derive(Component)]
pub struct WaterMesh;

fn spawn_water_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    _ocean: Res<OceanQuadtree>,
) {
    let mesh = Mesh::new(PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::default()); 
    let handle = meshes.add(mesh);
    
    let material = materials.add(WaterMaterial {
        color: LinearRgba::from(Color::srgba(0.0, 0.4, 0.8, 1.0)),
        time: 0.0,
    });
    
    commands.spawn((
        Mesh2d(handle),
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, -10.0), // Ensure it's behind ships
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        GlobalTransform::default(),
        WaterMesh,
        CombatEntity,
    ));
}

fn update_water_mesh(
    ocean: Res<OceanQuadtree>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    query: Query<(&Mesh2d, &MeshMaterial2d<WaterMaterial>), With<WaterMesh>>,
    time: Res<Time>,
) {
    let Ok((mesh3d, mat_handle)) = query.get_single() else {
        warn_once!("Water Render: No WaterMesh entity found!");
        return;
    };
    let Some(mesh) = meshes.get_mut(&mesh3d.0) else {
        warn_once!("Water Render: Mesh asset not found!");
        return;
    };
    
    // Update material time
    if let Some(material) = materials.get_mut(&mat_handle.0) {
        material.time = time.elapsed_secs();
    }
    
    // Rebuild mesh data
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut colors = Vec::new();
    // let mut normals = Vec::new(); // Not strictly needed for 2D unless using custom lighting
    
    let domain_size = ocean.domain_size;
    
    // Iterate all active nodes
    for (&(depth, code), cell) in ocean.nodes.iter() {
        let (gx, gy) = morton_decode(code);
        let cell_size = domain_size / (1u32 << depth) as f32;
        
        // Quadtree Grid Space -> World Space
        // (0,0) is usually top-left or bottom-left of grid.
        // Let's assume (0,0) grid is (-Half, -Half) world.
        let grid_dim = 1u32 << depth;
        let normalized_x = gx as f32 / grid_dim as f32;
        let normalized_y = gy as f32 / grid_dim as f32;
        let half_size = domain_size / 2.0;
        let world_x = (normalized_x * domain_size) - half_size + (cell_size / 2.0);
        let world_y = (normalized_y * domain_size) - half_size + (cell_size / 2.0);
        let center = Vec2::new(world_x, world_y);
        
        let z = cell.height; 
        
        let half_w = cell_size / 2.0;
        let v_idx_start = vertices.len() as u32;
        
        // Z-bias for overlapping nodes (if any, though disjoint leaves preferred)
        let z_bias = 0.0001 * depth as f32; 
        let final_z = 0.0 + z_bias; // 2D usually ignores Vertex Z for depth sorting if z-index is used, but internally it's still 3D.

        vertices.push([center.x - half_w, center.y - half_w, final_z]); // BL
        vertices.push([center.x + half_w, center.y - half_w, final_z]); // BR
        vertices.push([center.x + half_w, center.y + half_w, final_z]); // TR
        vertices.push([center.x - half_w, center.y + half_w, final_z]); // TL
        
        // Color encode data: R=Height, G=FlowX, B=FlowY, A=Depth/Lod
        // Height range +/- 10.0 -> 0..1
        let h_norm = (z + 10.0) / 20.0;
        let flow_x_norm = (cell.flow_right + 10.0) / 20.0;
        let flow_y_norm = (cell.flow_down + 10.0) / 20.0;
        
        let col = [h_norm.clamp(0.0, 1.0), flow_x_norm.clamp(0.0, 1.0), flow_y_norm.clamp(0.0, 1.0), depth as f32 / 10.0];
        
        colors.push(col);
        colors.push(col);
        colors.push(col);
        colors.push(col);
        
        indices.push(v_idx_start + 0);
        indices.push(v_idx_start + 1);
        indices.push(v_idx_start + 2);
        
        indices.push(v_idx_start + 2);
        indices.push(v_idx_start + 3);
        indices.push(v_idx_start + 0);
    }
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::water::quadtree::OceanGridPlugin;

    #[test]
    fn test_water_mesh_generation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        
        // Manual resource setup
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<WaterMaterial>>(); // Custom material asset
        app.add_plugins(OceanGridPlugin); 
        
        // Init quadtree data
        let mut ocean = app.world_mut().resource_mut::<OceanQuadtree>();
        ocean.nodes.insert((0, 0), crate::features::water::quadtree::WaterCell::default()); 
        
        // Create material
        let mut materials = app.world_mut().resource_mut::<Assets<WaterMaterial>>();
        let material = materials.add(WaterMaterial {
             color: LinearRgba::WHITE,
             time: 0.0,
        });

        // Spawn mesh entity manually
        let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::new(PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::default()));
        app.world_mut().spawn((
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material),
            WaterMesh,
            CombatEntity,
        ));
        
        // Run update system
        app.add_systems(Update, update_water_mesh);
        app.update();
        
        // Check mesh
        let meshes = app.world().resource::<Assets<Mesh>>();
        let mesh_asset = meshes.get(&mesh).expect("Mesh should exist");
        
        // Check if attribute exists
        if let Some(positions) = mesh_asset.attribute(Mesh::ATTRIBUTE_POSITION) {
             if let bevy::render::mesh::VertexAttributeValues::Float32x3(vals) = positions {
                println!("Vertex count: {}", vals.len());
                assert!(vals.len() > 0, "Mesh should have vertices. Count: {}", vals.len());
            } else {
                panic!("Wrong attribute format");
            }
        } else {
            panic!("No position attribute found - mesh update failed?");
        }
    }
}
