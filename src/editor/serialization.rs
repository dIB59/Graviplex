//! Map serialization for saving and loading.

use std::collections::HashMap;
use std::io;
use std::path::Path;

#[cfg(feature = "serialize")]
use std::fs;

use crate::core::color::Color;
use crate::core::math::Vec2;

use super::map_object::{PlacedObject, ObjectProperty, PropertyValue};

/// Serializable map data format.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct MapData {
    /// Map name.
    pub name: String,
    /// Map version (for migration).
    pub version: u32,
    /// Map dimensions (optional, for bounded maps).
    pub width: Option<f32>,
    pub height: Option<f32>,
    /// All placed objects.
    pub objects: Vec<SerializedObject>,
    /// Custom map properties.
    pub properties: HashMap<String, String>,
}

impl Default for MapData {
    fn default() -> Self {
        Self {
            name: "Untitled".to_string(),
            version: 1,
            width: None,
            height: None,
            objects: Vec::new(),
            properties: HashMap::new(),
        }
    }
}

impl MapData {
    /// Create a new empty map.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the map dimensions.
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Add a property.
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Convert placed objects to serializable format.
    pub fn from_placed_objects(objects: &[PlacedObject]) -> Vec<SerializedObject> {
        objects.iter().map(SerializedObject::from_placed).collect()
    }

    /// Convert serializable objects back to placed objects.
    pub fn to_placed_objects(&self) -> Vec<PlacedObject> {
        self.objects.iter().map(|so| so.to_placed()).collect()
    }
}

/// Serializable representation of a placed object.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct SerializedObject {
    /// Object type name.
    pub object_type: String,
    /// Position.
    pub x: f32,
    pub y: f32,
    /// Rotation in radians.
    pub rotation: f32,
    /// Scale.
    pub scale_x: f32,
    pub scale_y: f32,
    /// Layer.
    pub layer: i32,
    /// Instance name (optional).
    pub name: Option<String>,
    /// Property overrides.
    pub properties: HashMap<String, SerializedProperty>,
    /// Whether locked.
    pub locked: bool,
}

impl SerializedObject {
    /// Create from a placed object.
    pub fn from_placed(obj: &PlacedObject) -> Self {
        let mut properties = HashMap::new();
        for prop in &obj.property_overrides {
            properties.insert(prop.name.clone(), SerializedProperty::from_value(&prop.value));
        }

        Self {
            object_type: obj.object_type.clone(),
            x: obj.position.x,
            y: obj.position.y,
            rotation: obj.rotation,
            scale_x: obj.scale.x,
            scale_y: obj.scale.y,
            layer: obj.layer,
            name: obj.instance_name.clone(),
            properties,
            locked: obj.locked,
        }
    }

    /// Convert to a placed object.
    pub fn to_placed(&self) -> PlacedObject {
        let mut obj = PlacedObject::new(&self.object_type, Vec2::new(self.x, self.y))
            .with_rotation(self.rotation)
            .with_scale(Vec2::new(self.scale_x, self.scale_y))
            .with_layer(self.layer);

        obj.locked = self.locked;
        obj.instance_name = self.name.clone();

        for (name, value) in &self.properties {
            obj.property_overrides.push(ObjectProperty {
                name: name.clone(),
                value: value.to_property_value(),
                editable: true,
            });
        }

        obj
    }
}

/// Serializable property value.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum SerializedProperty {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Color([f32; 4]),
    Vec2([f32; 2]),
}

impl SerializedProperty {
    /// Create from a property value.
    pub fn from_value(value: &PropertyValue) -> Self {
        match value {
            PropertyValue::String(s) => Self::String(s.clone()),
            PropertyValue::Int(i) => Self::Int(*i),
            PropertyValue::Float(f) => Self::Float(*f),
            PropertyValue::Bool(b) => Self::Bool(*b),
            PropertyValue::Color(c) => Self::Color([c.r, c.g, c.b, c.a]),
            PropertyValue::Vec2(v) => Self::Vec2([v.x, v.y]),
        }
    }

    /// Convert to a property value.
    pub fn to_property_value(&self) -> PropertyValue {
        match self {
            Self::String(s) => PropertyValue::String(s.clone()),
            Self::Int(i) => PropertyValue::Int(*i),
            Self::Float(f) => PropertyValue::Float(*f),
            Self::Bool(b) => PropertyValue::Bool(*b),
            Self::Color([r, g, b, a]) => PropertyValue::Color(Color::rgba(*r, *g, *b, *a)),
            Self::Vec2([x, y]) => PropertyValue::Vec2(Vec2::new(*x, *y)),
        }
    }
}

/// Error type for map operations.
#[derive(Debug)]
pub enum MapError {
    /// IO error.
    Io(io::Error),
    /// Serialization error.
    #[cfg(feature = "serialize")]
    Serialize(serde_json::Error),
    /// Feature not enabled.
    FeatureNotEnabled(&'static str),
}

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapError::Io(e) => write!(f, "IO error: {}", e),
            #[cfg(feature = "serialize")]
            MapError::Serialize(e) => write!(f, "Serialization error: {}", e),
            MapError::FeatureNotEnabled(feature) => {
                write!(f, "Feature '{}' not enabled", feature)
            }
        }
    }
}

impl std::error::Error for MapError {}

impl From<io::Error> for MapError {
    fn from(e: io::Error) -> Self {
        MapError::Io(e)
    }
}

#[cfg(feature = "serialize")]
impl From<serde_json::Error> for MapError {
    fn from(e: serde_json::Error) -> Self {
        MapError::Serialize(e)
    }
}

/// Save a map to a JSON file.
///
/// Requires the `serialize` feature.
#[cfg(feature = "serialize")]
pub fn save_map(path: impl AsRef<Path>, data: &MapData) -> Result<(), MapError> {
    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json)?;
    Ok(())
}

/// Save a map to a JSON file (stub when serialize feature is disabled).
#[cfg(not(feature = "serialize"))]
pub fn save_map(_path: impl AsRef<Path>, _data: &MapData) -> Result<(), MapError> {
    Err(MapError::FeatureNotEnabled("serialize"))
}

/// Load a map from a JSON file.
///
/// Requires the `serialize` feature.
#[cfg(feature = "serialize")]
pub fn load_map(path: impl AsRef<Path>) -> Result<MapData, MapError> {
    let json = fs::read_to_string(path)?;
    let data = serde_json::from_str(&json)?;
    Ok(data)
}

/// Load a map from a JSON file (stub when serialize feature is disabled).
#[cfg(not(feature = "serialize"))]
pub fn load_map(_path: impl AsRef<Path>) -> Result<MapData, MapError> {
    Err(MapError::FeatureNotEnabled("serialize"))
}

/// Save map data to a string.
#[cfg(feature = "serialize")]
pub fn save_map_to_string(data: &MapData) -> Result<String, MapError> {
    Ok(serde_json::to_string_pretty(data)?)
}

/// Save map data to a string (stub).
#[cfg(not(feature = "serialize"))]
pub fn save_map_to_string(_data: &MapData) -> Result<String, MapError> {
    Err(MapError::FeatureNotEnabled("serialize"))
}

/// Load map data from a string.
#[cfg(feature = "serialize")]
pub fn load_map_from_string(json: &str) -> Result<MapData, MapError> {
    Ok(serde_json::from_str(json)?)
}

/// Load map data from a string (stub).
#[cfg(not(feature = "serialize"))]
pub fn load_map_from_string(_json: &str) -> Result<MapData, MapError> {
    Err(MapError::FeatureNotEnabled("serialize"))
}
