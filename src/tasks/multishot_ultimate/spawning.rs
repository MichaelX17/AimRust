use bevy::prelude::*;

use super::types::*;

pub fn sample_candidate_pos(rng: &mut UltimateRng) -> Vec3 {
    // Play area directly in front of the backboard wall (WALL_Z = -17.5, width 24m, height 8.5m):
    // Horizontal X: -8.0 to +8.0 (~53 deg flick spread with generous wall margins)
    // Vertical Y: 1.3 to 6.4 (below eye level up to upper wall, clear of floor & top edge)
    // Depth Z: -16.4 to -15.4 (floating just in front of the wall without clipping)
    let x = rng.gen_range(-8.0, 8.0);
    let y = rng.gen_range(1.3, 6.4);
    let z = rng.gen_range(-16.4, -15.4);

    Vec3::new(x, y, z)
}

pub fn pick_separated_pos(existing: &[Vec3], rng: &mut UltimateRng) -> Vec3 {
    let mut best_pos = Vec3::new(0.0, 2.5, -16.0);
    let mut max_min_dist = -1.0f32;

    for _ in 0..30 {
        let candidate = sample_candidate_pos(rng);
        if existing.is_empty() {
            return candidate;
        }

        let mut min_dist = f32::INFINITY;
        for &ex in existing {
            let dist = candidate.distance(ex);
            if dist < min_dist {
                min_dist = dist;
            }
        }

        if min_dist >= MIN_TARGET_SEPARATION {
            return candidate;
        }

        if min_dist > max_min_dist {
            max_min_dist = min_dist;
            best_pos = candidate;
        }
    }

    best_pos
}

/// Samples `(peak_radius, total_lifetime_secs)` matching Aimlabs Multishot Ultimate angular
/// sizing and growth/shrink pace at Z ~ -16m.
pub fn sample_target_size(rng: &mut UltimateRng) -> (f32, f32) {
    let roll = rng.gen_f32();
    if roll < 0.35 {
        // Small: peak radius 0.38m (~2.7 deg diameter), 3.0s lifetime
        (0.38, 3.0)
    } else if roll < 0.75 {
        // Medium: peak radius 0.48m (~3.4 deg diameter), 3.5s lifetime
        (0.48, 3.5)
    } else {
        // Large: peak radius 0.60m (~4.3 deg diameter), 4.0s lifetime
        (0.60, 4.0)
    }
}
