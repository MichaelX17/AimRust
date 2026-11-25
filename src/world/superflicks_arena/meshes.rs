//! Procedural geometry builders for the SuperFlicks 4-level tower arena.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

/// Flat circular ring (annulus) lying in the XZ plane at Y = 0.
pub fn build_annulus_mesh(inner_r: f32, outer_r: f32, segments: u32, face_up: bool) -> Mesh {
    let mut positions = Vec::with_capacity(((segments + 1) * 2) as usize);
    let mut normals = Vec::with_capacity(((segments + 1) * 2) as usize);
    let mut uvs = Vec::with_capacity(((segments + 1) * 2) as usize);
    let mut indices = Vec::with_capacity((segments * 6) as usize);

    let ny = if face_up { 1.0 } else { -1.0 };

    for i in 0..=segments {
        let u = i as f32 / segments as f32;
        let a = u * TAU;
        let (sa, ca) = a.sin_cos();

        positions.push([inner_r * sa, 0.0, -inner_r * ca]);
        normals.push([0.0, ny, 0.0]);
        uvs.push([u * 4.0, 0.0]);

        positions.push([outer_r * sa, 0.0, -outer_r * ca]);
        normals.push([0.0, ny, 0.0]);
        uvs.push([u * 4.0, 1.0]);
    }

    for i in 0..segments {
        let in0 = i * 2;
        let out0 = in0 + 1;
        let in1 = in0 + 2;
        let out1 = in0 + 3;

        if face_up {
            indices.extend_from_slice(&[in0, in1, out0, in1, out1, out0]);
        } else {
            indices.extend_from_slice(&[in0, out0, in1, in1, out0, out1]);
        }
    }

    build_mesh(positions, normals, uvs, indices)
}

/// 3D curved horizontal ledge / catwalk arc with top surface at Y = 0 and bottom at Y = -thickness.
/// Angles are centered on -Z (0 rad = straight ahead, +angle = right +X, -angle = left -X).
pub fn build_curved_ledge_mesh(
    inner_r: f32,
    outer_r: f32,
    thickness: f32,
    start_angle: f32,
    end_angle: f32,
    segments: u32,
) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let span = end_angle - start_angle;
    let is_closed = span.abs() >= TAU - 0.01;

    // Helper to append a single quad with a uniform normal.
    let mut push_quad = |p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], n: [f32; 3]| {
        let base = positions.len() as u32;
        positions.extend_from_slice(&[p0, p1, p2, p3]);
        normals.extend_from_slice(&[n, n, n, n]);
        uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    };

    for i in 0..segments {
        let t0 = i as f32 / segments as f32;
        let t1 = (i + 1) as f32 / segments as f32;
        let a0 = start_angle + t0 * span;
        let a1 = start_angle + t1 * span;
        let a_mid = 0.5 * (a0 + a1);

        let (s0, c0) = a0.sin_cos();
        let (s1, c1) = a1.sin_cos();
        let (sm, cm) = a_mid.sin_cos();

        let in_top0 = [inner_r * s0, 0.0, -inner_r * c0];
        let in_top1 = [inner_r * s1, 0.0, -inner_r * c1];
        let out_top0 = [outer_r * s0, 0.0, -outer_r * c0];
        let out_top1 = [outer_r * s1, 0.0, -outer_r * c1];

        let in_bot0 = [inner_r * s0, -thickness, -inner_r * c0];
        let in_bot1 = [inner_r * s1, -thickness, -inner_r * c1];
        let out_bot0 = [outer_r * s0, -thickness, -outer_r * c0];
        let out_bot1 = [outer_r * s1, -thickness, -outer_r * c1];

        // Top (+Y)
        push_quad(in_top0, in_top1, out_top1, out_top0, [0.0, 1.0, 0.0]);
        // Bottom (-Y)
        push_quad(in_bot0, out_bot0, out_bot1, in_bot1, [0.0, -1.0, 0.0]);
        // Inner curved wall (facing center)
        push_quad(in_bot0, in_bot1, in_top1, in_top0, [-sm, 0.0, cm]);
        // Outer curved wall (facing outward)
        push_quad(out_bot0, out_top0, out_top1, out_bot1, [sm, 0.0, -cm]);
    }

    if !is_closed {
        let (ss, cs) = start_angle.sin_cos();
        let start_n = [-cs, 0.0, -ss];
        push_quad(
            [inner_r * ss, -thickness, -inner_r * cs],
            [inner_r * ss, 0.0, -inner_r * cs],
            [outer_r * ss, 0.0, -outer_r * cs],
            [outer_r * ss, -thickness, -outer_r * cs],
            start_n,
        );

        let (se, ce) = end_angle.sin_cos();
        let end_n = [ce, 0.0, se];
        push_quad(
            [inner_r * se, -thickness, -inner_r * ce],
            [outer_r * se, -thickness, -outer_r * ce],
            [outer_r * se, 0.0, -outer_r * ce],
            [inner_r * se, 0.0, -inner_r * ce],
            end_n,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

/// Inward-facing vertically fluted cylindrical shaft wall (Y = 0..height).
/// Alternates smooth architectural bays with fluted vertical ribs around the circumference.
pub fn build_fluted_shaft_mesh(
    radius: f32,
    height: f32,
    segments: u32,
    flute_depth: f32,
) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let radius_at = |angle: f32| -> f32 {
        // 8 architectural bays; narrow fluted pilaster strips at the edges of each bay
        let bay_phase = (angle / TAU * 8.0).fract();
        if !(0.12..=0.88).contains(&bay_phase) {
            radius - flute_depth * (angle * 96.0).cos().abs()
        } else {
            radius
        }
    };

    for i in 0..segments {
        let u0 = i as f32 / segments as f32;
        let u1 = (i + 1) as f32 / segments as f32;
        let a0 = u0 * TAU;
        let a1 = u1 * TAU;

        let r0 = radius_at(a0);
        let r1 = radius_at(a1);
        let p_b0 = Vec3::new(r0 * a0.sin(), 0.0, -r0 * a0.cos());
        let p_b1 = Vec3::new(r1 * a1.sin(), 0.0, -r1 * a1.cos());
        let p_t0 = Vec3::new(r0 * a0.sin(), height, -r0 * a0.cos());
        let p_t1 = Vec3::new(r1 * a1.sin(), height, -r1 * a1.cos());

        let n = (p_b1 - p_b0).cross(p_t0 - p_b0).normalize_or_zero();
        let base = positions.len() as u32;
        positions.extend_from_slice(&[p_b0.into(), p_b1.into(), p_t1.into(), p_t0.into()]);
        normals.extend_from_slice(&[n.into(); 4]);
        uvs.extend_from_slice(&[[u0, 0.0], [u1, 0.0], [u1, 1.0], [u0, 1.0]]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    build_mesh(positions, normals, uvs, indices)
}

/// Flat-shaded inward regular polygonal wall prism (e.g., 8-sided octagonal atrium).
/// Vertex 0 lies along -Z so two crisp flat facets meet down the forward centerline.
pub fn build_polygonal_wall_mesh(radius: f32, height: f32, sides: u32) -> Mesh {
    let mut positions = Vec::with_capacity((sides * 4) as usize);
    let mut normals = Vec::with_capacity((sides * 4) as usize);
    let mut uvs = Vec::with_capacity((sides * 4) as usize);
    let mut indices = Vec::with_capacity((sides * 6) as usize);

    for i in 0..sides {
        let a0 = i as f32 / sides as f32 * TAU;
        let a1 = (i + 1) as f32 / sides as f32 * TAU;

        let p_b0 = Vec3::new(radius * a0.sin(), 0.0, -radius * a0.cos());
        let p_b1 = Vec3::new(radius * a1.sin(), 0.0, -radius * a1.cos());
        let p_t0 = Vec3::new(radius * a0.sin(), height, -radius * a0.cos());
        let p_t1 = Vec3::new(radius * a1.sin(), height, -radius * a1.cos());

        let n = (p_b1 - p_b0).cross(p_t0 - p_b0).normalize_or_zero();
        let base = positions.len() as u32;
        positions.extend_from_slice(&[p_b0.into(), p_b1.into(), p_t1.into(), p_t0.into()]);
        normals.extend_from_slice(&[n.into(); 4]);
        uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    build_mesh(positions, normals, uvs, indices)
}

/// Flat annulus whose inner edge is circular (`inner_r`) and outer edge matches a regular
/// `outer_sides`-sided polygon of circumradius `outer_r` (vertex 0 at -Z).
pub fn build_polygonal_annulus_mesh(
    inner_r: f32,
    outer_r: f32,
    outer_sides: u32,
    subdiv_per_side: u32,
    face_up: bool,
) -> Mesh {
    let total = outer_sides * subdiv_per_side;
    let mut positions = Vec::with_capacity(((total + 1) * 2) as usize);
    let mut normals = Vec::with_capacity(((total + 1) * 2) as usize);
    let mut uvs = Vec::with_capacity(((total + 1) * 2) as usize);
    let mut indices = Vec::with_capacity((total * 6) as usize);

    let ny = if face_up { 1.0 } else { -1.0 };

    for side in 0..outer_sides {
        let a_start = side as f32 / outer_sides as f32 * TAU;
        let a_end = (side + 1) as f32 / outer_sides as f32 * TAU;
        let c0 = Vec2::new(outer_r * a_start.sin(), -outer_r * a_start.cos());
        let c1 = Vec2::new(outer_r * a_end.sin(), -outer_r * a_end.cos());

        for sub in 0..subdiv_per_side {
            let t0 = sub as f32 / subdiv_per_side as f32;
            let t1 = (sub + 1) as f32 / subdiv_per_side as f32;
            let ang0 = a_start + t0 * (a_end - a_start);
            let ang1 = a_start + t1 * (a_end - a_start);

            let in0 = [inner_r * ang0.sin(), 0.0, -inner_r * ang0.cos()];
            let in1 = [inner_r * ang1.sin(), 0.0, -inner_r * ang1.cos()];
            let o0 = c0.lerp(c1, t0);
            let o1 = c0.lerp(c1, t1);
            let out0 = [o0.x, 0.0, o0.y];
            let out1 = [o1.x, 0.0, o1.y];

            let base = positions.len() as u32;
            positions.extend_from_slice(&[in0, in1, out1, out0]);
            normals.extend_from_slice(&[[0.0, ny, 0.0]; 4]);
            uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
            if face_up {
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            } else {
                indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
            }
        }
    }

    build_mesh(positions, normals, uvs, indices)
}

fn build_mesh(
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
