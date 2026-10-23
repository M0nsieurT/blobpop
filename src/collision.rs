use crate::audio::play_merge_sound;
use crate::blob::{random_blob_val, update_queue_target_positions, Blob};
use macroquad::audio::Sound;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;

pub fn get_ellipse_contact_dist(b1: &Blob, b2: &Blob) -> (f32, Vec2) {
    let dir = (b2.pos - b1.pos).normalize_or_zero();
    let angle = dir.y.atan2(dir.x);

    let r1 = ((angle.cos() * b1.radius_x).powi(2) + (angle.sin() * b1.radius_y).powi(2)).sqrt();
    let r2 = (((angle + PI).cos() * b2.radius_x).powi(2) + ((angle + PI).sin() * b2.radius_y).powi(2)).sqrt();

    (r1 + r2, dir)
}

pub fn deform_blobs_on_contact(b1: &mut Blob, b2: &mut Blob) {
    let dist = b1.pos.distance(b2.pos);
    let (visual_contact_dist, dir) = get_ellipse_contact_dist(b1, b2);

    if dist >= visual_contact_dist * 1.05 || dist < 0.001 {
        return;
    }

    let plane_normal = dir;

    for i in 0..b1.sides {
        let angle = (i as f32 / b1.sides as f32) * 2.0 * PI;
        let rx1 = b1.radius_x + b1.offsets[i];
        let ry1 = b1.radius_y + b1.offsets[i];
        let v_pos = b1.pos + vec2(angle.cos() * rx1, angle.sin() * ry1);

        let proj = (v_pos - b1.pos).dot(plane_normal);
        let max_allowed = dist * 0.52;

        if proj > max_allowed {
            let push_back = proj - max_allowed;
            b1.offsets[i] -= push_back * 0.45;
        }
    }

    for j in 0..b2.sides {
        let angle = (j as f32 / b2.sides as f32) * 2.0 * PI;
        let rx2 = b2.radius_x + b2.offsets[j];
        let ry2 = b2.radius_y + b2.offsets[j];
        let v_pos = b2.pos + vec2(angle.cos() * rx2, angle.sin() * ry2);

        let proj = (v_pos - b2.pos).dot(-plane_normal);
        let max_allowed = dist * 0.52;

        if proj > max_allowed {
            let push_back = proj - max_allowed;
            b2.offsets[j] -= push_back * 0.45;
        }
    }
}

pub fn check_queue_merges(
    queue_blobs: &mut Vec<Blob>,
    slots_y: f32,
    center_x: f32,
    spawn_from_left: &mut bool,
    max_game_val: u32,
    merge_sounds: &HashMap<u32, Sound>,
    score: &mut u32,
) {
    let mut merge_occurred = true;

    while merge_occurred {
        merge_occurred = false;

        let active_indices: Vec<usize> = queue_blobs
            .iter()
            .enumerate()
            .filter(|(_, b)| !b.is_merging && !b.is_growing)
            .map(|(idx, _)| idx)
            .collect();

        if active_indices.len() < 2 {
            break;
        }

        for i in 0..(active_indices.len() - 1) {
            let idx_a = active_indices[i];
            let idx_b = active_indices[i + 1];

            if queue_blobs[idx_a].value == queue_blobs[idx_b].value
                && queue_blobs[idx_a].is_arrived
                && queue_blobs[idx_b].is_arrived
            {
                let merged_val = queue_blobs[idx_a].value * 2;
                play_merge_sound(merge_sounds, merged_val);

                // Addition des valeurs non fusionnées
                let points_a = if queue_blobs[idx_a].is_fused_result { 0 } else { queue_blobs[idx_a].value };
                let points_b = if queue_blobs[idx_b].is_fused_result { 0 } else { queue_blobs[idx_b].value };
                *score += points_a + points_b;

                let target_id = queue_blobs[idx_a].id;
                queue_blobs[idx_a].start_growing(merged_val);
                queue_blobs[idx_b].start_merging(target_id);

                let start_x = if *spawn_from_left { -50.0 } else { screen_width() + 50.0 };
                let new_blob = Blob::new(start_x, slots_y, 0.0, slots_y, random_blob_val(max_game_val));

                if *spawn_from_left {
                    queue_blobs.insert(0, new_blob);
                } else {
                    queue_blobs.push(new_blob);
                }

                *spawn_from_left = !*spawn_from_left;
                update_queue_target_positions(queue_blobs, center_x);

                merge_occurred = true;
                break;
            }
        }
    }
}

pub fn handle_collisions(
    blobs: &mut Vec<Blob>,
    queue_blobs: &mut Vec<Blob>,
    center_x: f32,
    merge_sounds: &HashMap<u32, Sound>,
    score: &mut u32,
) {
    for launched_blob in blobs.iter_mut() {
        if launched_blob.is_merging || launched_blob.is_growing {
            continue;
        }

        for queue_b in queue_blobs.iter_mut() {
            if queue_b.is_merging || queue_b.is_growing {
                continue;
            }

            let dist = launched_blob.pos.distance(queue_b.pos);
            let (visual_contact, n) = get_ellipse_contact_dist(launched_blob, queue_b);

            if dist < visual_contact {
                if launched_blob.value == queue_b.value {
                    let merged_val = queue_b.value * 2;
                    play_merge_sound(merge_sounds, merged_val);

                    // Si un blob est DÉJÀ issu d'une fusion, sa contribution vaut 0
                    let points_launched = if launched_blob.is_fused_result { 0 } else { launched_blob.value };
                    let points_queue = if queue_b.is_fused_result { 0 } else { queue_b.value };
                    *score += points_launched + points_queue;

                    let target_id = queue_b.id;
                    queue_b.start_growing(merged_val);
                    launched_blob.start_merging(target_id);

                    update_queue_target_positions(queue_blobs, center_x);
                    break;
                } else {
                    deform_blobs_on_contact(launched_blob, queue_b);

                    let overlap = visual_contact - dist;
                    let push_dir_x = if launched_blob.pos.x < queue_b.pos.x { 1.0 } else { -1.0 };
                    queue_b.pos.x += push_dir_x * overlap * 0.8;

                    launched_blob.squish_impact(n, 3.5);
                    queue_b.squish_impact(-n, 4.5);
                }
            }
        }
    }

    let iterations = 3;
    for _ in 0..iterations {
        for i in 0..blobs.len() {
            for j in (i + 1)..blobs.len() {
                if blobs[i].is_merging || blobs[i].is_growing || blobs[j].is_merging || blobs[j].is_growing {
                    continue;
                }

                let dist = blobs[i].pos.distance(blobs[j].pos);
                let (visual_contact, n) = get_ellipse_contact_dist(&blobs[i], &blobs[j]);

                if dist < visual_contact {
                    if blobs[i].value == blobs[j].value {
                        let merged_val = blobs[i].value * 2;
                        play_merge_sound(merge_sounds, merged_val);

                        // Seuls les blobs jamais fusionnés ajoutent leurs points
                        let points_i = if blobs[i].is_fused_result { 0 } else { blobs[i].value };
                        let points_j = if blobs[j].is_fused_result { 0 } else { blobs[j].value };
                        *score += points_i + points_j;

                        let target_id = blobs[i].id;
                        blobs[i].start_growing(merged_val);
                        blobs[j].start_merging(target_id);
                    } else {
                        let (first, second) = blobs.split_at_mut(j);
                        deform_blobs_on_contact(&mut first[i], &mut second[0]);

                        let phys_dist = visual_contact * 0.82;
                        if dist < phys_dist {
                            let overlap = phys_dist - dist;
                            blobs[i].pos -= n * overlap * 0.5;
                            blobs[j].pos += n * overlap * 0.5;
                        }

                        blobs[i].squish_impact(-n, 2.0);
                        blobs[j].squish_impact(n, 2.0);

                        let mass_i = blobs[i].mass;
                        let mass_j = blobs[j].mass;

                        let k = 0.5 * (blobs[i].vel - blobs[j].vel).dot(n) / (mass_i + mass_j);
                        blobs[i].vel -= n * k * mass_j;
                        blobs[j].vel += n * k * mass_i;
                    }
                }
            }
        }
    }
}