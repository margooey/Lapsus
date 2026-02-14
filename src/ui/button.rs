/*
    Warning, this implementation is of my own design, but was implemented with help from an LLM.
    I haven't fully reviewed this in depth, but it does work so it's going to stay for now.
    WIP, but basically abstracts the need to create a class for each button manually in order
    to set a target and action on click.
 */
use objc2::{AnyThread, DefinedClass, MainThreadOnly, define_class, msg_send, rc::Retained, sel};
use objc2_app_kit::NSButton;
use objc2_core_foundation::CGRect;
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSString};

struct ActionIvars {
    callback: Box<dyn Fn(&NSButton) + 'static>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "ActionTarget"]
    #[ivars = ActionIvars]
    struct ActionTarget;

    impl ActionTarget {
        #[unsafe(method(buttonClicked:))]
        fn button_clicked(&self, sender: &NSButton) {
            (self.ivars().callback)(sender);
        }
    }

    unsafe impl NSObjectProtocol for ActionTarget {}
);

impl ActionTarget {
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

pub struct Button {
    pub button: Retained<NSButton>,
    _target: Option<Retained<ActionTarget>>,
}

impl Button {
    pub fn new(mtm: MainThreadMarker, rect: CGRect) -> Self {
        let button = NSButton::initWithFrame(NSButton::alloc(mtm), rect);
        Self {
            button,
            _target: None,
        }
    }

    pub fn set_title(&mut self, string: &str) {
        self.button.setTitle(&NSString::from_str(string));
    }

    pub fn set_action<F>(&mut self, mtm: MainThreadMarker, f: F)
    where
        F: Fn(&NSButton) + 'static,
    {
        let target = ActionTarget::new(mtm, f);
        unsafe {
            self.button.setTarget(Some(&target));
            self.button.setAction(Some(sel!(buttonClicked:)));
        }
        self._target = Some(target);
    }
}
