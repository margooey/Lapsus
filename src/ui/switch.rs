// WIP
use objc2::{AnyThread, DefinedClass, MainThreadOnly, define_class, msg_send, rc::Retained, sel};
use objc2_app_kit::{NSControlStateValue, NSControlStateValueOn, NSSwitch};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol};

struct ActionIvars {
    callback: Box<dyn Fn(&NSSwitch) + 'static>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "SwitchActionTarget"]
    #[ivars = ActionIvars]
    struct SwitchActionTarget;

    impl SwitchActionTarget {
        #[unsafe(method(switchValueChanged:))]
        fn switch_toggled(&self, sender: &NSSwitch) {
            (self.ivars().callback)(sender);
        }
    }

    unsafe impl NSObjectProtocol for SwitchActionTarget {}
);

impl SwitchActionTarget {
    fn new<F>(_: MainThreadMarker, f: F) -> Retained<Self>
    where
        F: Fn(&NSSwitch) + 'static,
    {
        let this = Self::alloc().set_ivars(ActionIvars {
            callback: Box::new(f),
        });
        unsafe { msg_send![super(this), init] }
    }
}

pub struct Switch {
    pub switch: Retained<NSSwitch>,
    _target: Option<Retained<SwitchActionTarget>>,
}

impl Switch {
    pub fn init(mtm: MainThreadMarker) -> Self {
        let switch = NSSwitch::init(NSSwitch::alloc(mtm));
        Self {
            switch,
            _target: None,
        }
    }

    pub fn set_action<F>(&mut self, mtm: MainThreadMarker, f: F)
    where
        F: Fn(&NSSwitch) + 'static,
    {
        let target = SwitchActionTarget::new(mtm, f);
        unsafe {
            self.switch.setTarget(Some(&target));
            self.switch.setAction(Some(sel!(switchValueChanged:)));
        }
        self._target = Some(target);
    }
    pub fn set_state(&self, state: NSControlStateValue) {
        self.switch.setState(state);
    }
}
