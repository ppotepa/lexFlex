use lexflex_parser::ParseScore;

#[test]
fn parse_score_orders_by_priority_then_complexity() {
    let better = ParseScore::lexical(1);
    let worse = ParseScore::lexical(2);
    assert!(better < worse);
}
