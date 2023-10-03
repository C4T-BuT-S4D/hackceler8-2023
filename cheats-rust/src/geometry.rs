use std::{
    f64::consts::PI,
    ops::{Div, Mul, Neg, Sub},
};

use pyo3::{pyclass, pymethods};

#[derive(Copy, Clone, Debug)]
#[pyclass]
pub struct Pointf {
    pub x: f64,
    pub y: f64,
}

#[pymethods]
impl Pointf {
    #[new]
    pub fn new(x: f64, y: f64) -> Self {
        Pointf { x, y }
    }
}

impl Pointf {
    fn unit_vector(&self) -> Pointf {
        *self / self.len()
    }

    fn angle(&self, other: &Pointf) -> f64 {
        let dot = self.unit_vector() * other.unit_vector();
        180.0 - f64::acos(f64::min(f64::max(dot, -1.0), 1.0)) * 180.0 / PI
    }

    pub fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn ortho(&self) -> Pointf {
        Pointf {
            x: -self.y,
            y: self.x,
        }
    }
}

impl Sub for Pointf {
    type Output = Pointf;

    fn sub(self, other: Pointf) -> Pointf {
        Pointf {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Neg for Pointf {
    type Output = Pointf;

    fn neg(self) -> Pointf {
        Pointf {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Mul<f64> for Pointf {
    type Output = Pointf;

    fn mul(self, rhs: f64) -> Pointf {
        Pointf {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Pointf> for Pointf {
    type Output = f64;

    fn mul(self, rhs: Pointf) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl Div<f64> for Pointf {
    type Output = Pointf;

    fn div(self, rhs: f64) -> Pointf {
        Pointf {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct Polygon {
    pub outline: Vec<Pointf>,

    pub vectors: Vec<Pointf>,
    angles: Vec<f64>,

    pub center: Pointf,
    pub highest_point: f64,
    pub lowest_point: f64,
    pub rightmost_point: f64,
    pub leftmost_point: f64,
}

#[pymethods]
impl Polygon {
    #[new]
    pub fn new(outline: Vec<Pointf>) -> Self {
        let mut polygon = Polygon {
            outline,
            vectors: vec![],
            angles: vec![],

            center: Pointf { x: 0.0, y: 0.0 },
            highest_point: 0.0,
            lowest_point: 0.0,
            rightmost_point: 0.0,
            leftmost_point: 0.0,
        };
        polygon.init();
        polygon
    }
}

impl Polygon {
    fn init(&mut self) {
        self.get_vectors();
        self.get_angles();
        self.is_convex();
        self.cache_points();
    }

    fn get_vectors(&mut self) {
        self.vectors.clear();
        for i in 0..self.outline.len() {
            self.vectors
                .push(self.outline[(i + 1) % self.outline.len()] - self.outline[i]);
        }
    }

    fn get_angles(&mut self) {
        self.angles.clear();
        for i in 0..self.vectors.len() {
            self.angles
                .push(self.vectors[i].angle(&self.vectors[(i + 1) % self.vectors.len()]));
        }
    }

    fn is_convex(&self) {
        for &angle in &self.angles {
            assert!(angle < 180.0);
        }
    }

    fn cache_points(&mut self) {
        self.lowest_point = f64::INFINITY;
        self.highest_point = f64::NEG_INFINITY;
        self.leftmost_point = f64::INFINITY;
        self.rightmost_point = f64::NEG_INFINITY;

        let mut center_x = 0.0;
        let mut center_y = 0.0;

        for point in &self.outline {
            self.lowest_point = f64::min(self.lowest_point, point.y);
            self.highest_point = f64::max(self.highest_point, point.y);
            self.leftmost_point = f64::min(self.leftmost_point, point.x);
            self.rightmost_point = f64::max(self.rightmost_point, point.x);

            center_x += point.x;
            center_y += point.y;
        }
        self.center = Pointf {
            x: center_x / self.outline.len() as f64,
            y: center_y / self.outline.len() as f64,
        };
    }
}

impl PartialEq for Polygon {
    fn eq(&self, other: &Self) -> bool {
        self.outline.len() == other.outline.len()
            && self
                .outline
                .iter()
                .zip(other.outline.iter())
                .all(|(a, b)| a.x == b.x && a.y == b.y)
    }
}
