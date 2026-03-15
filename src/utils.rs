use cidre::cg::{EventSrc, EventSrcStateId, Point, Rect, Size};

pub fn min(a: f64, b: f64) -> f64 {
    if a > b { b } else { a }
}

pub fn max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

pub fn union_rect(a: &Rect, b: &Rect) -> Rect {
    if *a == Rect::null() {
        return *b;
    }
    if *b == Rect::null() {
        return *a;
    }
    let min_x = min(a.origin.x, b.origin.x);
    let min_y = min(a.origin.y, b.origin.y);
    let max_x = max(a.origin.x + a.size.width, b.origin.x + b.size.width);
    let max_y = max(a.origin.y + a.size.height, b.origin.y + b.size.height);

    Rect {
        origin: Point { x: min_x, y: min_y },
        size: Size {
            width: max_x - min_x,
            height: max_y - min_y,
        },
    }
}

pub fn disable_local_event_suppression() {
    let state_id = EventSrcStateId::CombinedSession;
    let mut event_source_ref = EventSrc::with_state(state_id);
    if let Some(ref mut retained) = event_source_ref {
        EventSrc::set_local_events_suppression_interval(retained, 0.0);
    }
}

// My first rust macro. I got tired of writing "NSRect::new(NSPoint::new..."
macro_rules! new_nsrect {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        NSRect::new(NSPoint::new($a, $b), NSSize::new($c, $d))
    };
}
pub(crate) use new_nsrect;

macro_rules! env_f64 {
    ($name:literal) => {
        env!($name).parse::<f64>().unwrap()
    };
}
pub(crate) use env_f64;
