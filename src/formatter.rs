use crate::arguments::Arguments;
use crate::buffer::{Buffer, BufferSnapshot};
use crate::context::FormatContext;
use crate::format_element::FormatElement;
use crate::options::FormatOptions;
use crate::state::FormatState;

pub struct Formatter<'a> {
    buffer: &'a mut dyn Buffer,
}

impl<'a> Formatter<'a> {
    pub fn new(buffer: &'a mut (dyn Buffer + 'a)) -> Self {
        Self { buffer }
    }
}

impl<'a> Formatter<'a> {
    /// Returns the format options
    pub fn options(&self) -> &FormatOptions {
        self.context().options()
    }

    /// Returns the Context specifying how to format the current CST
    pub fn context(&self) -> &FormatContext {
        self.state().context()
    }

    /// Returns a mutable reference to the context.
    pub fn context_mut(&mut self) -> &mut FormatContext {
        self.state_mut().context_mut()
    }
}

impl Buffer for Formatter<'_> {
    #[inline(always)]
    fn write_element(&mut self, element: FormatElement) {
        self.buffer.write_element(element);
    }

    fn elements(&self) -> &[FormatElement] {
        self.buffer.elements()
    }

    #[inline(always)]
    fn write_fmt(&mut self, arguments: Arguments) {
        for argument in arguments.items() {
            argument.format(self);
        }
    }

    fn state(&self) -> &FormatState {
        self.buffer.state()
    }

    fn state_mut(&mut self) -> &mut FormatState {
        self.buffer.state_mut()
    }

    fn snapshot(&self) -> BufferSnapshot {
        self.buffer.snapshot()
    }

    fn restore_snapshot(&mut self, snapshot: BufferSnapshot) {
        self.buffer.restore_snapshot(snapshot)
    }
}
