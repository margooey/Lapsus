use objc2::{ClassType, MainThreadMarker, MainThreadOnly, rc::Retained};
use objc2_app_kit::{
    NSGridCell, NSGridCellPlacement, NSGridColumn, NSGridRow, NSGridRowAlignment, NSGridView,
    NSView,
};
use objc2_core_foundation::CGFloat;
use objc2_foundation::{NSArray, NSRect};

use crate::view::View;

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

    pub fn set_x_placeholder(&self, x_placement: NSGridCellPlacement) {
        self.grid_view.setXPlacement(x_placement)
    }

    pub fn set_row_alignment(&self, row_alignment: NSGridRowAlignment) {
        self.grid_view.setRowAlignment(row_alignment)
    }

    pub fn column_at_index(&self, index: isize) -> Retained<NSGridColumn> {
        self.grid_view.columnAtIndex(index)
    }

    pub fn row_at_index(&self, index: isize) -> Retained<NSGridRow> {
        self.grid_view.rowAtIndex(index)
    }

    /// Placeholder view for a grid cell that should stay empty.
    pub fn empty_cell(mtm: MainThreadMarker) -> Retained<NSView> {
        NSGridCell::emptyContentView(mtm)
    }

    pub fn set_translates_autoresizing_mask_into_constraints(
        &self,
        translates_autoresizing_mask_into_constraints: bool,
    ) {
        self.grid_view.setTranslatesAutoresizingMaskIntoConstraints(
            translates_autoresizing_mask_into_constraints,
        );
    }
}
