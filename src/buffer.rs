#![expect(clippy::mutable_key_type)]
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use rustc_hash::FxHashMap;

use crate::FormatState;
use crate::arguments::Arguments;
use crate::format_element::FormatElement;
use crate::format_element::{
    Interned, LineMode, PrintMode,
    tag::{Condition, Tag},
};
use crate::write_with_formatter;

/// A trait for writing or formatting into [FormatElement]-accepting buffers or streams.
pub trait Buffer {
    /// Writes a [crate::FormatElement] into this buffer, returning whether the write succeeded.
    ///
    /// # Errors
    /// This function will return an instance of [crate::FormatError] on error.
    ///
    /// # Examples
    ///
    /// ```
    /// use biome_formatter::{Buffer, FormatElement, FormatState, SimpleFormatContext, VecBuffer};
    ///
    /// let mut state = FormatState::new(SimpleFormatContext::default());
    /// let mut buffer = VecBuffer::new(&mut state);
    ///
    /// buffer.write_element(FormatElement::StaticText { text: "test"}).unwrap();
    ///
    /// assert_eq!(buffer.into_vec(), vec![FormatElement::StaticText { text: "test" }]);
    /// ```
    ///
    fn write_element(&mut self, element: FormatElement);

    /// Returns a slice containing all elements written into this buffer.
    ///
    /// Prefer using [BufferExtensions::start_recording] over accessing [Buffer::elements] directly.
    #[doc(hidden)]
    fn elements(&self) -> &[FormatElement];

    /// Glue for usage of the [`write!`] macro with implementors of this trait.
    ///
    /// This method should generally not be invoked manually, but rather through the [`write!`] macro itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use biome_formatter::prelude::*;
    /// use biome_formatter::{Buffer, FormatState, SimpleFormatContext, VecBuffer, format_args};
    ///
    /// let mut state = FormatState::new(SimpleFormatContext::default());
    /// let mut buffer = VecBuffer::new(&mut state);
    ///
    /// buffer.write_fmt(format_args!(text("Hello World"))).unwrap();
    ///
    /// assert_eq!(buffer.into_vec(), vec![FormatElement::StaticText{ text: "Hello World" }]);
    /// ```
    fn write_fmt(mut self: &mut Self, arguments: Arguments) {
        write_with_formatter(&mut self, arguments);
    }

    /// Returns the formatting state relevant for this formatting session.
    fn state(&self) -> &FormatState;

    /// Returns the mutable formatting state relevant for this formatting session.
    fn state_mut(&mut self) -> &mut FormatState;

    /// Takes a snapshot of the Buffers state, excluding the formatter state.
    fn snapshot(&self) -> BufferSnapshot;

    /// Restores the snapshot buffer
    ///
    /// ## Panics
    /// If the passed snapshot id is a snapshot of another buffer OR
    /// if the snapshot is restored out of order
    fn restore_snapshot(&mut self, snapshot: BufferSnapshot);
}

/// Snapshot of a buffer state that can be restored at a later point.
///
/// Used in cases where the formatting of an object fails but a parent formatter knows an alternative
/// strategy on how to format the object that might succeed.
#[derive(Debug)]
pub enum BufferSnapshot {
    /// Stores an absolute position of a buffers state, for example, the offset of the last written element.
    Position(usize),

    /// Generic structure for custom buffers that need to store more complex data. Slightly more
    /// expensive because it requires allocating the buffer state on the heap.
    Any(Box<dyn Any>),
}

impl BufferSnapshot {
    /// Creates a new buffer snapshot that points to the specified position.
    pub const fn position(index: usize) -> Self {
        Self::Position(index)
    }

    /// Unwraps the position value.
    ///
    /// # Panics
    ///
    /// If self is not a [`BufferSnapshot::Position`]
    pub fn unwrap_position(&self) -> usize {
        match self {
            BufferSnapshot::Position(index) => *index,
            BufferSnapshot::Any(_) => panic!("Tried to unwrap Any snapshot as a position."),
        }
    }

    /// Unwraps the any value.
    ///
    /// # Panics
    ///
    /// If `self` is not a [`BufferSnapshot::Any`].
    pub fn unwrap_any<T: 'static>(self) -> T {
        match self {
            BufferSnapshot::Position(_) => {
                panic!("Tried to unwrap Position snapshot as Any snapshot.")
            }
            BufferSnapshot::Any(value) => match value.downcast::<T>() {
                Ok(snapshot) => *snapshot,
                Err(err) => {
                    panic!(
                        "Tried to unwrap snapshot of type {:?} as {:?}",
                        (*err).type_id(),
                        TypeId::of::<T>()
                    )
                }
            },
        }
    }
}

/// Implements the `[Buffer]` trait for all mutable references of objects implementing [Buffer].
impl<W: Buffer + ?Sized> Buffer for &mut W {
    fn write_element(&mut self, element: FormatElement) {
        (**self).write_element(element);
    }

    fn elements(&self) -> &[FormatElement] {
        (**self).elements()
    }

    fn write_fmt(&mut self, args: Arguments) {
        (**self).write_fmt(args);
    }

    fn state(&self) -> &FormatState {
        (**self).state()
    }

    fn state_mut(&mut self) -> &mut FormatState {
        (**self).state_mut()
    }

    fn snapshot(&self) -> BufferSnapshot {
        (**self).snapshot()
    }

    fn restore_snapshot(&mut self, snapshot: BufferSnapshot) {
        (**self).restore_snapshot(snapshot)
    }
}

/// Vector backed [`Buffer`] implementation.
///
/// The buffer writes all elements into the internal elements buffer.
#[derive(Debug)]
pub struct VecBuffer<'a> {
    state: &'a mut FormatState,
    elements: Vec<FormatElement>,
}

impl<'a> VecBuffer<'a> {
    pub fn new(state: &'a mut FormatState) -> Self {
        Self::new_with_vec(state, Vec::new())
    }

    pub fn new_with_vec(state: &'a mut FormatState, elements: Vec<FormatElement>) -> Self {
        Self { state, elements }
    }

    /// Creates a buffer with the specified capacity
    pub fn with_capacity(capacity: usize, state: &'a mut FormatState) -> Self {
        Self {
            state,
            elements: Vec::with_capacity(capacity),
        }
    }

    /// Consumes the buffer and returns the written [`FormatElement]`s as a vector.
    pub fn into_vec(self) -> Vec<FormatElement> {
        self.elements
    }

    /// Takes the elements without consuming self
    pub fn take_vec(&mut self) -> Vec<FormatElement> {
        std::mem::take(&mut self.elements)
    }
}

impl Deref for VecBuffer<'_> {
    type Target = [FormatElement];

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

impl DerefMut for VecBuffer<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elements
    }
}

impl Buffer for VecBuffer<'_> {
    fn write_element(&mut self, element: FormatElement) {
        self.elements.push(element);
    }

    fn elements(&self) -> &[FormatElement] {
        self
    }

    fn state(&self) -> &FormatState {
        self.state
    }

    fn state_mut(&mut self) -> &mut FormatState {
        self.state
    }

    fn snapshot(&self) -> BufferSnapshot {
        BufferSnapshot::position(self.elements.len())
    }

    fn restore_snapshot(&mut self, snapshot: BufferSnapshot) {
        let position = snapshot.unwrap_position();
        assert!(
            self.elements.len() >= position,
            r#"Outdated snapshot. This buffer contains fewer elements than at the time the snapshot was taken.
Make sure that you take and restore the snapshot in order and that this snapshot belongs to the current buffer."#
        );

        self.elements.truncate(position);
    }
}

/// Buffer that allows you inspecting elements as they get written to the formatter.
pub struct Inspect<'inner, Inspector> {
    inner: &'inner mut dyn Buffer,
    inspector: Inspector,
}

impl<'inner, Inspector> Inspect<'inner, Inspector> {
    fn new(inner: &'inner mut dyn Buffer, inspector: Inspector) -> Self {
        Self { inner, inspector }
    }
}

impl<Inspector> Buffer for Inspect<'_, Inspector>
where
    Inspector: FnMut(&FormatElement),
{
    fn write_element(&mut self, element: FormatElement) {
        (self.inspector)(&element);
        self.inner.write_element(element);
    }

    fn elements(&self) -> &[FormatElement] {
        self.inner.elements()
    }

    fn state(&self) -> &FormatState {
        self.inner.state()
    }

    fn state_mut(&mut self) -> &mut FormatState {
        self.inner.state_mut()
    }

    fn snapshot(&self) -> BufferSnapshot {
        self.inner.snapshot()
    }

    fn restore_snapshot(&mut self, snapshot: BufferSnapshot) {
        self.inner.restore_snapshot(snapshot)
    }
}

/// A Buffer that removes any soft line breaks.
///
/// * Removes [`lines`](FormatElement::Line) with the mode [`Soft`](LineMode::Soft).
/// * Replaces [`lines`](FormatElement::Line) with the mode [`Soft`](LineMode::SoftOrSpace) with a [`Space`](FormatElement::Space)
///
/// # Examples
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{format, write};
///
/// # fn main() -> FormatResult<()> {
/// use biome_formatter::{RemoveSoftLinesBuffer, SimpleFormatContext, VecBuffer};
/// use biome_formatter::prelude::format_with;
/// let formatted = format!(
///     SimpleFormatContext::default(),
///     [format_with(|f| {
///         let mut buffer = RemoveSoftLinesBuffer::new(f);
///
///         write!(
///             buffer,
///             [
///                 text("The next soft line or space gets replaced by a space"),
///                 soft_line_break_or_space(),
///                 text("and the line here"),
///                 soft_line_break(),
///                 text("is removed entirely.")
///             ]
///         )
///     })]
/// )?;
///
/// assert_eq!(
///     formatted.document().as_ref(),
///     &[
///         FormatElement::StaticText { text: "The next soft line or space gets replaced by a space" },
///         FormatElement::Space,
///         FormatElement::StaticText { text: "and the line here" },
///         FormatElement::StaticText { text: "is removed entirely." }
///     ]
/// );
///
/// # Ok(())
/// # }
/// ```
pub struct RemoveSoftLinesBuffer<'a> {
    inner: &'a mut dyn Buffer,

    /// Caches the interned elements after the soft line breaks have been removed.
    ///
    /// The `key` is the [Interned] element as it has been passed to [Self::write_element] or the child of another
    /// [Interned] element. The `value` is the matching document of the key where all soft line breaks have been removed.
    ///
    /// It's fine to not snapshot the cache. The worst that can happen is that it holds on interned elements
    /// that are now unused. But there's little harm in that and the cache is cleaned when dropping the buffer.
    interned_cache: FxHashMap<Interned, Interned>,

    /// Store the conditional content stack to help determine if the current element is within expanded conditional content.
    conditional_content_stack: Vec<Condition>,
}

impl<'a> RemoveSoftLinesBuffer<'a> {
    /// Creates a new buffer that removes the soft line breaks before writing them into `buffer`.
    pub fn new(inner: &'a mut dyn Buffer) -> Self {
        Self {
            inner,
            interned_cache: FxHashMap::default(),
            conditional_content_stack: Vec::new(),
        }
    }

    /// Removes the soft line breaks from an interned element.
    fn clean_interned(&mut self, interned: &Interned) -> Interned {
        clean_interned(
            interned,
            &mut self.interned_cache,
            &mut self.conditional_content_stack,
        )
    }

    /// Marker for whether a `StartConditionalContent(mode: Expanded)` has been
    /// written but not yet closed.
    fn is_in_expanded_conditional_content(&self) -> bool {
        self.conditional_content_stack
            .iter()
            .last()
            .is_some_and(|condition| condition.mode == PrintMode::Expanded)
    }
}

// Extracted to function to avoid monomorphization
fn clean_interned(
    interned: &Interned,
    interned_cache: &mut FxHashMap<Interned, Interned>,
    condition_content_stack: &mut Vec<Condition>,
) -> Interned {
    match interned_cache.get(interned) {
        Some(cleaned) => cleaned.clone(),
        None => {
            // Find the first soft line break element, interned element, or conditional expanded
            // content that must be changed.
            let result = interned
                .iter()
                .enumerate()
                .find_map(|(index, element)| match element {
                    FormatElement::Line(LineMode::Soft | LineMode::SoftOrSpace)
                    | FormatElement::Tag(
                        Tag::StartConditionalContent(_) | Tag::EndConditionalContent,
                    )
                    | FormatElement::BestFitting(_) => {
                        let mut cleaned = Vec::new();
                        cleaned.extend_from_slice(&interned[..index]);
                        Some((cleaned, &interned[index..]))
                    }
                    FormatElement::Interned(inner) => {
                        let cleaned_inner =
                            clean_interned(inner, interned_cache, condition_content_stack);

                        if &cleaned_inner != inner {
                            let mut cleaned = Vec::with_capacity(interned.len());
                            cleaned.extend_from_slice(&interned[..index]);
                            cleaned.push(FormatElement::Interned(cleaned_inner));
                            Some((cleaned, &interned[index + 1..]))
                        } else {
                            None
                        }
                    }
                    _ => None,
                });

            let result = match result {
                // Copy the whole interned buffer so that becomes possible to change the necessary elements.
                Some((mut cleaned, rest)) => {
                    let mut element_stack = rest.iter().rev().collect::<Vec<_>>();
                    while let Some(element) = element_stack.pop() {
                        match element {
                            FormatElement::Tag(Tag::StartConditionalContent(condition)) => {
                                condition_content_stack.push(condition.clone());
                                continue;
                            }
                            FormatElement::Tag(Tag::EndConditionalContent) => {
                                condition_content_stack.pop();
                                continue;
                            }
                            // All content within an expanded conditional gets dropped. If there's a
                            // matching flat variant, that will still get kept.
                            _ if condition_content_stack
                                .iter()
                                .last()
                                .is_some_and(|condition| condition.mode == PrintMode::Expanded) =>
                            {
                                continue;
                            }

                            FormatElement::Line(LineMode::Soft) => continue,
                            FormatElement::Line(LineMode::SoftOrSpace) => {
                                cleaned.push(FormatElement::Space)
                            }

                            FormatElement::Interned(interned) => {
                                cleaned.push(FormatElement::Interned(clean_interned(
                                    interned,
                                    interned_cache,
                                    condition_content_stack,
                                )))
                            }
                            // Since this buffer aims to simulate infinite print width, we don't need to retain the best fitting.
                            // Just extract the flattest variant and then handle elements within it.
                            FormatElement::BestFitting(best_fitting) => {
                                let most_flat = best_fitting.most_flat();
                                most_flat
                                    .iter()
                                    .rev()
                                    .for_each(|element| element_stack.push(element));
                            }
                            element => cleaned.push(element.clone()),
                        };
                    }

                    Interned::new(cleaned)
                }
                // No change necessary, return existing interned element
                None => interned.clone(),
            };

            interned_cache.insert(interned.clone(), result.clone());
            result
        }
    }
}

impl Buffer for RemoveSoftLinesBuffer<'_> {
    fn write_element(&mut self, element: FormatElement) {
        let mut element_statck = Vec::new();
        element_statck.push(element);

        while let Some(element) = element_statck.pop() {
            match element {
                FormatElement::Tag(Tag::StartConditionalContent(condition)) => {
                    self.conditional_content_stack.push(condition.clone());
                }
                FormatElement::Tag(Tag::EndConditionalContent) => {
                    self.conditional_content_stack.pop();
                }
                // All content within an expanded conditional gets dropped. If there's a
                // matching flat variant, that will still get kept.
                _ if self.is_in_expanded_conditional_content() => continue,

                FormatElement::Line(LineMode::Soft) => continue,
                FormatElement::Line(LineMode::SoftOrSpace) => {
                    self.inner.write_element(FormatElement::Space)
                }
                FormatElement::Interned(interned) => {
                    let cleaned = self.clean_interned(&interned);
                    self.inner.write_element(FormatElement::Interned(cleaned))
                }
                // Since this buffer aims to simulate infinite print width, we don't need to retain the best fitting.
                // Just extract the flattest variant and then handle elements within it.
                FormatElement::BestFitting(best_fitting) => {
                    let most_flat = best_fitting.most_flat();
                    most_flat
                        .iter()
                        .rev()
                        .for_each(|element| element_statck.push(element.clone()));
                }
                element => self.inner.write_element(element),
            }
        }
    }

    fn elements(&self) -> &[FormatElement] {
        self.inner.elements()
    }

    fn state(&self) -> &FormatState {
        self.inner.state()
    }

    fn state_mut(&mut self) -> &mut FormatState {
        self.inner.state_mut()
    }

    fn snapshot(&self) -> BufferSnapshot {
        self.inner.snapshot()
    }

    fn restore_snapshot(&mut self, snapshot: BufferSnapshot) {
        self.inner.restore_snapshot(snapshot)
    }
}

pub trait BufferExtensions: Buffer + Sized {
    /// Returns a new buffer that calls the passed inspector for every element that gets written to the output
    #[must_use]
    fn inspect<F>(&mut self, inspector: F) -> Inspect<F>
    where
        F: FnMut(&FormatElement),
    {
        Inspect::new(self, inspector)
    }

    /// Starts a recording that gives you access to all elements that have been written between the start
    /// and end of the recording
    ///
    /// #Examples
    ///
    /// ```
    /// use std::ops::Deref;
    /// use biome_formatter::prelude::*;
    /// use biome_formatter::{write, format, SimpleFormatContext};
    ///
    /// # fn main() -> FormatResult<()> {
    /// let formatted = format!(SimpleFormatContext::default(), [format_with(|f| {
    ///     let mut recording = f.start_recording();
    ///
    ///     write!(recording, [text("A")])?;
    ///     write!(recording, [text("B")])?;
    ///
    ///     write!(recording, [format_with(|f| write!(f, [text("C"), text("D")]))])?;
    ///
    ///     let recorded = recording.stop();
    ///     assert_eq!(
    ///         recorded.deref(),
    ///         &[
    ///             FormatElement::StaticText{ text: "A" },
    ///             FormatElement::StaticText{ text: "B" },
    ///             FormatElement::StaticText{ text: "C" },
    ///             FormatElement::StaticText{ text: "D" }
    ///         ]
    ///     );
    ///
    ///     Ok(())
    /// })])?;
    ///
    /// assert_eq!(formatted.print()?.as_code(), "ABCD");
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    fn start_recording(&mut self) -> Recording<Self> {
        Recording::new(self)
    }

    /// Writes a sequence of elements into this buffer.
    fn write_elements<I>(&mut self, elements: I)
    where
        I: IntoIterator<Item = FormatElement>,
    {
        for element in elements {
            self.write_element(element);
        }
    }
}

impl<T> BufferExtensions for T where T: Buffer {}

#[derive(Debug)]
pub struct Recording<'buf, Buffer> {
    start: usize,
    buffer: &'buf mut Buffer,
}

impl<'buf, B> Recording<'buf, B>
where
    B: Buffer,
{
    fn new(buffer: &'buf mut B) -> Self {
        Self {
            start: buffer.elements().len(),
            buffer,
        }
    }

    #[inline(always)]
    pub fn write_fmt(&mut self, arguments: Arguments) {
        self.buffer.write_fmt(arguments);
    }

    #[inline(always)]
    pub fn write_element(&mut self, element: FormatElement) {
        self.buffer.write_element(element);
    }

    pub fn stop(self) -> Recorded<'buf> {
        let buffer: &'buf B = self.buffer;
        let elements = buffer.elements();

        let recorded = if self.start > elements.len() {
            // May happen if buffer was rewinded.
            &[]
        } else {
            &elements[self.start..]
        };

        Recorded(recorded)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Recorded<'a>(&'a [FormatElement]);

impl Deref for Recorded<'_> {
    type Target = [FormatElement];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
