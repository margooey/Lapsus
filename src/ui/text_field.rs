use objc2::{MainThreadMarker, MainThreadOnly, rc::Retained};
use objc2_app_kit::NSTextField;
use objc2_foundation::NSString;

pub struct TextField {
    pub text_field: Retained<NSTextField>,
}

impl TextField {
    pub fn init(mtm: MainThreadMarker) -> Self {
        let text_field = NSTextField::init(NSTextField::alloc(mtm));
        Self { text_field }
    }
    pub fn set_string_value(&self, string_value: &str) {
        self.text_field
            .setStringValue(&NSString::from_str(string_value))
    }
    pub fn set_editable(&self, editable: bool) {
        self.text_field.setEditable(editable);
    }
    pub fn set_bordered(&self, bordered: bool) {
        self.text_field.setBordered(bordered);
    }
}
