use std::f32::consts::PI;

use crate::position::*;

pub const A: f32 = 2.0 * PI / 6.0;

pub fn pos_to_point(pos: Pos, r: f32, window_center: (f32, f32)) -> (f32, f32) {
    if pos == 0 {
        return window_center;
    }

    let ring = ring(pos);
    let edge_idx = ring_edge_index(pos);

    if is_at_ring_tip(pos) {
        let ring = ring as f32;

        let (xm, ym) = match edge_idx {
            0 => (0.0, -2.0 * ring),
            1 => (3.0 * ring, -ring),
            2 => (3.0 * ring, ring),
            3 => (0.0, 2.0 * ring),
            4 => (-3.0 * ring, ring),
            5 => (-3.0 * ring, -ring),
            _ => unreachable!(),
        };
        let x = (xm * (r * A.cos())) as f32;
        let y = (ym * (r * A.sin())) as f32;

        (window_center.0 + x, window_center.1 + y)
    } else {
        let ring_offset = ring_offset(ring);
        let ring_pos = pos - ring_offset;
        let tip_offset = ring_pos - edge_idx * ring;
        let tip_pos = ring_offset + edge_idx * ring;
        let ring = tip_offset as f32;

        let tip_point = pos_to_point(tip_pos, r, window_center);

        let (xm, ym) = match (edge_idx + 2) % 6 {
            0 => (0.0, -2.0 * ring),
            1 => (3.0 * ring, -ring),
            2 => (3.0 * ring, ring),
            3 => (0.0, 2.0 * ring),
            4 => (-3.0 * ring, ring),
            5 => (-3.0 * ring, -ring),
            _ => unreachable!(),
        };
        let x = (xm * (r * A.cos())) as f32;
        let y = (ym * (r * A.sin())) as f32;

        (tip_point.0 + x, tip_point.1 + y)
    }
}

pub fn point_to_pos(
    point_x: f32,
    point_y: f32,
    window_center_x: f32,
    window_center_y: f32,
    r: f32,
) -> Option<Pos> {
    todo!();
}
