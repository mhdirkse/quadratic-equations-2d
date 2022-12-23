#[cfg(test)]
mod test {
    use fail::fail_point;
    use crate::lang::ExprParser;
    use crate::expression::Expression;

    #[test]
    fn parse_add_expression() {
        parse_and_expect("5+3", "(5 + 3)");
    }

    #[test]
    fn parse_minus_expression() {
        parse_and_expect("5-3", "(5 - 3)");
    }

    #[test]
    fn parse_mult_expression() {
        parse_and_expect("5*3", "(5 * 3)");
    }

    #[test]
    fn parse_div_expression() {
        parse_and_expect("5/3", "(5 / 3)");
    }

    #[test]
    fn left_op_takes_precedence() {
        parse_and_expect("2*3+5", "((2 * 3) + 5)");
    }

    #[test]
    fn right_op_takes_precedence() {
        parse_and_expect("2+3*5", "(2 + (3 * 5))");
    }

    fn parse_and_expect(input: &str, expected: &str) {
        let r = ExprParser::new().parse(input);
        if r.is_ok() {
            let parsed: Expression = *r.ok().unwrap();
            assert_eq!(parsed.to_string(), expected);
        } else {
            fail_point!(r.err().to_string());
        }
    }
}