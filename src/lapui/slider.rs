use objc2::{AnyThread, DefinedClass, define_class, msg_send, rc::Retained, sel};
use objc2_app_kit::NSSlider;
use objc2_foundation::{MainThreadMarker, NSInteger, NSObject, NSObjectProtocol};

struct ActionIvars {
    callback: Box<dyn Fn(&NSSlider) + 'static>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "SliderActionTarget"]
    #[ivars = ActionIvars]
    struct SliderActionTarget;

    impl SliderActionTarget {
        #[unsafe(method(sliderMoved:))]
        fn slider_moved(&self, sender: &NSSlider) {
            (self.ivars().callback)(sender);
        }
    }

    unsafe impl NSObjectProtocol for SliderActionTarget {}
);

impl SliderActionTarget {
    fn new<F>(_: MainThreadMarker, f: F) -> Retained<Self>
    where
        F: Fn(&NSSlider) + 'static,
    {
        let this = Self::alloc().set_ivars(ActionIvars {
            callback: Box::new(f),
        });
        unsafe { msg_send![super(this), init] }
    }
}

pub struct Slider {
    pub slider: Retained<NSSlider>,
    _target: Option<Retained<SliderActionTarget>>,
}

impl Slider {
    pub fn new(mtm: MainThreadMarker, value: f64, min: f64, max: f64) -> Self {
        let slider = unsafe {
            NSSlider::sliderWithValue_minValue_maxValue_target_action(
                value, min, max, None, None, mtm,
            )
        };
        Self {
            slider,
            _target: None,
        }
    }

    pub fn set_action<F>(&mut self, mtm: MainThreadMarker, f: F)
    where
        F: Fn(&NSSlider) + 'static,
    {
        let target = SliderActionTarget::new(mtm, f);
        unsafe {
            self.slider.setTarget(Some(&target));
            self.slider.setAction(Some(sel!(sliderMoved:)));
        }
        self._target = Some(target);
    }

    pub fn value(&self) -> f64 {
        self.slider.doubleValue()
    }

    pub fn set_value(&self, value: f64) {
        self.slider.setDoubleValue(value);
    }

    /// When false, the action only fires once the user lets go of the knob.
    pub fn set_continuous(&self, continuous: bool) {
        self.slider.setContinuous(continuous);
    }

    pub fn set_number_of_tick_marks(&self, count: NSInteger) {
        self.slider.setNumberOfTickMarks(count);
    }

    pub fn set_allows_tick_mark_values_only(&self, snap: bool) {
        self.slider.setAllowsTickMarkValuesOnly(snap);
    }
}
