use crate::core::color::Color;
use crate::core::math::Vec2;

/// A rich Circle model representing a physical or visual circle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
    pub color: Color,
}

impl Circle {
    pub fn new(center: Vec2, radius: f32, color: Color) -> Self {
        Self {
            center,
            radius,
            color,
        }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        self.center.distance(point) <= self.radius
    }

    pub fn intersects(&self, other: &Circle) -> bool {
        self.center.distance(other.center) <= (self.radius + other.radius)
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(
            self.center - Vec2::new(self.radius, self.radius),
            Vec2::new(self.radius * 2.0, self.radius * 2.0),
        )
    }
}

/// A rich rectangle model used for bounds, UI, and spatial indexing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub min: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(min: Vec2, size: Vec2) -> Self {
        Self { min, size }
    }

    pub fn max(&self) -> Vec2 {
        self.min + self.size
    }

    pub fn center(&self) -> Vec2 {
        self.min + self.size * 0.5
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.min.x + self.size.x
            && point.y >= self.min.y
            && point.y <= self.min.y + self.size.y
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        let max_a = self.max();

        !(other.min.x > max_a.x
            || other.max().x < self.min.x
            || other.min.y > max_a.y
            || other.max().y < self.min.y)
    }
}
