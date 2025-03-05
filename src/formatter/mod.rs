mod js;

use crate::format_element::FormatElement;

pub struct Formatter<'a> {
    pub elements: Vec<FormatElement>,
    pub source_text: &'a str,
}

impl<'a> Formatter<'a> {
    pub fn new(source_text: &'a str) -> Self {
        Self { elements: vec![], source_text }
    }

    fn write_element(&mut self, element: FormatElement) {
        self.elements.push(element);
    }
}

pub trait Format {
    fn fmt(&self, f: &mut Formatter) {
        self.fmt_fields(f);
    }

    fn fmt_fields(&self, f: &mut Formatter);
}
