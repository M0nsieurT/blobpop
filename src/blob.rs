use macroquad::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;

pub const MERGE_DURATION: f32 = 0.30;

#[derive(Clone)]
pub struct Blob {
    pub id: u64,
    pub pos: Vec2,
    pub target_pos: Vec2,
    pub vel: Vec2,
    pub value: u32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub radius: f32,
    pub mass: f32,
    pub is_launched: bool,
    pub is_arrived: bool,
    pub sides: usize,
    pub offsets: Vec<f32>,
    pub offset_vels: Vec<f32>,

    // Si true, ce blob est déjà issu d'une fusion (sa valeur vaut 0 dans les futures additions)
    pub is_fused_result: bool,

    pub is_merging: bool,
    pub merge_target_id: Option<u64>,
    pub merge_timer: f32,

    pub is_growing: bool,
    pub grow_timer: f32,
    pub next_value: u32,
    pub start_radius_x: f32,
    pub start_radius_y: f32,
    pub target_radius_x: f32,
    pub target_radius_y: f32,
}

impl Blob {
    pub fn new(start_x: f32, start_y: f32, target_x: f32, target_y: f32, value: u32) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let base_r = 16.0 + (value as f32).log2() * 5.0;
        let radius_x = base_r * 1.35;
        let radius_y = base_r * 0.75;
        let radius = (radius_x + radius_y) * 0.5;
        let mass = radius * radius * 0.1;
        let log_val = (value as f32).log2().max(0.0) as usize;
        let sides = 28 + log_val * 4;

        let is_at_target = vec2(start_x, start_y).distance(vec2(target_x, target_y)) < 1.0;

        Self {
            id,
            pos: vec2(start_x, start_y),
            target_pos: vec2(target_x, target_y),
            vel: Vec2::ZERO,
            value,
            radius_x,
            radius_y,
            radius,
            mass,
            is_launched: false,
            is_arrived: is_at_target,
            sides,
            offsets: vec![0.0; sides],
            offset_vels: vec![0.0; sides],
            is_fused_result: false,
            is_merging: false,
            merge_target_id: None,
            merge_timer: 0.0,
            is_growing: false,
            grow_timer: 0.0,
            next_value: value,
            start_radius_x: radius_x,
            start_radius_y: radius_y,
            target_radius_x: radius_x,
            target_radius_y: radius_y,
        }
    }

    pub fn start_merging(&mut self, target_id: u64) {
        self.is_merging = true;
        self.merge_target_id = Some(target_id);
        self.merge_timer = MERGE_DURATION;
        self.vel = Vec2::ZERO;
    }

    pub fn start_growing(&mut self, new_value: u32) {
        self.next_value = new_value;
        self.is_growing = true;
        self.grow_timer = MERGE_DURATION;
        self.start_radius_x = self.radius_x;
        self.start_radius_y = self.radius_y;

        let base_r = 16.0 + (new_value as f32).log2() * 5.0;
        self.target_radius_x = base_r * 1.35;
        self.target_radius_y = base_r * 0.75;
    }

    pub fn finalize_growth(&mut self) {
        self.value = self.next_value;
        self.is_growing = false;
        self.is_fused_result = true;
        self.radius_x = self.target_radius_x;
        self.radius_y = self.target_radius_y;
        self.radius = (self.radius_x + self.radius_y) * 0.5;
        self.mass = self.radius * self.radius * 0.1;
        let log_val = (self.value as f32).log2().max(0.0) as usize;
        self.sides = 28 + log_val * 4;
        self.offsets = vec![0.0; self.sides];
        self.offset_vels = vec![0.0; self.sides];
        self.squish_impact(vec2(0.0, 1.0), 5.0);
    }

    pub fn update(&mut self, dt: f32, bounds: Rect, all_blobs_pos: &HashMap<u64, Vec2>) -> bool {
        if self.is_merging {
            self.merge_timer -= dt;
            if let Some(target_id) = self.merge_target_id {
                if let Some(&target_pos) = all_blobs_pos.get(&target_id) {
                    self.pos += (target_pos - self.pos) * 8.0 * dt;
                }
            }
            return self.merge_timer <= 0.0;
        }

        if self.is_growing {
            self.grow_timer -= dt;
            let progress = (1.0 - (self.grow_timer / MERGE_DURATION).max(0.0)).clamp(0.0, 1.0);
            self.radius_x = self.start_radius_x + (self.target_radius_x - self.start_radius_x) * progress;
            self.radius_y = self.start_radius_y + (self.target_radius_y - self.start_radius_y) * progress;
            self.radius = (self.radius_x + self.radius_y) * 0.5;

            if self.grow_timer <= 0.0 {
                self.finalize_growth();
            }
        }

        let speed = self.vel.length();
        let max_offset = self.radius * 0.45;

        for i in 0..self.sides {
            let angle = (i as f32 / self.sides as f32) * 2.0 * PI;
            let vertex_dir = vec2(angle.cos(), angle.sin());

            let spring_force = -self.offsets[i] * 70.0;
            self.offset_vels[i] += spring_force * dt;

            if self.is_launched {
                let gravity_squish = vertex_dir.y.powi(2) * -25.0 + vertex_dir.x.powi(2) * 15.0;
                self.offset_vels[i] += gravity_squish * dt;
            }

            self.offset_vels[i] *= 0.55;
            self.offsets[i] += self.offset_vels[i] * dt;

            if speed > 15.0 {
                let move_dir = self.vel.normalize_or_zero();
                self.offsets[i] += move_dir.dot(vertex_dir) * (speed * 0.001);
            }

            self.offsets[i] = self.offsets[i].clamp(-max_offset, max_offset);
        }

        if !self.is_launched {
            self.pos.x = self.pos.x + (self.target_pos.x - self.pos.x) * 4.0 * dt;
            self.pos.y = self.target_pos.y;

            if (self.pos.x - self.target_pos.x).abs() < 1.0 {
                self.pos.x = self.target_pos.x;
                self.is_arrived = true;
            } else {
                self.is_arrived = false;
            }
            return false;
        }

        let gravity = 350.0;
        self.vel.y += gravity * dt;

        self.pos += self.vel * dt;
        self.vel -= self.vel * 1.5 * dt;

        for i in 0..self.sides {
            let angle = (i as f32 / self.sides as f32) * 2.0 * PI;
            let rx = self.radius_x + self.offsets[i];
            let ry = self.radius_y + self.offsets[i];
            
            let vx = self.pos.x + angle.cos() * rx;
            let vy = self.pos.y + angle.sin() * ry;

            if vx < bounds.x {
                let overlap = bounds.x - vx;
                self.offsets[i] -= overlap * angle.cos().abs() * 0.6;
            }
            if vx > bounds.x + bounds.w {
                let overlap = vx - (bounds.x + bounds.w);
                self.offsets[i] -= overlap * angle.cos().abs() * 0.6;
            }
            if vy < bounds.y {
                let overlap = bounds.y - vy;
                self.offsets[i] -= overlap * angle.sin().abs() * 0.6;
            }
            if vy > bounds.y + bounds.h {
                let overlap = vy - (bounds.y + bounds.h);
                self.offsets[i] -= overlap * angle.sin().abs() * 0.6;
            }
        }

        if self.pos.x - self.radius_x < bounds.x {
            self.pos.x = bounds.x + self.radius_x;
            self.vel.x *= -0.3;
        }
        if self.pos.x + self.radius_x > bounds.x + bounds.w {
            self.pos.x = bounds.x + bounds.w - self.radius_x;
            self.vel.x *= -0.3;
        }
        if self.pos.y - self.radius_y < bounds.y {
            self.pos.y = bounds.y + self.radius_y;
            self.vel.y *= -0.2;
        }
        if self.pos.y + self.radius_y > bounds.y + bounds.h {
            self.pos.y = bounds.y + bounds.h - self.radius_y;
            self.vel.y *= -0.1;
            self.vel.x *= 0.8;
        }

        false
    }

    pub fn squish_impact(&mut self, impact_dir: Vec2, intensity: f32) {
        for i in 0..self.sides {
            let angle = (i as f32 / self.sides as f32) * 2.0 * PI;
            let vertex_dir = vec2(angle.cos(), angle.sin());
            let dot = vertex_dir.dot(impact_dir);
            if dot > 0.0 {
                self.offset_vels[i] -= dot * intensity;
            }
        }
    }

    pub fn draw_density_field(&self) {
        let scale_factor = if self.is_merging {
            (self.merge_timer / MERGE_DURATION).max(0.0)
        } else {
            1.0
        };

        let steps = 3;
        for s in (1..=steps).rev() {
            let factor = s as f32 / steps as f32;
            let alpha = 0.3 + factor * 0.5;
            let scale = (0.9 + (1.0 - factor) * 0.2) * scale_factor;

            let mut vertices = Vec::with_capacity(self.sides);
            for i in 0..self.sides {
                let angle = (i as f32 / self.sides as f32) * 2.0 * PI;
                let rx = (self.radius_x + self.offsets[i]) * scale;
                let ry = (self.radius_y + self.offsets[i]) * scale;
                
                let vx = self.pos.x + angle.cos() * rx;
                let vy = self.pos.y + angle.sin() * ry;
                vertices.push(vec2(vx, vy));
            }

            for i in 0..self.sides {
                let next_i = (i + 1) % self.sides;
                draw_triangle(
                    self.pos,
                    vertices[i],
                    vertices[next_i],
                    Color::new(1.0, 1.0, 1.0, alpha),
                );
            }
        }
    }
}

pub fn random_blob_val(max_game_val: u32) -> u32 {
    let allowed_max = (max_game_val / 4).max(4);
    let mut options = Vec::new();
    let mut val = 1;
    while val <= allowed_max {
        options.push(val);
        val *= 2;
    }

    let roll = rand::gen_range(0, 100);
    if options.len() == 3 {
        if roll < 50 { 1 } else if roll < 80 { 2 } else { 4 }
    } else if options.len() > 3 {
        if roll < 40 {
            1
        } else if roll < 70 {
            2
        } else if roll < 88 {
            4
        } else {
            let upper_idx = rand::gen_range(3, options.len());
            options[upper_idx]
        }
    } else {
        if roll < 60 { 1 } else { 2 }
    }
}

pub fn update_queue_target_positions(queue_blobs: &mut [Blob], center_x: f32) {
    let active_blobs: Vec<&mut Blob> = queue_blobs.iter_mut().filter(|b| !b.is_merging).collect();
    if active_blobs.is_empty() {
        return;
    }

    let mut total_width = 0.0;
    for b in active_blobs.iter() {
        total_width += b.radius_x * 2.0;
    }

    let mut current_x = center_x - (total_width * 0.5);

    for b in active_blobs {
        b.target_pos.x = current_x + b.radius_x;
        current_x += b.radius_x * 2.0;
    }
}