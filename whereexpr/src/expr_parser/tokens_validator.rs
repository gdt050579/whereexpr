    use super::{Token, TokenKind, TokenSpan};
    use crate::Error;

    const MAX_PAREN_DEPTH: usize = 8;

    pub(crate) fn validate_parentheses(tokens: &[Token], condition: &str) -> Result<(), Error> {
        let mut depth: usize = 0;
        let mut stack = [TokenSpan::default(); MAX_PAREN_DEPTH];

        for token in tokens {
            match token.kind() {
                TokenKind::LParen => {
                    if depth == MAX_PAREN_DEPTH {
                        return Err(Error::MaxParenDepthExceeded(token.span().start() as u16, token.span().end() as u16, condition.to_string()));
                    }
                    stack[depth] = token.span();
                    depth += 1;
                }
                TokenKind::RParen => {
                    if depth == 0 {
                        return Err(Error::UnexpectedCloseParen(token.span().start() as u16, token.span().end() as u16, condition.to_string()));
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }

        if depth > 0 {
            // report the innermost unclosed paren
            return Err(Error::UnclosedParenthesis(stack[depth - 1].start() as u16, stack[depth - 1].end() as u16, condition.to_string()));
        }

        Ok(())
    }

    pub(crate) fn resolve_rule_names(tokens: &mut [Token], input: &str, resolve: impl Fn(&str) -> Option<u16>) -> Result<(), Error> {
        for token in tokens.iter_mut() {
            if token.kind() == TokenKind::RuleName(u16::MAX) {
                let name = token.span().as_slice(input);
                match resolve(name) {
                    Some(idx) => *token = Token::new(TokenKind::RuleName(idx), token.span().start(), token.span().end()),
                    None => return Err(Error::UnknownRuleName(token.span().start() as u16, token.span().end() as u16, input.to_string())),
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_token_pairs(tokens: &[Token], condition: &str) -> Result<(), Error> {
        // check first token
        match tokens[0].kind() {
            TokenKind::RuleName(_) | TokenKind::Not | TokenKind::LParen => {}
            _ => return Err(Error::UnexpectedTokenAtStart(tokens[0].span().start() as u16, tokens[0].span().end() as u16, condition.to_string())),
        }

        // check last token
        match tokens[tokens.len() - 1].kind() {
            TokenKind::RuleName(_) | TokenKind::RParen => {}
            _ => return Err(Error::UnexpectedTokenAtEnd(tokens[tokens.len() - 1].span().start() as u16, tokens[tokens.len() - 1].span().end() as u16, condition.to_string())),
        }

        // check pairs
        for i in 0..tokens.len() - 1 {
            let current = tokens[i].kind();
            let next = tokens[i + 1].kind();

            match (current, next) {
                // after RuleName: only AND, OR, RParen allowed
                (TokenKind::RuleName(_), TokenKind::And) => {}
                (TokenKind::RuleName(_), TokenKind::Or) => {}
                (TokenKind::RuleName(_), TokenKind::RParen) => {}
                (TokenKind::RuleName(_), TokenKind::RuleName(_)) => {
                    return Err(Error::MissingOperator(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }
                (TokenKind::RuleName(_), TokenKind::LParen) => {
                    return Err(Error::MissingOperator(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }
                (TokenKind::RuleName(_), TokenKind::Not) => {
                    return Err(Error::MissingOperator(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }

                // after AND: only RuleName, NOT, LParen allowed
                (TokenKind::And, TokenKind::RuleName(_)) => {}
                (TokenKind::And, TokenKind::Not) => {}
                (TokenKind::And, TokenKind::LParen) => {}
                (TokenKind::And, TokenKind::And) | (TokenKind::And, TokenKind::Or) => {
                    return Err(Error::MissingOperand(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }
                (TokenKind::And, TokenKind::RParen) => {
                    return Err(Error::MissingOperand(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }

                // after OR: only RuleName, NOT, LParen allowed
                (TokenKind::Or, TokenKind::RuleName(_)) => {}
                (TokenKind::Or, TokenKind::Not) => {}
                (TokenKind::Or, TokenKind::LParen) => {}
                (TokenKind::Or, TokenKind::And) | (TokenKind::Or, TokenKind::Or) => {
                    return Err(Error::MissingOperand(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }
                (TokenKind::Or, TokenKind::RParen) => {
                    return Err(Error::MissingOperand(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }

                // after NOT: only RuleName, LParen allowed
                (TokenKind::Not, TokenKind::RuleName(_)) => {}
                (TokenKind::Not, TokenKind::LParen) => {}
                (TokenKind::Not, TokenKind::Not) => {
                    return Err(Error::DoubleNegation(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }
                (TokenKind::Not, TokenKind::And) | (TokenKind::Not, TokenKind::Or) => {
                    return Err(Error::NegationOfOperator(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }
                (TokenKind::Not, TokenKind::RParen) => {
                    return Err(Error::NegationOfCloseParen(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }

                // after LParen: only RuleName, NOT, LParen allowed
                (TokenKind::LParen, TokenKind::RuleName(_)) => {}
                (TokenKind::LParen, TokenKind::Not) => {}
                (TokenKind::LParen, TokenKind::LParen) => {}
                (TokenKind::LParen, TokenKind::RParen) => {
                    return Err(Error::EmptyParenthesis(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }
                (TokenKind::LParen, TokenKind::And) | (TokenKind::LParen, TokenKind::Or) => {
                    return Err(Error::OperatorAfterOpenParen(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }

                // after RParen: only AND, OR, RParen allowed
                (TokenKind::RParen, TokenKind::And) => {}
                (TokenKind::RParen, TokenKind::Or) => {}
                (TokenKind::RParen, TokenKind::RParen) => {}
                (TokenKind::RParen, TokenKind::RuleName(_)) => {
                    return Err(Error::MissingOperator(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }
                (TokenKind::RParen, TokenKind::LParen) => {
                    return Err(Error::MissingOperator(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }
                (TokenKind::RParen, TokenKind::Not) => {
                    return Err(Error::MissingOperator(tokens[i + 1].span().start() as u16, tokens[i + 1].span().end() as u16, condition.to_string()));
                }
            }
        }

        Ok(())
    }

    #[derive(Copy, Clone, Default)]
    struct NestedDepth {
        uses_and: bool,
        uses_or: bool,
    }

    pub(crate) fn validate_same_operation_per_level(tokens: &[Token], condition: &str) -> Result<(), Error> {
        let mut nested_level: [NestedDepth; MAX_PAREN_DEPTH + 2] = [NestedDepth::default(); MAX_PAREN_DEPTH + 2];
        let mut index = 1; // root is consdered level 1
        for tok in tokens {
            match tok.kind() {
                TokenKind::And => {
                    if nested_level[index].uses_or {
                        return Err(Error::MixedOperators(tok.span().start() as u16, tok.span().end() as u16, condition.to_string()));
                    }
                    nested_level[index].uses_and = true;
                }
                TokenKind::Or => {
                    if nested_level[index].uses_and {
                        return Err(Error::MixedOperators(tok.span().start() as u16, tok.span().end() as u16, condition.to_string()));
                    }                
                    nested_level[index].uses_or = true;
                }
                TokenKind::LParen => {
                    index += 1;
                    nested_level[index] = NestedDepth::default();
                }
                TokenKind::RParen => index -= 1,
                _ => {}
            }
        }
        Ok(())
    }

    pub(crate) fn validate(tokens: &mut [Token], input: &str, resolve: impl Fn(&str) -> Option<u16>) -> Result<(), Error> {
        validate_parentheses(tokens, input)?;
        resolve_rule_names(tokens, input,  resolve)?;
        validate_token_pairs(tokens, input)?;
        // must be called after the depth is being tested
        validate_same_operation_per_level(tokens, input)?;
        Ok(())
    }
