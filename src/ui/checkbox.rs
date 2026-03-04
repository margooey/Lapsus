use objc2::{AnyThread, DefinedClass, define_class, msg_send, rc::Retained, sel};
use objc2_app_kit::{NSButton, NSControlStateValue};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSRect, NSString};

struct ActionIvars {
    callback: Box<dyn Fn(&NSButton) + 'static>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "CheckboxActionTarget"]
    #[ivars = ActionIvars]
    struct CheckboxActionTarget;

    impl CheckboxActionTarget {
        #[unsafe(method(checkboxClicked:))]
        fn checkbox_clicked(&self, sender: &NSButton) {
            (self.ivars().callback)(sender);
        }
    }

    unsafe impl NSObjectProtocol for CheckboxActionTarget {}
);

impl CheckboxActionTarget {
    fn new<F>(_: MainThreadMarker, f: F) -> Retained<Self>
    where
        F: Fn(&NSButton) + 'static,
    {
        let this = Self::alloc().set_ivars(ActionIvars {
            callback: Box::new(f),
        });
        unsafe { msg_send![super(this), init] }
    }
}

pub struct Checkbox {
    pub button: Retained<NSButton>,
    _target: Option<Retained<CheckboxActionTarget>>,
}

impl Checkbox {
    pub fn init_with_title(mtm: MainThreadMarker, title: &str) -> Self {
        let button = unsafe {
            NSButton::checkboxWithTitle_target_action(&NSString::from_str(title), None, None, mtm)
        };
        Self {
            button,
            _target: None,
        }
    }

    pub fn set_action<F>(&mut self, mtm: MainThreadMarker, f: F)
    where
        F: Fn(&NSButton) + 'static,
    {
        let target = CheckboxActionTarget::new(mtm, f);
        unsafe {
            self.button.setTarget(Some(&target));
            self.button.setAction(Some(sel!(checkboxClicked:)));
        }
        self._target = Some(target);
    }

    pub fn set_state(&self, state: NSControlStateValue) {
        self.button.setState(state);
    }

    pub fn size_to_fit(&self) {
        self.button.sizeToFit();
    }

    pub fn frame(&self) -> NSRect {
        self.button.frame()
    }

    pub fn set_frame(&self, frame: NSRect) {
        self.button.setFrame(frame);
    }
}
