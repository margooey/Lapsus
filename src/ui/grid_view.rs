use objc2::{ClassType, MainThreadMarker, MainThreadOnly, rc::Retained};
use objc2_app_kit::{NSGridRow, NSGridView, NSView};
use objc2_core_foundation::CGFloat;
use objc2_foundation::{NSArray, NSRect};

use crate::ui::view::View;

pub struct GridView {
    pub grid_view: Retained<NSGridView>,
}

impl GridView {
    pub fn new(mtm: MainThreadMarker, frame: NSRect) -> Self {
        let grid_view = NSGridView::initWithFrame(NSGridView::alloc(mtm), frame);
        Self { grid_view }
    }

    pub fn add_row_with_views(&self, views: &[&NSView]) -> Retained<NSGridRow> {
        let views = NSArray::from_slice(views);
        self.grid_view.addRowWithViews(&views)
    }

    pub fn as_view(&self) -> View {
        View {
            view: self.grid_view.as_super().into(),
        }
    }

    pub fn set_column_spacing(&self, column_spacing: CGFloat) {
        self.grid_view.setColumnSpacing(column_spacing)
    }

    pub fn set_row_spacing(&self, row_spacing: CGFloat) {
        self.grid_view.setRowSpacing(row_spacing)
    }
}
