mod audio;
mod blob;
mod collision;
mod shaders;

use audio::*;
use blob::*;
use collision::*;
use shaders::*;

use macroquad::audio::{load_sound_from_bytes, play_sound_once};
use macroquad::prelude::*;
use std::collections::HashMap;

#[macroquad::main("Blob Pop")]
pub async fn main() {
    let material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![UniformDesc::new("u_resolution", UniformType::Float2)],
            ..Default::default()
        },
    )
    .unwrap();

    let sound_select = load_sound_from_bytes(&generate_select_sound()).await.unwrap();
    let sound_launch = load_sound_from_bytes(&generate_launch_sound()).await.unwrap();

    let mut merge_sounds = HashMap::new();
    let mut val = 2u32;
    while val <= 8192 {
        let sound = load_sound_from_bytes(&generate_merge_sound(val)).await.unwrap();
        merge_sounds.insert(val, sound);
        val *= 2;
    }

    let mut target_w = screen_width();
    let mut target_h = screen_height();
    let mut rt = render_target(target_w as u32, target_h as u32);
    rt.texture.set_filter(FilterMode::Nearest);

    let board = Rect::new(100.0, 40.0, screen_width() - 200.0, screen_height() - 70.0);

    let mut blobs: Vec<Blob> = Vec::new();
    let mut score: u32 = 0;

    let center_x = board.x + board.w / 2.0;
    let slots_y = board.y + 70.0;

    let mut queue_blobs = vec![
        Blob::new(center_x, slots_y, center_x, slots_y, random_blob_val(4)),
        Blob::new(center_x, slots_y, center_x, slots_y, random_blob_val(4)),
        Blob::new(center_x, slots_y, center_x, slots_y, random_blob_val(4)),
    ];
    update_queue_target_positions(&mut queue_blobs, center_x);

    let mut selected_slot: usize = 1;
    let mut is_aiming = false;
    let mut spawn_from_left = true;

    let lcd_bg_color = Color::new(0.607, 0.737, 0.058, 1.0);

    loop {
        let dt = get_frame_time().min(0.033);
        let mouse_pos = vec2(mouse_position().0, mouse_position().1);

        let mut max_game_val = 4;
        for b in &blobs {
            if b.value > max_game_val {
                max_game_val = b.value;
            }
        }
        for b in &queue_blobs {
            if b.value > max_game_val {
                max_game_val = b.value;
            }
        }

        if screen_width() != target_w || screen_height() != target_h {
            target_w = screen_width();
            target_h = screen_height();
            rt = render_target(target_w as u32, target_h as u32);
            rt.texture.set_filter(FilterMode::Nearest);
        }

        let mut all_blobs_pos = HashMap::new();
        for b in &queue_blobs {
            all_blobs_pos.insert(b.id, b.pos);
        }
        for b in &blobs {
            all_blobs_pos.insert(b.id, b.pos);
        }

        queue_blobs.retain_mut(|b| !b.update(dt, board, &all_blobs_pos));

        check_queue_merges(
            &mut queue_blobs,
            slots_y,
            center_x,
            &mut spawn_from_left,
            max_game_val,
            &merge_sounds,
            &mut score,
        );

        if is_mouse_button_pressed(MouseButton::Left) {
            for (idx, b) in queue_blobs.iter().enumerate() {
                if !b.is_merging && !b.is_growing && mouse_pos.distance(b.pos) < b.radius * 1.5 {
                    selected_slot = idx;
                    is_aiming = true;
                    play_sound_once(&sound_select);
                    break;
                }
            }
        }

        if is_aiming && is_mouse_button_released(MouseButton::Left) {
            is_aiming = false;
            if selected_slot < queue_blobs.len() {
                let current_pos = queue_blobs[selected_slot].pos;
                let launch_vector = mouse_pos - current_pos;

                if launch_vector.length() > 10.0 {
                    play_sound_once(&sound_launch);

                    let mut launched_blob = queue_blobs[selected_slot].clone();
                    launched_blob.vel = (launch_vector * 7.0).clamp_length_max(1000.0);
                    launched_blob.is_launched = true;
                    blobs.push(launched_blob);

                    let new_val = random_blob_val(max_game_val);
                    let start_x = if spawn_from_left { -50.0 } else { screen_width() + 50.0 };

                    queue_blobs.remove(selected_slot);
                    let new_blob = Blob::new(start_x, slots_y, 0.0, slots_y, new_val);

                    if spawn_from_left {
                        queue_blobs.insert(0, new_blob);
                    } else {
                        queue_blobs.push(new_blob);
                    }

                    spawn_from_left = !spawn_from_left;
                    update_queue_target_positions(&mut queue_blobs, center_x);

                    selected_slot = 1;
                }
            }
        }

        for b in blobs.iter_mut() {
            b.update(dt, board, &all_blobs_pos);
        }

        blobs.retain(|b| !(b.is_merging && b.merge_timer <= 0.0));

        handle_collisions(&mut blobs, &mut queue_blobs, center_x, &merge_sounds, &mut score);

        let render_camera = Camera2D {
            render_target: Some(rt.clone()),
            target: vec2(screen_width() / 2.0, screen_height() / 2.0),
            zoom: vec2(2.0 / screen_width(), -2.0 / screen_height()),
            ..Default::default()
        };

        set_camera(&render_camera);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));

        for b in &queue_blobs {
            b.draw_density_field();
        }
        for blob in &blobs {
            blob.draw_density_field();
        }

        set_default_camera();
        clear_background(lcd_bg_color);

        material.set_uniform("u_resolution", vec2(screen_width() / 2.0, screen_height() / 2.0));

        gl_use_material(&material);
        draw_texture_ex(
            &rt.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();

        draw_rectangle_lines(board.x, board.y, board.w, board.h, 4.0, Color::new(0.058, 0.220, 0.058, 1.0));

        let score_txt = format!("SCORE: {}", score);
        draw_text(&score_txt, board.x + 10.0, board.y + 25.0, 22.0, Color::new(0.058, 0.220, 0.058, 1.0));

        if selected_slot < queue_blobs.len() {
            let current_b = &queue_blobs[selected_slot];
            if !current_b.is_merging {
                draw_ellipse_lines(
                    current_b.pos.x,
                    current_b.pos.y,
                    current_b.radius_x + 6.0,
                    current_b.radius_y + 6.0,
                    0.0,
                    3.0,
                    BLACK,
                );

                if is_aiming {
                    let aim_dir = (mouse_pos - current_b.pos).normalize_or_zero();
                    let force_len = (mouse_pos - current_b.pos).length().min(150.0);
                    let aim_end = current_b.pos + aim_dir * force_len;

                    draw_line(current_b.pos.x, current_b.pos.y, aim_end.x, aim_end.y, 4.0, BLACK);
                    draw_circle(aim_end.x, aim_end.y, 5.0, BLACK);
                }
            }
        }

        let draw_blob_text = |b: &Blob| {
            if b.is_merging {
                return;
            }
            let display_val = if b.is_growing { b.next_value } else { b.value };
            let txt = format!("{}", display_val);
            let font_size = 20.0;
            let center = get_text_center(&txt, Option::None, font_size as u16, 1.0, 0.0);
            draw_text(&txt, b.pos.x - center.x, b.pos.y + center.y, font_size, BLACK);
        };

        for b in &queue_blobs {
            draw_blob_text(b);
        }
        for blob in &blobs {
            draw_blob_text(blob);
        }

        next_frame().await;
    }
}