use whereexpr::{AttributeIndex, Attributes, Condition, ExpressionBuilder, Value, ValueKind, Expression};


struct TestCondition {
    left: bool,
    right: bool,
}

impl TestCondition {
    const LEFT: AttributeIndex = AttributeIndex::new(0);
    const RIGHT: AttributeIndex = AttributeIndex::new(1);

    fn new(left: bool, right: bool) -> Self {
        Self { left, right }
    }
}


impl Attributes for TestCondition {
    const TYPE_ID: u64 = 0x1e123456;
    const TYPE_NAME: &'static str = "TestCondition";

    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        match idx {
            Self::LEFT => {
                println!("evaluating LEFT, value: {}", self.left);
                Some(Value::Bool(self.left))
            }
            Self::RIGHT => {
                println!("evaluating RIGHT, value: {}", self.right);
                Some(Value::Bool(self.right))
            }
            _ => None,
        }
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::LEFT => Some(ValueKind::Bool),
            Self::RIGHT => Some(ValueKind::Bool),
            _ => None,
        }
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "left" => Some(Self::LEFT),
            "right" => Some(Self::RIGHT),
            _ => None,
        }
    }
}


fn build_expression(expression: &str) -> Expression {
    ExpressionBuilder::<TestCondition>::new()
        .add("left", Condition::from_str("left is true"))
        .add("right", Condition::from_str("right is true"))
        .build(expression)
        .expect("Failed to build expression")
}


fn main() {
    println!("Testing: false && true");
    let expr_1 = build_expression("left && right");
    let cond_1 = TestCondition::new(false, true);
    let result_1 = expr_1.matches(&cond_1);
    println!("Result: {}", result_1);

    println!();

    println!("Testing: true || false");
    let expr_2 = build_expression("left || right");
    let cond_2 = TestCondition::new(true, false);
    let result_2 = expr_2.matches(&cond_2);
    println!("Result: {}", result_2);

    println!();

    println!("Testing: true && false");
    let expr_3 = build_expression("left && right");
    let cond_3 = TestCondition::new(true, false);
    let result_3 = expr_3.matches(&cond_3);
    println!("Result: {}", result_3);

    println!();

    println!("Testing: false || true");
    let expr_4 = build_expression("left || right");
    let cond_4 = TestCondition::new(false, true);
    let result_4 = expr_4.matches(&cond_4);
    println!("Result: {}", result_4);
}