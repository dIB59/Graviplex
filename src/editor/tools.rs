//! Editor tools for interacting with the map.

use crate::core::math::Vec2;

/// Available editor tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTool {
    /// Select and move objects.
    #[default]
    Select,
    /// Place new objects from the palette.
    Place,
    /// Erase/delete objects.
    Erase,
    /// Pan the camera.
    Pan,
    /// Measure distances.
    Measure,
    /// Paint with selected object (continuous placement).
    Paint,
}

impl EditorTool {
    /// Get all available tools.
    pub fn all() -> &'static [EditorTool] {
        &[
            EditorTool::Select,
            EditorTool::Place,
            EditorTool::Erase,
            EditorTool::Pan,
            EditorTool::Paint,
            EditorTool::Measure,
        ]
    }

    /// Get the display name for this tool.
    pub fn name(&self) -> &'static str {
        match self {
            EditorTool::Select => "Select",
            EditorTool::Place => "Place",
            EditorTool::Erase => "Erase",
            EditorTool::Pan => "Pan",
            EditorTool::Measure => "Measure",
            EditorTool::Paint => "Paint",
        }
    }

    /// Get the icon/emoji for this tool.
    pub fn icon(&self) -> &'static str {
        match self {
            EditorTool::Select => "⬚",
            EditorTool::Place => "➕",
            EditorTool::Erase => "🗑",
            EditorTool::Pan => "✋",
            EditorTool::Measure => "📏",
            EditorTool::Paint => "🖌",
        }
    }

    /// Get the keyboard shortcut for this tool.
    pub fn shortcut(&self) -> Option<char> {
        match self {
            EditorTool::Select => Some('V'),
            EditorTool::Place => Some('P'),
            EditorTool::Erase => Some('E'),
            EditorTool::Pan => Some('H'),
            EditorTool::Measure => Some('M'),
            EditorTool::Paint => Some('B'),
        }
    }
}

/// State for the current tool operation.
#[derive(Debug, Clone, Default)]
pub struct ToolState {
    /// Currently active tool.
    pub tool: EditorTool,
    /// Is the tool currently being used (mouse down)?
    pub active: bool,
    /// Start position of the current operation.
    pub start_pos: Option<Vec2>,
    /// Current position during operation.
    pub current_pos: Option<Vec2>,
    /// Objects being dragged (for Select tool).
    pub drag_offset: Option<Vec2>,
    /// Last paint position (for Paint tool to avoid duplicates).
    pub last_paint_pos: Option<Vec2>,
    /// Paint spacing (minimum distance between painted objects).
    pub paint_spacing: f32,
}

impl ToolState {
    /// Create a new tool state.
    pub fn new() -> Self {
        Self {
            paint_spacing: 32.0,
            ..Default::default()
        }
    }

    /// Begin a tool operation.
    pub fn begin(&mut self, world_pos: Vec2) {
        self.active = true;
        self.start_pos = Some(world_pos);
        self.current_pos = Some(world_pos);
    }

    /// Update the current position during operation.
    pub fn update(&mut self, world_pos: Vec2) {
        self.current_pos = Some(world_pos);
    }

    /// End the current operation.
    pub fn end(&mut self) {
        self.active = false;
        self.start_pos = None;
        self.current_pos = None;
        self.drag_offset = None;
        self.last_paint_pos = None;
    }

    /// Get the selection rectangle (for Select tool).
    pub fn selection_rect(&self) -> Option<(Vec2, Vec2)> {
        match (self.start_pos, self.current_pos) {
            (Some(start), Some(current)) => {
                let min = Vec2::new(start.x.min(current.x), start.y.min(current.y));
                let max = Vec2::new(start.x.max(current.x), start.y.max(current.y));
                Some((min, max))
            }
            _ => None,
        }
    }

    /// Check if we should paint at the current position.
    pub fn should_paint(&self, current_pos: Vec2) -> bool {
        match self.last_paint_pos {
            Some(last) => (current_pos - last).length() >= self.paint_spacing,
            None => true,
        }
    }

    /// Record a paint position.
    pub fn record_paint(&mut self, pos: Vec2) {
        self.last_paint_pos = Some(pos);
    }
}

/// Grid snapping configuration.
#[derive(Debug, Clone)]
pub struct GridConfig {
    /// Whether grid snapping is enabled.
    pub enabled: bool,
    /// Grid cell size.
    pub cell_size: Vec2,
    /// Grid origin offset.
    pub offset: Vec2,
    /// Whether to show the grid.
    pub visible: bool,
    /// Grid line color.
    pub color: [f32; 4],
    /// Major grid line interval (every N cells).
    pub major_interval: u32,
    /// Major grid line color.
    pub major_color: [f32; 4],
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cell_size: Vec2::new(32.0, 32.0),
            offset: Vec2::ZERO,
            visible: true,
            color: [0.3, 0.3, 0.3, 0.5],
            major_interval: 4,
            major_color: [0.4, 0.4, 0.4, 0.7],
        }
    }
}

impl GridConfig {
    /// Snap a position to the grid.
    pub fn snap(&self, pos: Vec2) -> Vec2 {
        if !self.enabled {
            return pos;
        }
        let adjusted = pos - self.offset;
        let snapped = Vec2::new(
            (adjusted.x / self.cell_size.x).round() * self.cell_size.x,
            (adjusted.y / self.cell_size.y).round() * self.cell_size.y,
        );
        snapped + self.offset
    }

    /// Snap to the nearest grid cell center.
    pub fn snap_to_center(&self, pos: Vec2) -> Vec2 {
        if !self.enabled {
            return pos;
        }
        let snapped = self.snap(pos);
        snapped + self.cell_size * 0.5
    }
}
