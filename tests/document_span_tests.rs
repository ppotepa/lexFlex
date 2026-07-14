use lexflex::document::{LineIndex, LocatedSpan, SourceSpan, SpanPrecision};

#[test]
fn utf8_spans_validate_and_slice_safely() {
    let text = "A🙂B";
    let span = SourceSpan::new(1, 5).unwrap();
    assert_eq!(span.slice(text).unwrap(), "🙂");
    assert!(SourceSpan::new(2, 4).unwrap().slice(text).is_err());
}

#[test]
fn line_index_handles_eof_and_line_spans() {
    let text = "A\r\nB\n🙂";
    let index = LineIndex::new(text);
    assert_eq!(index.line_count(), 3);
    assert_eq!(index.line_span(text, 1).unwrap(), SourceSpan { start: 0, end: 3 });
    assert_eq!(index.line_span(text, 2).unwrap(), SourceSpan { start: 3, end: 5 });
    assert_eq!(index.line_span(text, 3).unwrap(), SourceSpan { start: 5, end: text.len() });
    assert_eq!(index.line_column_at(text, 0).unwrap().line, 1);
    assert_eq!(index.line_column_at(text, 0).unwrap().column, 1);
    assert_eq!(index.line_column_at(text, 3).unwrap().line, 2);
    assert_eq!(index.line_column_at(text, 5).unwrap().line, 3);
    let single = LineIndex::new("A");
    assert_eq!(single.line_column_at("A", 1).unwrap().column, 2);
    assert_eq!(single.line_count(), 1);
    assert_eq!(single.line_span("A", 1).unwrap(), SourceSpan { start: 0, end: 1 });
    let trailing = LineIndex::new("A\n");
    assert_eq!(trailing.line_count(), 2);
    assert_eq!(trailing.line_span("A\n", 2).unwrap(), SourceSpan { start: 2, end: 2 });
    let empty = LineIndex::new("");
    assert_eq!(empty.line_count(), 1);
    assert_eq!(empty.line_span("", 1).unwrap(), SourceSpan { start: 0, end: 0 });
}

#[test]
fn located_spans_encode_precision() {
    let span = LocatedSpan::exact(SourceSpan::new(0, 1).unwrap());
    assert_eq!(span.precision, SpanPrecision::Exact);
    assert!(span.span.is_some());
    assert_eq!(LocatedSpan::unknown().precision, SpanPrecision::Unknown);
}

#[test]
fn try_union_respects_source_bounds() {
    let text = "A🙂B";
    let left = SourceSpan::new(0, 1).unwrap();
    let right = SourceSpan::new(1, 5).unwrap();
    assert_eq!(left.try_union(&right, text).unwrap(), SourceSpan { start: 0, end: 5 });
    assert!(SourceSpan::new(0, 2).unwrap().try_union(&right, text).is_err());
}
