//! Map objects that can be placed in the editor.

use crate::core::color::Color;
use crate::core::math::Vec2;

/// Unique identifier for a placed map object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapObjectId(pub u64);

impl MapObjectId {
    /// Generate a new unique ID.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for MapObjectId {
    fn default() -> Self {
        Self::new()
    }
}

/// Definition of a placeable map object type.
#[derive(Debug, Clone)]
pub struct MapObject {
    /// Unique name/identifier for this object type.
    pub name: String,
    /// Display name shown in the editor UI.
    pub display_name: String,
    /// Category for organizing in the palette.
    pub category: String,
    /// Visual representation type.
    pub visual: ObjectVisual,
    /// Size in world units.
    pub size: Vec2,
    /// Pivot point (0,0 = bottom-left, 0.5,0.5 = center).
    pub pivot: Vec2,
    /// Whether this object can be rotated.
    pub rotatable: bool,
    /// Whether this object can be scaled.
    pub scalable: bool,
    /// Custom properties (for game-specific data).
    pub properties: Vec<ObjectProperty>,
    /// Optional collision shape.
    pub collision: Option<CollisionShape>,
    /// Z-order layer.
    pub default_layer: i32,
}

impl MapObject {
    /// Create a new map object with a texture visual.
    pub fn texture(name: impl Into<String>, texture_name: impl Into<String>, size: Vec2) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            name,
            category: "Default".to_string(),
            visual: ObjectVisual::Texture {
                texture_name: texture_name.into(),
                tint: Color::WHITE,
            },
            size,
            pivot: Vec2::new(0.5, 0.5),
            rotatable: true,
            scalable: true,
            properties: Vec::new(),
            collision: None,
            default_layer: 0,
        }
    }

    /// Create a new map object with an animated sprite sheet visual.
    pub fn sprite_sheet(
        name: impl Into<String>,
        texture_name: impl Into<String>,
        frame_count: u32,
        size: Vec2,
    ) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            name,
            category: "Default".to_string(),
            visual: ObjectVisual::SpriteSheet {
                texture_name: texture_name.into(),
                frame_count,
                current_frame: 0,
                fps: 10.0,
                tint: Color::WHITE,
            },
            size,
            pivot: Vec2::new(0.5, 0.5),
            rotatable: true,
            scalable: true,
            properties: Vec::new(),
            collision: None,
            default_layer: 0,
        }
    }

    /// Create a new map object with a tileset visual.
    pub fn tileset(
        name: impl Into<String>,
        texture_name: impl Into<String>,
        columns: u32,
        rows: u32,
        tile_size: Vec2,
    ) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            name,
            category: "Default".to_string(),
            visual: ObjectVisual::Tileset {
                texture_name: texture_name.into(),
                columns,
                rows,
                selected_tile: 0,
                ignored_tiles: Vec::new(),
                tint: Color::WHITE,
            },
            size: tile_size,
            pivot: Vec2::new(0.5, 0.5),
            rotatable: false,
            scalable: true,
            properties: Vec::new(),
            collision: None,
            default_layer: 0,
        }
    }

    /// Create a new map object with a 9-slice UI visual.
    /// 9-slice allows UI elements to stretch while keeping corners/edges intact.
    pub fn nine_slice(
        name: impl Into<String>,
        texture_name: impl Into<String>,
        size: Vec2,
        left: u32,
        right: u32,
        top: u32,
        bottom: u32,
    ) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            name,
            category: "UI".to_string(),
            visual: ObjectVisual::NineSlice {
                texture_name: texture_name.into(),
                left,
                right,
                top,
                bottom,
                tint: Color::WHITE,
            },
            size,
            pivot: Vec2::new(0.5, 0.5),
            rotatable: false,
            scalable: true,
            properties: Vec::new(),
            collision: None,
            default_layer: 0,
        }
    }

    /// Create a new map object with a circle visual.
    pub fn circle(name: impl Into<String>, radius: f32, color: Color) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            name,
            category: "Default".to_string(),
            visual: ObjectVisual::Circle { radius, color },
            size: Vec2::new(radius * 2.0, radius * 2.0),
            pivot: Vec2::new(0.5, 0.5),
            rotatable: false,
            scalable: true,
            properties: Vec::new(),
            collision: Some(CollisionShape::Circle { radius }),
            default_layer: 0,
        }
    }

    /// Create a new map object with a rectangle visual.
    pub fn rect(name: impl Into<String>, width: f32, height: f32, color: Color) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            name,
            category: "Default".to_string(),
            visual: ObjectVisual::Rect { color },
            size: Vec2::new(width, height),
            pivot: Vec2::new(0.5, 0.5),
            rotatable: true,
            scalable: true,
            properties: Vec::new(),
            collision: Some(CollisionShape::Rect {
                width,
                height,
            }),
            default_layer: 0,
        }
    }

    /// Set the display name.
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    /// Set the category.
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    /// Set the pivot point.
    pub fn with_pivot(mut self, pivot: Vec2) -> Self {
        self.pivot = pivot;
        self
    }

    /// Set whether the object is rotatable.
    pub fn with_rotatable(mut self, rotatable: bool) -> Self {
        self.rotatable = rotatable;
        self
    }

    /// Set whether the object is scalable.
    pub fn with_scalable(mut self, scalable: bool) -> Self {
        self.scalable = scalable;
        self
    }

    /// Add a custom property.
    pub fn with_property(mut self, property: ObjectProperty) -> Self {
        self.properties.push(property);
        self
    }

    /// Set the collision shape.
    pub fn with_collision(mut self, collision: CollisionShape) -> Self {
        self.collision = Some(collision);
        self
    }

    /// Set the default layer.
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.default_layer = layer;
        self
    }
}

/// Visual representation of a map object.
#[derive(Debug, Clone)]
pub enum ObjectVisual {
    /// Textured sprite.
    Texture {
        texture_name: String,
        tint: Color,
    },
    /// Animated sprite sheet.
    SpriteSheet {
        /// Base texture name (frames are named {texture_name}_0, {texture_name}_1, etc.)
        texture_name: String,
        /// Number of frames in the animation
        frame_count: u32,
        /// Current frame index
        current_frame: u32,
        /// Frames per second
        fps: f32,
        tint: Color,
    },
    /// Tileset - a grid of tiles for map building.
    Tileset {
        /// Base texture name (tiles are named {texture_name}_0, {texture_name}_1, etc.)
        texture_name: String,
        /// Number of columns in the tileset
        columns: u32,
        /// Number of rows in the tileset
        rows: u32,
        /// Currently selected tile index
        selected_tile: u32,
        /// Tile indices that are ignored/empty (not usable)
        ignored_tiles: Vec<u32>,
        tint: Color,
    },
    /// 9-slice UI element - corners stay fixed, edges and center stretch.
    NineSlice {
        /// Texture name
        texture_name: String,
        /// Left margin (pixels from left edge that don't stretch)
        left: u32,
        /// Right margin (pixels from right edge that don't stretch)
        right: u32,
        /// Top margin (pixels from top edge that don't stretch)
        top: u32,
        /// Bottom margin (pixels from bottom edge that don't stretch)
        bottom: u32,
        tint: Color,
    },
    /// Solid circle.
    Circle {
        radius: f32,
        color: Color,
    },
    /// Solid rectangle.
    Rect {
        color: Color,
    },
    /// Line (for paths, etc.).
    Line {
        color: Color,
        thickness: f32,
    },
}

/// Custom property for map objects.
#[derive(Debug, Clone)]
pub struct ObjectProperty {
    /// Property name.
    pub name: String,
    /// Property value.
    pub value: PropertyValue,
    /// Whether this property is editable in the UI.
    pub editable: bool,
}

impl ObjectProperty {
    /// Create a new string property.
    pub fn string(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: PropertyValue::String(value.into()),
            editable: true,
        }
    }

    /// Create a new integer property.
    pub fn int(name: impl Into<String>, value: i32) -> Self {
        Self {
            name: name.into(),
            value: PropertyValue::Int(value),
            editable: true,
        }
    }

    /// Create a new float property.
    pub fn float(name: impl Into<String>, value: f32) -> Self {
        Self {
            name: name.into(),
            value: PropertyValue::Float(value),
            editable: true,
        }
    }

    /// Create a new boolean property.
    pub fn bool(name: impl Into<String>, value: bool) -> Self {
        Self {
            name: name.into(),
            value: PropertyValue::Bool(value),
            editable: true,
        }
    }

    /// Create a new color property.
    pub fn color(name: impl Into<String>, value: Color) -> Self {
        Self {
            name: name.into(),
            value: PropertyValue::Color(value),
            editable: true,
        }
    }
}

/// Value types for custom properties.
#[derive(Debug, Clone)]
pub enum PropertyValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Color(Color),
    Vec2(Vec2),
}

/// Collision shape for map objects.
#[derive(Debug, Clone)]
pub enum CollisionShape {
    Circle { radius: f32 },
    Rect { width: f32, height: f32 },
    Polygon { points: Vec<Vec2> },
}

/// An instance of a map object placed in the world.
#[derive(Debug, Clone)]
pub struct PlacedObject {
    /// Unique instance ID.
    pub id: MapObjectId,
    /// Name of the object type (references MapObject::name).
    pub object_type: String,
    /// World position.
    pub position: Vec2,
    /// Rotation in radians.
    pub rotation: f32,
    /// Scale multiplier.
    pub scale: Vec2,
    /// Z-order layer.
    pub layer: i32,
    /// Instance-specific property overrides.
    pub property_overrides: Vec<ObjectProperty>,
    /// Whether this object is selected.
    pub selected: bool,
    /// Whether this object is locked (can't be moved/deleted).
    pub locked: bool,
    /// Optional name for this instance.
    pub instance_name: Option<String>,
}

impl PlacedObject {
    /// Create a new placed object.
    pub fn new(object_type: impl Into<String>, position: Vec2) -> Self {
        Self {
            id: MapObjectId::new(),
            object_type: object_type.into(),
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
            layer: 0,
            property_overrides: Vec::new(),
            selected: false,
            locked: false,
            instance_name: None,
        }
    }

    /// Set the rotation.
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set the scale.
    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    /// Set the layer.
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    /// Set the instance name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.instance_name = Some(name.into());
        self
    }

    /// Get the bounds of this placed object (for selection/collision).
    pub fn bounds(&self, object_def: &MapObject) -> (Vec2, Vec2) {
        // Component-wise multiplication for scale
        let scaled_size = Vec2::new(
            object_def.size.x * self.scale.x,
            object_def.size.y * self.scale.y,
        );
        let half_size = scaled_size * 0.5;
        let pivot_offset = Vec2::new(0.5, 0.5) - object_def.pivot;
        let offset = Vec2::new(
            pivot_offset.x * scaled_size.x,
            pivot_offset.y * scaled_size.y,
        );
        let center = self.position + offset;
        (center - half_size, center + half_size)
    }

    /// Check if a point is inside this object's bounds.
    pub fn contains_point(&self, point: Vec2, object_def: &MapObject) -> bool {
        let (min, max) = self.bounds(object_def);
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }
}
