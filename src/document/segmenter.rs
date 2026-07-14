use crate::core::interlingua::LanguageId;
use crate::document::builder::{DocumentBuildError, DocumentBuilder};
use crate::document::model::{Document, DocumentInput, DocumentSegmentationOptions};
use crate::document::reconstruction::DocumentReconstructor;
use crate::document::span::{LocatedSpan, SourceSpan};
use crate::document::validation::DocumentStructureError;

pub trait DocumentSegmenter {
    fn segment(
        &self,
        input: DocumentInput,
        options: &DocumentSegmentationOptions,
    ) -> Result<Document, DocumentSegmentationError>;
}

#[derive(Debug, Clone, Default)]
pub struct LosslessParagraphSegmenter;

#[derive(Debug, thiserror::Error)]
pub enum DocumentSegmentationError {
    #[error("unsupported lossless segmentation option: {0}")]
    UnsupportedOption(String),
    #[error("document construction failed: {0}")]
    Build(#[from] DocumentBuildError),
    #[error("sentence segmentation failed: {0}")]
    Sentence(#[from] crate::document::sentence_segmenter::SentenceSegmentationError),
    #[error("segmented document failed structural validation")]
    InvalidStructure { errors: Vec<DocumentStructureError> },
    #[error("segmented document could not be reconstructed exactly")]
    ReconstructionMismatch,
    #[error("internal segmentation invariant failed: {0}")]
    InternalInvariant(String),
}

impl LosslessParagraphSegmenter {
    pub fn segment_text(
        &self,
        source: impl Into<String>,
        language: LanguageId,
    ) -> Result<Document, DocumentSegmentationError> {
        self.segment(
            DocumentInput {
                id: None,
                source_language: language,
                source: source.into(),
            },
            &DocumentSegmentationOptions::default(),
        )
    }
}

impl DocumentSegmenter for LosslessParagraphSegmenter {
    fn segment(
        &self,
        input: DocumentInput,
        options: &DocumentSegmentationOptions,
    ) -> Result<Document, DocumentSegmentationError> {
        validate_options(options)?;
        let mut builder = DocumentBuilder::new(input);
        let lines = scan_physical_lines(builder.source());
        let regions = build_regions(builder.source(), &lines)?;
        for region in regions {
            let located = LocatedSpan::exact(region.span);
            match region.kind {
                SegmentRegionKind::Paragraph => {
                    builder.push_paragraph(located)?;
                }
                SegmentRegionKind::PreservedWhitespace => {
                    builder.push_preserved_whitespace(located)?;
                }
                SegmentRegionKind::PreservedRaw => {
                    builder.push_preserved_raw(located)?;
                }
            }
        }
        let document = builder
            .finish()
            .map_err(|errors| DocumentSegmentationError::InvalidStructure { errors })?;
        DocumentReconstructor::verify_lossless(&document)
            .map_err(|_| DocumentSegmentationError::ReconstructionMismatch)?;
        Ok(document)
    }
}

fn validate_options(
    options: &DocumentSegmentationOptions,
) -> Result<(), DocumentSegmentationError> {
    if options.trim_document_edges {
        return Err(DocumentSegmentationError::UnsupportedOption(
            "trim_document_edges".into(),
        ));
    }
    if !options.preserve_empty_blocks {
        return Err(DocumentSegmentationError::UnsupportedOption(
            "preserve_empty_blocks=false".into(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhysicalLine {
    full_span: SourceSpan,
    body_span: SourceSpan,
    terminator_span: Option<SourceSpan>,
    is_blank: bool,
}

fn scan_physical_lines(source: &str) -> Vec<PhysicalLine> {
    let bytes = source.as_bytes();
    let mut lines = Vec::new();
    let mut line_start = 0;
    let mut index = 0;
    while index < bytes.len() {
        let terminator_end = match bytes[index] {
            b'\r' if index + 1 < bytes.len() && bytes[index + 1] == b'\n' => {
                Some(index + 2)
            }
            b'\r' | b'\n' => Some(index + 1),
            _ => None,
        };
        let Some(end) = terminator_end else {
            index += 1;
            continue;
        };
        lines.push(make_line(source, line_start, index, Some((index, end))));
        line_start = end;
        index = end;
    }
    if line_start < source.len() {
        lines.push(make_line(
            source,
            line_start,
            source.len(),
            None,
        ));
    }
    lines
}

fn make_line(
    source: &str,
    line_start: usize,
    body_end: usize,
    terminator: Option<(usize, usize)>,
) -> PhysicalLine {
    let full_end = terminator.map_or(body_end, |(_, end)| end);
    let body_span = SourceSpan {
        start: line_start,
        end: body_end,
    };
    debug_assert!(body_span.validate_for(source).is_ok());
    PhysicalLine {
        full_span: SourceSpan {
            start: line_start,
            end: full_end,
        },
        body_span,
        terminator_span: terminator.map(|(start, end)| SourceSpan { start, end }),
        is_blank: source[line_start..body_end]
            .chars()
            .all(char::is_whitespace),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentRegionKind {
    Paragraph,
    PreservedWhitespace,
    PreservedRaw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SegmentRegion {
    kind: SegmentRegionKind,
    span: SourceSpan,
}

fn build_regions(
    source: &str,
    lines: &[PhysicalLine],
) -> Result<Vec<SegmentRegion>, DocumentSegmentationError> {
    let mut paragraph_spans = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if lines[index].is_blank {
            index += 1;
            continue;
        }
        let start = lines[index].full_span.start;
        let mut end = lines[index].body_span.end;
        index += 1;
        while index < lines.len() && !lines[index].is_blank {
            end = lines[index].body_span.end;
            index += 1;
        }
        paragraph_spans.push(SourceSpan { start, end });
    }

    let mut regions = Vec::new();
    let mut cursor = 0;
    for paragraph in paragraph_spans {
        push_gap_region(source, cursor, paragraph.start, &mut regions);
        regions.push(SegmentRegion {
            kind: SegmentRegionKind::Paragraph,
            span: paragraph,
        });
        cursor = paragraph.end;
    }
    push_gap_region(source, cursor, source.len(), &mut regions);
    merge_adjacent_preserved_regions(&mut regions);
    validate_region_partition(source.len(), &regions)?;
    Ok(regions)
}

fn push_gap_region(
    source: &str,
    start: usize,
    end: usize,
    regions: &mut Vec<SegmentRegion>,
) {
    if start == end {
        return;
    }
    let kind = if source[start..end].chars().all(char::is_whitespace) {
        SegmentRegionKind::PreservedWhitespace
    } else {
        SegmentRegionKind::PreservedRaw
    };
    regions.push(SegmentRegion {
        kind,
        span: SourceSpan { start, end },
    });
}

fn merge_adjacent_preserved_regions(regions: &mut Vec<SegmentRegion>) {
    let mut merged: Vec<SegmentRegion> = Vec::with_capacity(regions.len());
    for region in regions.drain(..) {
        if region.kind != SegmentRegionKind::Paragraph {
            if let Some(previous) = merged.last_mut() {
                if previous.kind == region.kind && previous.span.end == region.span.start {
                    previous.span.end = region.span.end;
                    continue;
                }
            }
        }
        merged.push(region);
    }
    *regions = merged;
}

fn validate_region_partition(
    source_len: usize,
    regions: &[SegmentRegion],
) -> Result<(), DocumentSegmentationError> {
    if source_len == 0 {
        return if regions.is_empty() {
            Ok(())
        } else {
            Err(DocumentSegmentationError::InternalInvariant(
                "empty source produced regions".into(),
            ))
        };
    }
    let mut cursor = 0;
    for region in regions {
        if region.span.is_empty() || region.span.start != cursor {
            return Err(DocumentSegmentationError::InternalInvariant(format!(
                "region {:?} does not continue at byte {cursor}",
                region.span
            )));
        }
        cursor = region.span.end;
    }
    if cursor != source_len {
        return Err(DocumentSegmentationError::InternalInvariant(format!(
            "regions end at byte {cursor}, source ends at {source_len}"
        )));
    }
    Ok(())
}
