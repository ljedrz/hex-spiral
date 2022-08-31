//! Convert spiral coordinates to and from cube (q, r, s) coordinates.

use crate::position::{ring, ring_offset};

/// Cube coordinate system for hex grid.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Cube {
    q: i32,
    r: i32,
    s: i32,
}

impl Cube {
    fn new(q: i32, r: i32, s: i32) -> Self {
        Cube { q, r, s }
    }

    // Find the largest absolute value of cube coordinate components.
    fn abs_largest(&self) -> i32 {
        [self.q.abs(), self.r.abs(), self.s.abs()]
            .into_iter()
            .max()
            .unwrap()
    }

    // Find the sum of cube coordinate components.
    fn component_sum(&self) -> i32 {
        self.q + self.r + self.s
    }
}

/// Convert spiral hex coordinate x to cube coords (q,r,s).
/// See: https://www.redblobgames.com/grids/hexagons/
/// for a definition of cube coords.
pub fn spiral_to_cube(x: usize) -> Cube {
    // The origin is a special case: return (0,0,0)
    if x == 0 {
        return Cube::default();
    }

    // Find the ring index and ring-offset for this spiral
    let ring_index = ring(x) as f32;
    let ring_offset = ring_offset(ring_index as usize) as f32;

    // Calculate q and r
    let q = growing_trunc_tri(x as f32, ring_index, ring_offset, 0.0);
    let r = growing_trunc_tri(x as f32, ring_index, ring_offset, 4.0);

    // Could alternatively manually calculate s as:
    // let s = growing_trunc_tri(x, ring_offset, p, ring_index, -4.0);
    let s = -q - r;

    Cube::new(q, r, s)
}

/// Calculate a spiral hex coordinate for an input (q,r,s) in cube coordinates.
pub fn cube_to_spiral(coord: Cube) -> Result<usize, &'static str> {
    // The origin is a special case, return 0.
    if coord == Cube::default() {
        return Ok(0);
    }

    // Make sure we've been passed a valid cube coordinate. The components should sum to 0.
    if coord.component_sum() != 0 {
        return Err("q + r + s != 0");
    }

    // Find the ring index based on the maximum absolute value of q, r or s.
    let ring_index = coord.abs_largest() as usize;

    let ring_offset = ring_offset(ring_index);

    // We now know approximately where we are in the truncated triangle wave.
    // If we start at x = ring_offset and calculate q,r,s values from this point up to
    // x = (ring_offset + ring_index * 6), we should find matching q, r, s values for some value of x.

    let x = ring_offset..(ring_offset + ring_index * 6);

    match x
        .into_iter()
        .map(|v| (v, spiral_to_cube(v)))
        .find(|(_, c)| *c == coord)
        .map(|(x, _)| x)
    {
        Some(value) => Ok(value),
        None => Err("Couldn't find a solution"),
    }
}

/// Calculates y = f(x) where f is a truncated triangle wave of initial period, p = 6, and amplitude, a = 1.5
/// The amplitude and period increase each cycle.
/// - c is the cycle number that we're currently on (i.e. c=1 for the first cycle, and so on)
/// - x_prime is the value of x that this cycle began on
/// - phi is a phase shift in the triangle wave
fn growing_trunc_tri(x: f32, c: f32, x_prime: f32, phi: f32) -> i32 {
    // The base period of the triangle wave during cycle 1 (the number of sides a hexagon has)
    let p = 6.0;

    // How far along we are in the current cycle
    let offset_x = x - x_prime;

    // We'll use the modulo version of the equation for a triangle wave
    // https://en.wikipedia.org/wiki/Triangle_wave
    // But we'll modify it so that the cycle number is used to multiply the amplitude and period,
    // making the triangle wave get taller and broader each cyle. Define some params used in the calc:
    let s = offset_x - (c / 4.0) * (2.0 * phi + p);
    let p_star = c * p;

    // Here y_1 = g(x), where g is the triangle wave before it's truncated
    let y_1 = 6.0 / p * (modulo(s, p_star) - c * p / 2.0).abs() - 1.5 * (c);

    // We now truncate the wave so that it never has an amplitude greater than the cycle number
    match y_1.abs() > c {
        true => (y_1.signum() * c) as i32,
        false => y_1 as i32,
    }
}

/// In Rust, a % b finds the remainder of a / b. This function finds the actual modulo (not the remainder) of a and b.
fn modulo<T: std::ops::Rem<Output = T> + std::ops::Add<Output = T> + Copy>(a: T, b: T) -> T {
    ((a % b) + b) % b
}

#[cfg(test)]
mod tests {
    use crate::convert::{cube_to_spiral, spiral_to_cube, Cube};
    #[test]
    fn convert_spiral_to_cube() {
        // Test a few input values in spiral coordinates
        let spiral_vals: Vec<usize> = vec![0, 1, 4, 7, 8, 45];

        // Try find their cube coords
        let result = spiral_vals
            .into_iter()
            .map(spiral_to_cube)
            .collect::<Vec<Cube>>();

        // This is the result we expect to get
        let expected = [(0, 0, 0), (0, -1, 1), (0, 1, -1), (0, -2, 2), (1,-2,1), (4, 0, -4)]
            .into_iter()
            .map(|(q, r, s)| Cube::new(q, r, s))
            .collect::<Vec<Cube>>();

        assert_eq!(expected, result);
    }

    #[test]
    fn convert_cube_to_spiral() {
        // Test a few input values in cube coordinates
        let cube = [(0, 0, 0), (0, -1, 1), (0, 1, -1), (0, -2, 2), (1,-2,1), (4, 0, -4)]
            .into_iter()
            .map(|(q, r, s)| Cube::new(q, r, s));

        // Try find their spiral coords
        let result = cube
            .into_iter()
            .map(|c| cube_to_spiral(c).unwrap())
            .collect::<Vec<usize>>();

        assert_eq!(vec![0, 1, 4, 7, 8, 45], result);
    }

    #[test]
    fn convert_invalid_qrs() {
        // An invalid set of cube coords
        assert_eq!(Err("q + r + s != 0"), cube_to_spiral(Cube::new(-1, -1, 0)),)
    }
}

