use crate::parser::chart_stage::chart_stage;
use crate::parser::context::ParseContext;
use crate::parser::finish_stage::finish_stage;
use crate::parser::lexical_stage::lexical_stage;

pub fn parse_with_ambiguity(
    mut ctx: ParseContext<'_>,
) -> Result<crate::output::ParseOutput, crate::diagnostic::ParseError> {
    let candidates = lexical_stage(&mut ctx)?;
    let chart = chart_stage(&mut ctx, &candidates)?;
    finish_stage(ctx, chart)
}
