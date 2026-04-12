use super::{Token, TokenKind};

fn find_flat_pair(tokens: &[Token], from: usize) -> Option<(usize, usize, usize)> {
    let mut open = None;
    for i in from..tokens.len() {
        match tokens[i].kind() {
            TokenKind::LParen => open = Some(i),
            TokenKind::RParen => {
                if let Some(start) = open {
                    // count consecutive ( going left from start
                    let left_count = tokens[..start].iter().rev().take_while(|t| t.kind() == TokenKind::LParen).count();

                    // count consecutive ) going right from i
                    let right_count = tokens[i + 1..].iter().take_while(|t| t.kind() == TokenKind::RParen).count();

                    let extra = left_count.min(right_count);
                    return Some((start, i, extra));
                }
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn reduce_single_rule_wrapping(tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
        if let Some((left, right, consecutive)) = find_flat_pair(tokens, i) {
            let content_len = right - left - 1;
            let is_single_rule = content_len == 1 && matches!(tokens[left + 1].kind(), TokenKind::ConditionIndex(_));
            let is_not_rule =
                content_len == 2 && tokens[left + 1].kind() == TokenKind::Not && matches!(tokens[left + 2].kind(), TokenKind::ConditionIndex(_));
            if is_single_rule || is_not_rule {
                tokens.drain(right..=right + consecutive);
                tokens.drain(left - consecutive..=left);
                i = left - consecutive;
            } else {
                // nothing to remove
                i = right + consecutive + 1;
            }
        } else {
            break;
        }
    }
}
/// Removes outermost wrapping parens when they span the entire token list.
/// ((( A AND (B OR C) ))) → A AND (B OR C)
pub(crate) fn reduce_outermost_wrapping(tokens: &mut Vec<Token>) {
    loop {
        let len = tokens.len();
        if len < 2 { return; }
        if tokens[0].kind() != TokenKind::LParen { return; }
        if tokens[len - 1].kind() != TokenKind::RParen { return; }

        // verify ( at 0 matches ) at len-1 by tracking depth
        let mut depth = 0usize;
        let mut spans_all = false;
        for i in 0..len {
            match tokens[i].kind() {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        spans_all = i == len - 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !spans_all { return; }

        tokens.remove(len - 1);
        tokens.remove(0);
    }
}

/// Removes extra wrapping pairs (((X))) where X contains no nested parens,
/// keeping the innermost pair intact.
/// (((A AND B))) → (A AND B)
/// NOT (((A AND B))) → NOT (A AND B)
pub(crate) fn reduce_extra_wrapping(tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
        if let Some((left, right, consecutive)) = find_flat_pair(tokens, i) {
            if consecutive > 0 {
                tokens.drain(right + 1..=right + consecutive); // right side first
                tokens.drain(left - consecutive..left); // then left side
                i = left - consecutive; // restart from outermost left
            } else {
                i = right + 1; // no extra wrapping, move past
            }
        } else {
            break;
        }
    }
}

pub(crate) fn reduce_parentheses(tokens: &mut Vec<Token>) {
    loop {
        let len = tokens.len();
        reduce_single_rule_wrapping(tokens); // (((rule))) and (((NOT rule))) → fully unwrapped
        reduce_outermost_wrapping(tokens); // (((A AND (B OR C)))) → A AND (B OR C)
        reduce_extra_wrapping(tokens); // (((A AND B))) → (A AND B)
        if tokens.len() == len {
            break;
        }
    }
}
