use super::{Token, TokenKind};
use crate::expression::Composition;
use crate::expression::EvaluationNode;

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<TokenKind> {
        self.tokens.get(self.pos).map(|t| t.kind())
    }

    fn consume(&mut self) -> TokenKind {
        let kind = self.tokens[self.pos].kind();
        self.pos += 1;
        kind
    }

    // expr := term (OR term)*
    fn parse_expr(&mut self) -> EvaluationNode {
        let first = self.parse_term();

        if self.peek() != Some(TokenKind::Or) {
            return first;
        }

        let mut children = vec![first];
        while self.peek() == Some(TokenKind::Or) {
            self.consume(); // consume OR
            children.push(self.parse_term());
        }

        EvaluationNode::Group {
            composition: Composition::Or,
            children,
        }
    }

    // term := factor (AND factor)*
    fn parse_term(&mut self) -> EvaluationNode {
        let first = self.parse_factor();

        if self.peek() != Some(TokenKind::And) {
            return first;
        }

        let mut children = vec![first];
        while self.peek() == Some(TokenKind::And) {
            self.consume(); // consume AND
            children.push(self.parse_factor());
        }

        EvaluationNode::Group {
            composition: Composition::And,
            children,
        }
    }

    // factor := NOT factor | '(' expr ')' | ConditionIndex(idx)
    fn parse_factor(&mut self) -> EvaluationNode {
        match self.peek() {
            Some(TokenKind::Not) => {
                self.consume(); // consume NOT
                let child = self.parse_factor();
                EvaluationNode::Not {
                    child: Box::new(child),
                }
            }
            Some(TokenKind::LParen) => {
                self.consume(); // consume (
                let node = self.parse_expr();
                self.consume(); // consume )
                node
            }
            Some(TokenKind::ConditionIndex(idx)) => {
                self.consume();
                EvaluationNode::Condition(idx)
            }
            _ => unreachable!("validator guarantees valid token stream"),
        }
    }
}

pub(crate) fn parse(tokens: &[Token]) -> EvaluationNode {
    let mut parser = Parser::new(tokens);
    parser.parse_expr()
}