use crate::options::FormatOptions;

#[derive(Debug, Clone)]
pub struct FormatContext {
    options: FormatOptions,
    // TODO: Comments
}

impl FormatContext {
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }

    pub fn options(&self) -> &FormatOptions {
        &self.options
    }
}
