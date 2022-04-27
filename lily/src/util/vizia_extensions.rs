//! Trait extensions for making working with Vizia even nicer

use glam::Vec2;
use vizia::*;

pub trait BoundingBoxExt {
    fn map_ui_point(&self, point: Vec2, centered: bool) -> Vec2;
    fn map_data_point(&self, point: Vec2, centered: bool) -> Vec2;
    fn map_ui_point_unbounded(&self, point: Vec2, centered: bool) -> Vec2;
    fn contains_point(&self, point: Vec2) -> bool;
}

impl BoundingBoxExt for BoundingBox {
    /// Maps a UI point to a normalized `Vec2` from `(0,0)..=(1,1)`, or
    /// `(-1,-1)..=(1,1)` if centered.
    fn map_ui_point(&self, point: Vec2, centered: bool) -> Vec2 {
        // clamp point to rect bounds
        let point = point.clamp(self.top_left().into(), self.bottom_right().into());
        // local space for the point
        let point = point - Vec2::from(self.top_left());
        let result = Vec2::new(point.x / self.width(), point.y / self.height());
        match centered {
            true => (result * 2f32) - 1f32,
            false => result,
        }
    }
    /// Maps a normalized data point from `(0,0)..=(1,1)` (or `(-1,-1)..=(1,1)`
    /// if centered) to absolute UI coordinates
    fn map_data_point(&self, point: Vec2, centered: bool) -> Vec2 {
        let point = match centered {
            true => (point + Vec2::ONE) / 2f32,
            false => point,
        };
        let x = (point.x * self.width()) + self.left();
        let y = (point.y * self.height()) + self.top();
        Vec2::new(x, y)
    }

    /// Gets the width and height ratio of an arbitrary point (that may exist outside of the rect, in which case a ratio over 1 would be provided).
    fn map_ui_point_unbounded(&self, point: Vec2, centered: bool) -> Vec2 {
        // convert point to local space
        let local_point = point - Vec2::from(self.top_left());
        let result = Vec2::new(local_point.x / self.width(), local_point.y / self.height());
        match centered {
            true => (result * 2f32) - 1f32,
            false => result,
        }
    }

    fn contains_point(&self, point: Vec2) -> bool {
        point.x <= self.right()
            && point.x >= self.left()
            && point.y <= self.bottom()
            && point.y >= self.top()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    fn rect() -> BoundingBox {
        BoundingBox {
            x: 100f32,
            y: 100f32,
            w: 100f32,
            h: 100f32,
        }
    }

    #[test]
    fn get_map_unbounded() {
        let rect = rect();
        let cursor = Vec2::new(300f32, 300f32);
        assert_eq!(
            rect.map_ui_point_unbounded(cursor, false),
            Vec2::splat(2f32)
        );
    }

    #[test]
    fn get_map_unbounded_center() {
        let rect = rect();
        let cursor = Vec2::new(250f32, 250f32);
        assert_eq!(rect.map_ui_point_unbounded(cursor, true), Vec2::splat(2f32));
    }

    #[test]
    fn get_mapped_ui_point() {
        let rect = rect();
        let cursor = Vec2::new(150f32, 150f32);
        assert_eq!(rect.map_ui_point(cursor, false), Vec2::splat(0.5));
    }

    #[test]
    fn get_mapped_data_point() {
        let rect = rect();
        let data = Vec2::splat(0.5);
        assert_eq!(rect.map_data_point(data, false), Vec2::new(150f32, 150f32));
    }

    #[test]
    fn get_mapped_ui_point_center() {
        let rect = rect();
        let cursor = Vec2::new(150f32, 150f32);
        assert_eq!(rect.map_ui_point(cursor, true), Vec2::splat(0.0));
    }

    #[test]
    fn get_mapped_data_point_center() {
        let rect = rect();
        let data = Vec2::splat(0.0);
        assert_eq!(rect.map_data_point(data, true), Vec2::new(150f32, 150f32));
    }
}
