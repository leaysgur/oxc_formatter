use crate::context::FormatContext;
use crate::group_id::{GroupId, UniqueGroupIdBuilder};

pub struct FormatState {
    context: FormatContext,
    group_id_builder: UniqueGroupIdBuilder,
}
impl FormatState {
    pub fn new(context: FormatContext) -> Self {
        Self {
            context,
            group_id_builder: UniqueGroupIdBuilder::default(),
        }
    }

    pub fn into_context(self) -> FormatContext {
        self.context
    }

    /// Returns the context specifying how to format the current CST
    pub fn context(&self) -> &FormatContext {
        &self.context
    }

    /// Returns a mutable reference to the context
    pub fn context_mut(&mut self) -> &mut FormatContext {
        &mut self.context
    }

    /// Creates a new group id that is unique to this document.
    /// The passed debug name is used in the [std::fmt::Debug] of the document if this is a debug build.
    /// The name is unused for production builds and has no meaning on the equality of two group ids.
    pub fn group_id(&self, debug_name: &'static str) -> GroupId {
        self.group_id_builder.group_id(debug_name)
    }
}

impl std::fmt::Debug for FormatState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FormatState")
            .field("context", &self.context)
            .finish()
    }
}
