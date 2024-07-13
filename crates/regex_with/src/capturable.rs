use regex::{CaptureNames, Captures};

pub trait Capturable {
    fn captures<'h>(&self, heystack: &'h str) -> Option<(CaptureNames, Captures<'h>)>;
}
