pub enum Expression {
    Number(String),
    Composite(Composite)
}

pub enum Operator {
    PLUS,
    MINUS,
    MULT,
    DIV
}

pub struct Composite {
    pub operator: Operator,
    pub left: Box<Expression>,
    pub right: Box<Expression>
}

impl Composite {
    pub fn new(left: Box<Expression>, operator: Operator, right: Box<Expression>) -> Composite {
        return Composite{operator, left, right};
    }
}

impl ToString for Operator {
    fn to_string(&self) -> String {
        match self {
            Operator::PLUS => String::from("+"),
            Operator::MINUS => String::from("-"),
            Operator::MULT => String::from("*"),
            Operator::DIV => String::from("/")
        }
    }
}

impl ToString for Expression {
    fn to_string(&self) -> String {
        match self {
            Expression::Number(s) => s.to_owned(),
            Expression::Composite(c) => {
                let mut result: String = "(".to_owned();
                result.push_str(&c.left.to_string());
                result.push_str(" ");
                result.push_str(&c.operator.to_string());
                result.push_str(" ");
                result.push_str(&c.right.to_string());
                result.push_str(")");
                result
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::expression::Expression;
    use crate::expression::Composite;
    use crate::expression::Operator;

    #[test]
    fn expression_to_string() {
        let e = Expression::Composite(Composite {
            operator: Operator::PLUS,
            left: Box::new(Expression::Number(String::from("5"))),
            right: Box::new(Expression::Number(String::from("3")))
        });
        assert_eq!(e.to_string(), "(5 + 3)");
    }
}