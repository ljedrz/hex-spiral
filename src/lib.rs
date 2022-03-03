//! While most hex-grid-based 2D games use multiple coordinates, **hex-spiral** uses a single-coordinate spiral,
//! where the central hex has the position `0`, and further hexes are placed within theoretical hexagonal rings
//! that surround it. The hexes are flat-topped and every ring is indexed starting with the hex on the top edge
//! of the previous ring and with further positions growing clockwise.

use itertools::Itertools;

pub type Pos = usize;
pub type RingIdx = usize;

crepe::crepe! {
    @input
    struct Position(Pos);

    struct Neighbors(Pos, Pos);

    @output
    struct Grouped(Pos, Pos);

    Neighbors(p1, p2) <- Position(p1), Position(p2), (are_neighbors(p1, p2));
    Grouped(p1, p2) <- Neighbors(p1, p2);
    Grouped(p1, p3) <- Neighbors(p1, p2), Grouped(p2, p3), (p1 != p3);
}

/// The starting position of hexes within the ring with the given index.
pub fn ring_offset(ring: RingIdx) -> Pos {
    if ring == 0 {
        0
    } else {
        3 * (ring - 1) * ring + 1
    }
}

/// The index of the ring for the given position.
pub fn ring(pos: Pos) -> RingIdx {
    (0..)
        .map(ring_offset)
        .position(|offset| pos < offset)
        .unwrap()
        - 1
}

/// Returns `true` if the given position is at one of the tips of a ring.
pub fn is_at_ring_tip(pos: Pos) -> bool {
    let ring = ring(pos);
    let ring_offset = ring_offset(ring);

    (0..6)
        .map(|n| ring_offset + n * ring)
        .any(|ring_tip| pos == ring_tip)
}

/// Returns the index of the edge of the ring the given position belongs to.
pub fn ring_edge_index(pos: Pos) -> usize {
    let ring = ring(pos);
    (pos - ring_offset(ring)) / ring
}

/// An iterator returning subsequent neighboring positions in the given direction.
/// The top direction is `0`, and it increases up to `5` clockwise.
pub struct DirectionalNeighborIter {
    curr_pos: Pos,
    dir: usize,
}

impl DirectionalNeighborIter {
    /// Create a new `DirectionalNeighborIter` starting at the given position
    /// and progressing in the chosen direction.
    pub fn new(pos: Pos, dir: usize) -> Self {
        assert!(dir <= 5);
        Self { curr_pos: pos, dir }
    }

    /// Returns the position the `DirectionalNeighborIter` is currently at.
    pub fn curr_pos(&self) -> Pos {
        self.curr_pos
    }
}

impl Iterator for DirectionalNeighborIter {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        let next = neighboring_positions(self.curr_pos)[self.dir];
        self.curr_pos = next;
        Some(next)
    }
}

fn ring_neighboring_positions(pos: Pos) -> [Pos; 2] {
    assert!(pos != 0);

    let ring = ring(pos);

    if pos == ring_offset(ring) {
        [ring_offset(ring + 1) - 1, pos + 1]
    } else if pos == ring_offset(ring + 1) - 1 {
        [pos - 1, ring_offset(ring)]
    } else {
        [pos - 1, pos + 1]
    }
}

/// Returns the 6 neighbors of the given position, always in the same clockwise order.
pub fn neighboring_positions(pos: Pos) -> [Pos; 6] {
    let ring = ring(pos);

    match ring {
        0 => [1, 2, 3, 4, 5, 6],
        _ => {
            let edge_index = ring_edge_index(pos);

            let mut poss = if is_at_ring_tip(pos) {
                // 1 neighbor from the lower ring, 3 from the upper ring, 2 from the same ring
                let lower_neighbor = ring_offset(ring - 1) + (ring - 1) * edge_index;
                let ring_neighbors = ring_neighboring_positions(pos);
                let upper_tip_neighbor = if pos == ring_offset(ring + 1) - 1 {
                    ring_offset(ring + 2) - 2
                } else {
                    ring_offset(ring + 1) + (ring + 1) * edge_index
                };
                let upper_tip_neighbors = ring_neighboring_positions(upper_tip_neighbor);

                [
                    upper_tip_neighbor,
                    upper_tip_neighbors[1],
                    ring_neighbors[1],
                    lower_neighbor,
                    ring_neighbors[0],
                    upper_tip_neighbors[0],
                ]
            } else {
                // 2 neighbors from the lower ring, the upper ring, and the same ring
                let ring_pos = pos - ring_offset(ring);
                let tip_offset = ring_pos - (edge_index * ring);
                let (lower_neighbor1, lower_neighbor2) = if pos == ring_offset(ring + 1) - 1 {
                    (ring_offset(ring) - 1, ring_offset(ring - 1))
                } else {
                    let lower_neighbor1 =
                        ring_offset(ring - 1) + (edge_index * (ring - 1)) + tip_offset - 1;
                    (lower_neighbor1, lower_neighbor1 + 1)
                };
                let ring_neighbors = ring_neighboring_positions(pos);
                let upper_neighbor1 =
                    ring_offset(ring + 1) + (edge_index * (ring + 1)) + tip_offset;
                let upper_neighbor2 = upper_neighbor1 + 1;

                [
                    upper_neighbor1,
                    upper_neighbor2,
                    ring_neighbors[1],
                    lower_neighbor2,
                    lower_neighbor1,
                    ring_neighbors[0],
                ]
            };

            poss.rotate_right(edge_index);

            poss
        }
    }
}

/// Returns `true` if the given 2 positions are neighbors.
pub fn are_neighbors(pos1: Pos, pos2: Pos) -> bool {
    neighboring_positions(pos1).contains(&pos2)
}

/// Returns `true` if the given list of positions consists of subsequent neighbors.
pub fn is_path_consistent(poss: &[Pos]) -> bool {
    assert!(poss.len() >= 2);

    poss.windows(2).all(|pair| {
        if let &[p1, p2] = pair {
            are_neighbors(p1, p2)
        } else {
            unreachable!();
        }
    })
}

pub fn are_grouped(poss: &[Pos]) -> bool {
    let mut rt = Crepe::new();
    rt.extend(poss.iter().copied().map(Position));
    let (groups,) = rt.run();

    poss.iter()
        .permutations(2)
        .all(|pair| groups.contains(&Grouped(*pair[0], *pair[1])))
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn ring_offsets() {
        let offsets = [0, 1, 7, 19, 37, 61, 91];

        for (i, offset) in offsets.into_iter().enumerate() {
            assert_eq!(ring_offset(i), offset);
        }
    }

    #[test]
    fn position_rings() {
        let offsets = [0, 1, 7, 19, 37, 61, 91];

        for (i, window) in offsets.windows(2).enumerate() {
            if let [beg, end] = window {
                for pos in *beg..*end {
                    assert_eq!(ring(pos), i);
                }
            }
        }
    }

    #[test]
    fn ring_tips() {
        for pos in 1..=6 {
            assert!(is_at_ring_tip(pos), "{}", pos);
        }

        for pos in [7, 9, 11, 13, 15, 17] {
            assert!(is_at_ring_tip(pos), "{}", pos);
        }

        for pos in [61, 66, 71, 76, 81, 86] {
            assert!(is_at_ring_tip(pos), "{}", pos);
        }
    }

    #[test]
    fn ring_edges() {
        for pos in [8, 10, 12, 14, 16, 18] {
            assert!(!is_at_ring_tip(pos), "{}", pos);
        }

        for pos in (62..66)
            .chain(67..71)
            .chain(72..76)
            .chain(77..81)
            .chain(82..86)
        {
            assert!(!is_at_ring_tip(pos), "{}", pos);
        }
    }

    #[test]
    fn edge_indices_non_tips() {
        for pos in [8, 21].into_iter().chain(38..=40).chain(62..=65) {
            assert_eq!(ring_edge_index(pos), 0);
        }
        for pos in [10, 23, 24].into_iter().chain(42..=44).chain(67..=70) {
            assert_eq!(ring_edge_index(pos), 1);
        }
        for pos in [12, 26, 27].into_iter().chain(46..=48).chain(72..=75) {
            assert_eq!(ring_edge_index(pos), 2);
        }
        for pos in [14, 29, 30].into_iter().chain(50..=52).chain(77..=80) {
            assert_eq!(ring_edge_index(pos), 3);
        }
        for pos in [16, 32, 33].into_iter().chain(54..=56).chain(82..=85) {
            assert_eq!(ring_edge_index(pos), 4);
        }
        for pos in [18, 35, 36].into_iter().chain(58..=60).chain(87..=90) {
            assert_eq!(ring_edge_index(pos), 5);
        }
    }

    #[test]
    fn ring_neighbors() {
        assert_eq!(ring_neighboring_positions(1), [6, 2]);
        assert_eq!(ring_neighboring_positions(2), [1, 3]);
        assert_eq!(ring_neighboring_positions(3), [2, 4]);
        assert_eq!(ring_neighboring_positions(4), [3, 5]);
        assert_eq!(ring_neighboring_positions(5), [4, 6]);
        assert_eq!(ring_neighboring_positions(6), [5, 1]);
        assert_eq!(ring_neighboring_positions(18), [17, 7]);
        assert_eq!(ring_neighboring_positions(58), [57, 59]);
    }

    #[test]
    fn ring_tip_neighbors() {
        assert_eq!(neighboring_positions(1), [7, 8, 2, 0, 6, 18]);
        assert_eq!(neighboring_positions(2), [8, 9, 10, 3, 0, 1]);
        assert_eq!(neighboring_positions(3), [2, 10, 11, 12, 4, 0]);
        assert_eq!(neighboring_positions(4), [0, 3, 12, 13, 14, 5]);
        assert_eq!(neighboring_positions(5), [6, 0, 4, 14, 15, 16]);
        assert_eq!(neighboring_positions(6), [18, 1, 0, 5, 16, 17]);
        assert_eq!(neighboring_positions(7), [19, 20, 8, 1, 18, 36]);
        assert_eq!(neighboring_positions(9), [21, 22, 23, 10, 2, 8]);
        assert_eq!(neighboring_positions(11), [10, 24, 25, 26, 12, 3]);
        assert_eq!(neighboring_positions(13), [4, 12, 27, 28, 29, 14]);
        assert_eq!(neighboring_positions(15), [16, 5, 14, 30, 31, 32]);
        assert_eq!(neighboring_positions(17), [35, 18, 6, 16, 33, 34]);
        assert_eq!(neighboring_positions(28), [13, 27, 48, 49, 50, 29]);
        assert_eq!(neighboring_positions(53), [54, 31, 52, 80, 81, 82]);
        assert_eq!(neighboring_positions(57), [87, 58, 34, 56, 85, 86]);
    }

    #[test]
    fn ring_edge_neighbors() {
        assert_eq!(neighboring_positions(8), [20, 21, 9, 2, 1, 7]);
        assert_eq!(neighboring_positions(10), [9, 23, 24, 11, 3, 2]);
        assert_eq!(neighboring_positions(12), [3, 11, 26, 27, 13, 4]);
        assert_eq!(neighboring_positions(14), [5, 4, 13, 29, 30, 15]);
        assert_eq!(neighboring_positions(16), [17, 6, 5, 15, 32, 33]);
        assert_eq!(neighboring_positions(18), [36, 7, 1, 6, 17, 35]);
        assert_eq!(neighboring_positions(38), [62, 63, 39, 20, 19, 37]);
        assert_eq!(neighboring_positions(40), [64, 65, 41, 22, 21, 39]);
        assert_eq!(neighboring_positions(42), [41, 67, 68, 43, 23, 22]);
        assert_eq!(neighboring_positions(44), [43, 69, 70, 45, 25, 24]);
        assert_eq!(neighboring_positions(46), [25, 45, 72, 73, 47, 26]);
        assert_eq!(neighboring_positions(48), [27, 47, 74, 75, 49, 28]);
        assert_eq!(neighboring_positions(50), [29, 28, 49, 77, 78, 51]);
        assert_eq!(neighboring_positions(52), [31, 30, 51, 79, 80, 53]);
        assert_eq!(neighboring_positions(54), [55, 32, 31, 53, 82, 83]);
        assert_eq!(neighboring_positions(56), [57, 34, 33, 55, 84, 85]);
        assert_eq!(neighboring_positions(58), [88, 59, 35, 34, 57, 87]);
        assert_eq!(neighboring_positions(60), [90, 37, 19, 36, 59, 89]);
    }

    #[test]
    fn groups() {
        assert!([2, 8, 9]
            .into_iter()
            .permutations(3)
            .all(|perm| are_grouped(&perm)));
        assert!([1, 0, 4]
            .into_iter()
            .permutations(3)
            .all(|perm| are_grouped(&perm)));
        assert!([71, 45, 25, 24, 23, 22, 41, 66]
            .into_iter()
            .permutations(8)
            .all(|perm| are_grouped(&perm)));
        assert!([0, 1, 2, 3, 4, 5, 6]
            .into_iter()
            .permutations(7)
            .all(|perm| are_grouped(&perm)));
        assert!([5, 17, 18]
            .into_iter()
            .permutations(3)
            .all(|perm| !are_grouped(&perm)));
        assert!([2, 3, 5, 6]
            .into_iter()
            .permutations(4)
            .all(|perm| !are_grouped(&perm)));
        assert!([1, 4]
            .into_iter()
            .permutations(2)
            .all(|perm| !are_grouped(&perm)));

        assert!(are_grouped(&[11, 10, 2, 1, 6, 5, 15, 30, 29, 28, 27, 26]));
        assert!(!are_grouped(&[
            1, 2, 3, 4, 5, 16, 17, 35, 36, 20, 21, 22, 23, 24, 25, 26
        ]));
        assert!(are_grouped(&[
            1, 2, 3, 4, 5, 16, 17, 35, 36, 19, 20, 21, 22, 23, 24, 25, 26
        ]));
    }

    #[test]
    fn directional_neighbor_iter() {
        use DirectionalNeighborIter as DNI;

        assert_eq!(
            DNI::new(75, 0).take(9).collect::<Vec<_>>(),
            vec![48, 27, 12, 3, 2, 8, 20, 38, 62]
        );
        assert_eq!(
            DNI::new(76, 0).take(10).collect::<Vec<_>>(),
            vec![49, 28, 13, 4, 0, 1, 7, 19, 37, 61]
        );
        assert_eq!(
            DNI::new(77, 0).take(9).collect::<Vec<_>>(),
            vec![50, 29, 14, 5, 6, 18, 36, 60, 90]
        );

        assert_eq!(
            DNI::new(80, 1).take(9).collect::<Vec<_>>(),
            vec![52, 30, 14, 4, 3, 10, 23, 42, 67]
        );
        assert_eq!(
            DNI::new(81, 1).take(10).collect::<Vec<_>>(),
            vec![53, 31, 15, 5, 0, 2, 9, 22, 41, 66]
        );
        assert_eq!(
            DNI::new(82, 1).take(9).collect::<Vec<_>>(),
            vec![54, 32, 16, 6, 1, 8, 21, 40, 65]
        );

        assert_eq!(
            DNI::new(85, 2).take(9).collect::<Vec<_>>(),
            vec![56, 33, 16, 5, 4, 12, 26, 46, 72]
        );
        assert_eq!(
            DNI::new(86, 2).take(10).collect::<Vec<_>>(),
            vec![57, 34, 17, 6, 0, 3, 11, 25, 45, 71]
        );
        assert_eq!(
            DNI::new(87, 2).take(9).collect::<Vec<_>>(),
            vec![58, 35, 18, 1, 2, 10, 24, 44, 70]
        );
    }
}
