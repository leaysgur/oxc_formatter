#![expect(clippy::mutable_key_type)]
use crate::base_formatter::FormatElement;
use crate::base_formatter::format_element::tag::*;
use crate::base_formatter::format_element::*;
use rustc_hash::FxHashMap;
use std::ops::Deref;

/// A formatted document.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Document {
    elements: Vec<FormatElement>,
}

impl Document {
    /// Sets [`expand`](tag::Group::expand) to [`GroupMode::Propagated`] if the group contains any of:
    /// * a group with [`expand`](tag::Group::expand) set to [GroupMode::Propagated] or [GroupMode::Expand].
    /// * a non-soft [line break](FormatElement::Line) with mode [LineMode::Hard], [LineMode::Empty], or [LineMode::Literal].
    /// * a [FormatElement::ExpandParent]
    ///
    /// [`BestFitting`] elements act as expand boundaries, meaning that the fact that a
    /// [`BestFitting`]'s content expands is not propagated past the [`BestFitting`] element.
    ///
    /// [`BestFitting`]: FormatElement::BestFitting
    pub(crate) fn propagate_expand(&mut self) {
        #[derive(Debug)]
        enum Enclosing<'a> {
            Group(&'a tag::Group),
            BestFitting,
        }

        fn expand_parent(enclosing: &[Enclosing]) {
            if let Some(Enclosing::Group(group)) = enclosing.last() {
                group.propagate_expand();
            }
        }

        fn propagate_expands<'a>(
            elements: &'a [FormatElement],
            enclosing: &mut Vec<Enclosing<'a>>,
            checked_interned: &mut FxHashMap<&'a Interned, bool>,
        ) -> bool {
            let mut expands = false;
            for element in elements {
                let element_expands = match element {
                    FormatElement::Tag(Tag::StartGroup(group)) => {
                        enclosing.push(Enclosing::Group(group));
                        false
                    }
                    FormatElement::Tag(Tag::EndGroup) => match enclosing.pop() {
                        Some(Enclosing::Group(group)) => !group.mode().is_flat(),
                        _ => false,
                    },
                    FormatElement::Interned(interned) => match checked_interned.get(interned) {
                        Some(interned_expands) => *interned_expands,
                        None => {
                            let interned_expands =
                                propagate_expands(interned, enclosing, checked_interned);
                            checked_interned.insert(interned, interned_expands);
                            interned_expands
                        }
                    },
                    FormatElement::BestFitting(best_fitting) => {
                        enclosing.push(Enclosing::BestFitting);

                        for variant in best_fitting.variants() {
                            propagate_expands(variant, enclosing, checked_interned);
                        }

                        enclosing.pop();
                        // BestFitting acts as a boundary, meaning there is no need to continue
                        // processing this element and we can move onto the next. However, we
                        // _don't_ set `expands = false`, because that ends up negating the
                        // expansion when processing `Interned` elements.
                        //
                        // Only interned lists are affected, because they cache the expansion value
                        // based on the value of `expands` at the end of iterating the children. If
                        // a `best_fitting` element occurs after the last expanding element, and we
                        // end up setting `expands = false` here, then the interned element ends up
                        // thinking that its content doesn't expand, even though it might. Example:
                        //   group(1,
                        //     interned 1 [
                        //       expand_parent,
                        //       best_fitting,
                        //     ]
                        //   )
                        //   group(2,
                        //     [ref interned 1]
                        //   )
                        // Here, `group(1)` gets expanded directly by the `expand_parent` element.
                        // This happens immediately, and then `expands = true` is set. The interned
                        // element continues processing, and encounters the `best_fitting`. If
                        // we set `expands = false` there, then the interned element's result ends
                        // up being `false`, even though it does actually expand. Then, when
                        // `group(2)` checks for expansion, it looks at the ref to `interned 1`,
                        // which thinks it doesn't expand, and so `group(2)` stays flat.
                        //
                        // By _not_ setting `expands = false`, even though `best_fitting` is a
                        // boundary for expansion, we ensure that any references to the interned
                        // element will get the correct value for whether the contained content
                        // actually expands, regardless of the order of elements within it.
                        //
                        // Instead, just returning false here enforces that `best_fitting` doesn't
                        // think it expands _itself_, but allows other sibling elements to still
                        // propagate their expansion.
                        false
                    }
                    FormatElement::StaticText { text } => text.contains('\n'),
                    FormatElement::DynamicText { text, .. } => text.contains('\n'),
                    FormatElement::LocatedTokenText { slice, .. } => slice.contains('\n'),
                    FormatElement::ExpandParent
                    | FormatElement::Line(LineMode::Hard | LineMode::Empty) => true,
                    _ => false,
                };

                if element_expands {
                    expands = true;
                    expand_parent(enclosing)
                }
            }

            expands
        }

        let mut enclosing: Vec<Enclosing> = Vec::new();
        let mut interned = FxHashMap::default();
        propagate_expands(self, &mut enclosing, &mut interned);
    }
}

impl From<Vec<FormatElement>> for Document {
    fn from(elements: Vec<FormatElement>) -> Self {
        Self { elements }
    }
}

impl Deref for Document {
    type Target = [FormatElement];

    fn deref(&self) -> &Self::Target {
        self.elements.as_slice()
    }
}

impl std::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TODO: IrFormat")
    }
}

impl FormatElements for [FormatElement] {
    fn will_break(&self) -> bool {
        use Tag::*;
        let mut ignore_depth = 0usize;

        for element in self {
            match element {
                // Line suffix
                // Ignore if any of its content breaks
                FormatElement::Tag(StartLineSuffix) => {
                    ignore_depth += 1;
                }
                FormatElement::Tag(EndLineSuffix) => {
                    ignore_depth -= 1;
                }
                FormatElement::Interned(interned) if ignore_depth == 0 => {
                    if interned.will_break() {
                        return true;
                    }
                }

                element if ignore_depth == 0 && element.will_break() => {
                    return true;
                }
                _ => continue,
            }
        }

        debug_assert_eq!(ignore_depth, 0, "Unclosed start container");

        false
    }

    fn may_directly_break(&self) -> bool {
        use Tag::*;
        let mut ignore_depth = 0usize;

        for element in self {
            match element {
                // Line suffix
                // Ignore if any of its content breaks
                FormatElement::Tag(StartLineSuffix) => {
                    ignore_depth += 1;
                }
                FormatElement::Tag(EndLineSuffix) => {
                    ignore_depth -= 1;
                }
                FormatElement::Interned(interned) if ignore_depth == 0 => {
                    if interned.may_directly_break() {
                        return true;
                    }
                }

                element if ignore_depth == 0 && element.may_directly_break() => {
                    return true;
                }
                _ => continue,
            }
        }

        debug_assert_eq!(ignore_depth, 0, "Unclosed start container");

        false
    }

    fn has_label(&self, expected: LabelId) -> bool {
        self.first()
            .is_some_and(|element| element.has_label(expected))
    }

    fn start_tag(&self, kind: TagKind) -> Option<&Tag> {
        // Assert that the document ends at a tag with the specified kind;
        let _ = self.end_tag(kind)?;

        fn traverse_slice<'a>(
            slice: &'a [FormatElement],
            kind: TagKind,
            depth: &mut usize,
        ) -> Option<&'a Tag> {
            for element in slice.iter().rev() {
                match element {
                    FormatElement::Tag(tag) if tag.kind() == kind => {
                        if tag.is_start() {
                            if *depth == 0 {
                                // Invalid document
                                return None;
                            } else if *depth == 1 {
                                return Some(tag);
                            } else {
                                *depth -= 1;
                            }
                        } else {
                            *depth += 1;
                        }
                    }
                    FormatElement::Interned(interned) => {
                        match traverse_slice(interned, kind, depth) {
                            Some(start) => {
                                return Some(start);
                            }
                            // Reached end or invalid document
                            None if *depth == 0 => {
                                return None;
                            }
                            _ => {
                                // continue with other elements
                            }
                        }
                    }
                    _ => {}
                }
            }

            None
        }

        let mut depth = 0usize;

        traverse_slice(self, kind, &mut depth)
    }

    fn end_tag(&self, kind: TagKind) -> Option<&Tag> {
        self.last().and_then(|element| element.end_tag(kind))
    }
}
