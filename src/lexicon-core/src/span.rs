use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    /// Create a span with default location (line 1, column 1).
    /// Complexity: O(1).
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Self::with_location(start, end, 1, 1)
    }

    /// Create a span with explicit line/column.
    /// Complexity: O(1).
    #[inline]
    pub fn with_location(start: usize, end: usize, line: usize, column: usize) -> Self {
        Span {
            start,
            end,
            line,
            column,
        }
    }

    /// Empty span at offset 0.
    /// Complexity: O(1).
    #[inline]
    pub fn empty() -> Self {
        Span::new(0, 0)
    }

    /// Whether this span covers no bytes (`start >= end`).
    /// Complexity: O(1).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Byte length (`end.saturating_sub(start)`).
    /// Complexity: O(1).
    #[inline]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Whether `pos` lies inside (`start <= pos < end`, half-open).
    ///
    /// Empty spans contain nothing.
    /// Complexity: O(1).
    #[inline]
    pub fn contains(&self, pos: usize) -> bool {
        self.start <= pos && pos < self.end
    }

    /// Whether `other` lies entirely inside `self`.
    ///
    /// Empty `other` is contained when its position is within
    /// `[self.start, self.end]`; non-empty `other` requires
    /// `self.start <= other.start && other.end <= self.end`.
    /// Complexity: O(1).
    #[inline]
    pub fn contains_span(&self, other: Span) -> bool {
        if other.is_empty() {
            self.start <= other.start && other.start <= self.end
        } else {
            self.start <= other.start && other.end <= self.end
        }
    }

    /// Whether two spans overlap (share at least one byte).
    ///
    /// Half-open interval check; empty spans never overlap.
    /// Complexity: O(1).
    #[inline]
    pub fn overlaps(&self, other: Span) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Smallest span covering both (`min(start)`, `max(end)`).
    /// Keeps `self` line/column.
    /// Complexity: O(1).
    #[inline]
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line,
            column: self.column,
        }
    }

    /// Alias for [`Span::merge`].
    /// Complexity: O(1).
    #[inline]
    pub fn to(self, other: Span) -> Span {
        self.merge(other)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Attach a span to a value.
    /// Complexity: O(1).
    #[inline]
    pub fn new(value: T, span: Span) -> Self {
        Spanned { value, span }
    }

    /// Transform the inner value, preserving the span.
    /// Complexity: O(1) plus `f`.
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            value: f(self.value),
            span: self.span,
        }
    }

    /// Borrowed view (`Spanned<&T>`) without cloning.
    /// Complexity: O(1).
    #[inline]
    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            value: &self.value,
            span: self.span,
        }
    }

    /// Mutable borrowed view (`Spanned<&mut T>`) without cloning.
    /// Complexity: O(1).
    #[inline]
    pub fn as_mut(&mut self) -> Spanned<&mut T> {
        Spanned {
            value: &mut self.value,
            span: self.span,
        }
    }

    /// Borrowed inner value.
    /// Complexity: O(1).
    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Copy of the span (spans are `Copy`).
    /// Complexity: O(1).
    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    /// Decompose into `(value, span)`.
    /// Complexity: O(1).
    #[inline]
    pub fn into_inner(self) -> (T, Span) {
        (self.value, self.span)
    }
}

#[cfg(test)]
mod span_tests {
    use super::*;

    #[test]
    fn new_and_with_location() {
        let s = Span::new(2, 5);
        assert_eq!(s.start, 2);
        assert_eq!(s.end, 5);
        assert_eq!((s.line, s.column), (1, 1));
        let l = Span::with_location(0, 3, 4, 7);
        assert_eq!((l.line, l.column), (4, 7));
    }

    #[test]
    fn empty_and_is_empty_and_len() {
        let e = Span::empty();
        assert!(e.is_empty());
        assert_eq!(e.len(), 0);
        assert_eq!(Span::new(3, 3).len(), 0);
        assert!(Span::new(3, 3).is_empty());
        assert!(Span::new(5, 3).is_empty(), "inverted span is empty");
        let s = Span::new(2, 5);
        assert!(!s.is_empty());
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn merge_and_to() {
        let a = Span::with_location(2, 5, 1, 3);
        let b = Span::with_location(4, 9, 2, 1);
        let m = a.merge(b);
        assert_eq!((m.start, m.end), (2, 9));
        assert_eq!((m.line, m.column), (1, 3), "merge keeps self location");
        assert_eq!(a.to(b), m);
        // Merging with empty still covers.
        assert_eq!(Span::new(2, 5).merge(Span::empty()), Span::new(0, 5));
    }

    #[test]
    fn contains_offset() {
        let s = Span::new(2, 5);
        assert!(!s.contains(1));
        assert!(s.contains(2));
        assert!(s.contains(4));
        assert!(!s.contains(5), "half-open: end is exclusive");
        assert!(!s.contains(9));
        assert!(!Span::empty().contains(0));
    }

    #[test]
    fn contains_span() {
        let outer = Span::new(0, 10);
        assert!(outer.contains_span(Span::new(2, 5)));
        assert!(outer.contains_span(Span::new(0, 10)));
        assert!(!outer.contains_span(Span::new(5, 15)));
        assert!(!Span::new(2, 5).contains_span(outer));
        // Empty inner at boundary is contained.
        assert!(outer.contains_span(Span::new(10, 10)));
        assert!(!outer.contains_span(Span::new(11, 11)));
    }

    #[test]
    fn overlaps() {
        assert!(Span::new(0, 5).overlaps(Span::new(4, 9)));
        assert!(Span::new(4, 9).overlaps(Span::new(0, 5)));
        assert!(!Span::new(0, 5).overlaps(Span::new(5, 9)), "touching is not overlap");
        assert!(!Span::new(0, 5).overlaps(Span::new(6, 9)));
        assert!(!Span::empty().overlaps(Span::new(0, 5)));
        assert!(!Span::new(0, 5).overlaps(Span::empty()));
    }

    #[test]
    fn display_format() {
        let s = Span::with_location(0, 4, 3, 8);
        assert_eq!(format!("{}", s), "line 3, column 8");
    }

    #[test]
    fn spanned_new_map_as_ref() {
        let sp = Spanned::new(41, Span::new(0, 2));
        assert_eq!(*sp.value(), 41);
        assert_eq!(sp.span(), Span::new(0, 2));
        let mapped = sp.map(|v| v + 1);
        assert_eq!(mapped.value, 42);
        assert_eq!(mapped.span, Span::new(0, 2));
        let borrowed = mapped.as_ref();
        assert_eq!(*borrowed.value, 42);
        assert_eq!(borrowed.span, Span::new(0, 2));
    }

    #[test]
    fn spanned_as_mut_and_into_inner() {
        let mut sp = Spanned::new(vec![1], Span::new(1, 4));
        {
            let view = sp.as_mut();
            view.value.push(2);
            assert_eq!(view.span, Span::new(1, 4));
        }
        assert_eq!(sp.value, vec![1, 2]);
        let (v, s) = sp.into_inner();
        assert_eq!(v, vec![1, 2]);
        assert_eq!(s, Span::new(1, 4));
    }
}
