use super::{Token, TokenKind};

#[derive(Debug, PartialEq)]
pub(crate) enum ConditionNode {
    Rule(u16),
    And { children: Vec<ConditionNode>, negated: bool },
    Or  { children: Vec<ConditionNode>, negated: bool },
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos:    usize,
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
    fn parse_expr(&mut self) -> ConditionNode {
        let first = self.parse_term();

        if self.peek() != Some(TokenKind::Or) {
            return first;
        }

        let mut children = vec![first];
        while self.peek() == Some(TokenKind::Or) {
            self.consume(); // consume OR
            children.push(self.parse_term());
        }

        ConditionNode::Or { children, negated: false }
    }

    // term := factor (AND factor)*
    fn parse_term(&mut self) -> ConditionNode {
        let first = self.parse_factor();

        if self.peek() != Some(TokenKind::And) {
            return first;
        }

        let mut children = vec![first];
        while self.peek() == Some(TokenKind::And) {
            self.consume(); // consume AND
            children.push(self.parse_factor());
        }

        ConditionNode::And { children, negated: false }
    }

    // factor := NOT factor | '(' expr ')' | RuleName(idx)
    fn parse_factor(&mut self) -> ConditionNode {
        match self.peek() {
            Some(TokenKind::Not) => {
                self.consume(); // consume NOT
                let child = self.parse_factor();
                // wrap in a negated single-child node
                match child {
                    ConditionNode::And { children, .. } => {
                        ConditionNode::And { children, negated: true }
                    }
                    ConditionNode::Or { children, .. } => {
                        ConditionNode::Or { children, negated: true }
                    }
                    ConditionNode::Rule(idx) => {
                        ConditionNode::And { children: vec![ConditionNode::Rule(idx)], negated: true }
                    }
                }
            }
            Some(TokenKind::LParen) => {
                self.consume(); // consume (
                let node = self.parse_expr();
                self.consume(); // consume )
                node
            }
            Some(TokenKind::RuleName(idx)) => {
                self.consume();
                ConditionNode::Rule(idx)
            }
            _ => unreachable!("validator guarantees valid token stream"),
        }
    }
}

pub(crate) fn parse(tokens: &[Token]) -> ConditionNode {
    let mut parser = Parser::new(tokens);
    parser.parse_expr()
}