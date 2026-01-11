//! Structure of Arrays (SoA) data layout for GPU-friendly memory access patterns.
//!
//! This module provides `Bodies`, a container that stores particle data in separate
//! arrays for each attribute. This layout enables:
//! - Coalesced GPU memory access when processing single attributes
//! - Direct upload of arrays to GPU storage buffers
//! - Better cache utilization for CPU SIMD operations

use super::Body;

/// Structure of Arrays container for body data.
///
/// All arrays are guaranteed to have the same length. This layout is optimal
/// for GPU compute shaders where each thread processes a single attribute
/// across many particles.
#[derive(Clone, Default, Debug)]
pub struct Bodies {
    /// Number of bodies (convenience field, equals positions.len())
    count: usize,
    /// Position vectors [x, y] for each body
    pub positions: Vec<[f32; 2]>,
    /// Velocity vectors [x, y] for each body
    pub velocities: Vec<[f32; 2]>,
    /// Mass of each body
    pub masses: Vec<f32>,
    /// Radius of each body
    pub radii: Vec<f32>,
    /// RGBA color of each body
    pub colors: Vec<[u8; 4]>,
    /// Unique identifier for each body
    pub ids: Vec<u32>,
}

impl Bodies {
    /// Creates a new empty Bodies container.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new Bodies container with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            count: 0,
            positions: Vec::with_capacity(capacity),
            velocities: Vec::with_capacity(capacity),
            masses: Vec::with_capacity(capacity),
            radii: Vec::with_capacity(capacity),
            colors: Vec::with_capacity(capacity),
            ids: Vec::with_capacity(capacity),
        }
    }

    /// Adds a body to the container.
    pub fn push(&mut self, body: &Body) {
        self.positions.push(body.position);
        self.velocities.push(body.velocity);
        self.masses.push(body.mass);
        self.radii.push(body.radius);
        self.colors.push(body.color);
        self.ids.push(body.id);
        self.count += 1;
    }

    /// Adds a body from individual components.
    pub fn push_components(
        &mut self,
        id: u32,
        position: [f32; 2],
        velocity: [f32; 2],
        mass: f32,
        radius: f32,
        color: [u8; 4],
    ) {
        self.positions.push(position);
        self.velocities.push(velocity);
        self.masses.push(mass);
        self.radii.push(radius);
        self.colors.push(color);
        self.ids.push(id);
        self.count += 1;
    }

    /// Clears all bodies from the container.
    pub fn clear(&mut self) {
        self.positions.clear();
        self.velocities.clear();
        self.masses.clear();
        self.radii.clear();
        self.colors.clear();
        self.ids.clear();
        self.count = 0;
    }

    /// Returns the number of bodies.
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if the container is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns a Body struct for the given index.
    /// Returns None if index is out of bounds.
    pub fn get(&self, index: usize) -> Option<Body> {
        if index >= self.count {
            return None;
        }
        Some(Body {
            id: self.ids[index],
            position: self.positions[index],
            velocity: self.velocities[index],
            mass: self.masses[index],
            color: self.colors[index],
            radius: self.radii[index],
        })
    }

    /// Sets the position at the given index.
    #[inline]
    pub fn set_position(&mut self, index: usize, pos: [f32; 2]) {
        self.positions[index] = pos;
    }

    /// Sets the velocity at the given index.
    #[inline]
    pub fn set_velocity(&mut self, index: usize, vel: [f32; 2]) {
        self.velocities[index] = vel;
    }

    /// Removes a body by index using swap_remove (O(1) but changes order).
    /// Returns the removed body, or None if index is out of bounds.
    pub fn swap_remove(&mut self, index: usize) -> Option<Body> {
        if index >= self.count {
            return None;
        }
        let body = self.get(index);
        self.positions.swap_remove(index);
        self.velocities.swap_remove(index);
        self.masses.swap_remove(index);
        self.radii.swap_remove(index);
        self.colors.swap_remove(index);
        self.ids.swap_remove(index);
        self.count -= 1;
        body
    }

    /// Finds the index of a body by its ID.
    pub fn find_by_id(&self, id: u32) -> Option<usize> {
        self.ids.iter().position(|&i| i == id)
    }

    /// Removes a body by its ID.
    /// Returns true if the body was found and removed.
    pub fn remove_by_id(&mut self, id: u32) -> bool {
        if let Some(index) = self.find_by_id(id) {
            self.swap_remove(index);
            true
        } else {
            false
        }
    }

    /// Reserves capacity for at least `additional` more bodies.
    pub fn reserve(&mut self, additional: usize) {
        self.positions.reserve(additional);
        self.velocities.reserve(additional);
        self.masses.reserve(additional);
        self.radii.reserve(additional);
        self.colors.reserve(additional);
        self.ids.reserve(additional);
    }
}

/// Iterator over bodies, yielding Body structs.
pub struct BodiesIter<'a> {
    bodies: &'a Bodies,
    index: usize,
}

impl<'a> Iterator for BodiesIter<'a> {
    type Item = Body;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.bodies.count {
            let body = self.bodies.get(self.index);
            self.index += 1;
            body
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.bodies.count - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for BodiesIter<'a> {}

impl<'a> IntoIterator for &'a Bodies {
    type Item = Body;
    type IntoIter = BodiesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BodiesIter {
            bodies: self,
            index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_get() {
        let mut bodies = Bodies::new();
        let body = Body::new(1, [10.0, 20.0], [1.0, 2.0], 100.0, [255, 0, 0, 255], 5.0);
        bodies.push(&body);

        assert_eq!(bodies.len(), 1);
        let retrieved = bodies.get(0).unwrap();
        assert_eq!(retrieved.id, 1);
        assert_eq!(retrieved.position, [10.0, 20.0]);
        assert_eq!(retrieved.velocity, [1.0, 2.0]);
        assert_eq!(retrieved.mass, 100.0);
        assert_eq!(retrieved.radius, 5.0);
    }

    #[test]
    fn test_clear() {
        let mut bodies = Bodies::with_capacity(10);
        bodies.push(&Body::default());
        bodies.push(&Body::default());
        assert_eq!(bodies.len(), 2);

        bodies.clear();
        assert_eq!(bodies.len(), 0);
        assert!(bodies.is_empty());
    }

    #[test]
    fn test_swap_remove() {
        let mut bodies = Bodies::new();
        bodies.push(&Body::new(0, [0.0, 0.0], [0.0, 0.0], 1.0, [0; 4], 1.0));
        bodies.push(&Body::new(1, [1.0, 1.0], [0.0, 0.0], 2.0, [0; 4], 1.0));
        bodies.push(&Body::new(2, [2.0, 2.0], [0.0, 0.0], 3.0, [0; 4], 1.0));

        let removed = bodies.swap_remove(0).unwrap();
        assert_eq!(removed.id, 0);
        assert_eq!(bodies.len(), 2);
        // The last element should have been swapped into position 0
        assert_eq!(bodies.get(0).unwrap().id, 2);
    }

    #[test]
    fn test_iterator() {
        let mut bodies = Bodies::new();
        for i in 0..3 {
            bodies.push(&Body::new(i, [i as f32, 0.0], [0.0, 0.0], 1.0, [0; 4], 1.0));
        }

        let collected: Vec<Body> = bodies.into_iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0].id, 0);
        assert_eq!(collected[1].id, 1);
        assert_eq!(collected[2].id, 2);
    }
}
