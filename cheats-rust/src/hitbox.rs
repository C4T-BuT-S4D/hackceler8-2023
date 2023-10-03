use core::hash::Hash;
use std::hash::Hasher;

use pyo3::{pyclass, pymethods};

use crate::geometry::{Pointf, Polygon};

#[pyclass]
#[derive(Clone, Debug)]
pub struct Hitbox {
    pub polygon: Polygon,
}

impl PartialEq for Hitbox {
    fn eq(&self, other: &Self) -> bool {
        self.polygon == other.polygon
    }
}

impl Hash for Hitbox {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for point in &self.polygon.outline {
            state.write(&point.x.to_le_bytes());
            state.write(&point.y.to_le_bytes());
        }
    }
}

#[pymethods]
impl Hitbox {
    #[new]
    pub fn new(outline: Vec<Pointf>) -> Self {
        Hitbox {
            polygon: Polygon::new(outline),
        }
    }
}

impl Hitbox {
    pub fn collides(&self, other: &Hitbox) -> Option<Pointf> {
        let mut min = f64::INFINITY;
        let mut mpv = Pointf { x: 0.0, y: 0.0 };
        let self_axes = self.get_axes();
        let other_axes = other.get_axes();

        for axis in self_axes.iter().chain(other_axes.iter()) {
            let Some(pv) = self.is_separating_axis(axis, &other.polygon.outline) else {
                return None;
            };
            let norm = pv * pv;
            if norm < min {
                min = norm;
                mpv = pv;
            }
        }

        let d = self.centers_displacement(other);
        if d * mpv > 0.0 {
            mpv = -mpv;
        }

        Some(mpv)
    }

    pub fn collides_as_rect(&self, other: &Hitbox) -> bool {
        debug_assert!(self.polygon.outline.len() == 4);
        debug_assert!(other.polygon.outline.len() == 4);

        let (sx1, sx2) = (self.polygon.leftmost_point, self.polygon.rightmost_point);
        let (sy1, sy2) = (self.polygon.lowest_point, self.polygon.highest_point);

        let (ox1, ox2) = (other.polygon.leftmost_point, other.polygon.rightmost_point);
        let (oy1, oy2) = (other.polygon.lowest_point, other.polygon.highest_point);

        sx1 <= ox2 && sx2 >= ox1 && sy1 <= oy2 && sy2 >= oy1
    }

    fn centers_displacement(&self, other: &Hitbox) -> Pointf {
        other.polygon.center - self.polygon.center
    }

    fn is_separating_axis(&self, axis: &Pointf, other: &[Pointf]) -> Option<Pointf> {
        let mut min1 = f64::INFINITY;
        let mut max1 = f64::NEG_INFINITY;
        let mut min2 = f64::INFINITY;
        let mut max2 = f64::NEG_INFINITY;

        for point in &self.polygon.outline {
            let proj = *point * *axis;
            min1 = f64::min(min1, proj);
            max1 = f64::max(max1, proj);
        }

        for point in other {
            let proj = *point * *axis;
            min2 = f64::min(min2, proj);
            max2 = f64::max(max2, proj);
        }

        if max1 >= min2 && max2 >= min1 {
            let d = f64::min(max2 - min1, max1 - min2);
            let d_over_o_squared = d / (*axis * *axis) + 1e-10;
            Some(*axis * d_over_o_squared)
        } else {
            None
        }
    }

    fn get_axes(&self) -> Vec<Pointf> {
        self.polygon.vectors.iter().map(|v| v.ortho()).collect()
    }
}
