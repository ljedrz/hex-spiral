# hex-spiral
[![crates.io](https://img.shields.io/crates/v/hex-spiral)](https://crates.io/crates/hex-spiral)
[![docs.rs](https://docs.rs/hex-spiral/badge.svg)](https://docs.rs/hex-spiral)
[![dependencies](https://deps.rs/repo/github/ljedrz/hex-spiral/status.svg)](https://deps.rs/repo/github/ljedrz/hex-spiral)

**hex-spiral** is library for working with 2D hexagonal maps using single-coordinate positions.

## Overview

While most hex-grid-based 2D games use multiple coordinates, **hex-spiral** uses a single-coordinate spiral,
where the central hex has the position `0`, and further hexes are placed within theoretical hexagonal rings
that surround it.

<p align="center"><img src="https://i.imgur.com/WzFffuV.png"></p>
