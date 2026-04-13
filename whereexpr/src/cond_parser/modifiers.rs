use crate::Error;

pub(crate) struct Modifiers {
    pub(crate) ignore_case: bool,
}

impl Default for Modifiers {
    fn default() -> Self {
        Self { ignore_case: false }
    }
}

pub(crate) fn parse(text: &str) -> Result<(Modifiers, usize), Error> {
    let trimmed = text.trim_end();

    // no modifiers
    if !trimmed.ends_with('}') {
        return Ok((Modifiers::default(), text.len()));
    }

    // position of closing }
    let close_pos = trimmed.len() - 1;

    // find opening {
    let start = trimmed
        .rfind('{')
        .ok_or_else(|| Error::UnmatchedModifierBracket(close_pos as u16, (close_pos + 1) as u16, text.to_string()))?;

    // extract content between { and }
    let content = &trimmed[start + 1..close_pos];

    // empty modifier block e.g. {}
    if content.trim().is_empty() {
        return Err(Error::EmptyModifierBlock(start as u16, (close_pos+1) as u16, text.to_string()));
    }

    // parse modifiers tracking position of each one
    let mut modifiers = Modifiers::default();
    let content_start = start + 1;
    let mut offset = 0;

    for modifier in content.split(',') {
        // find position of trimmed modifier within this segment
        let leading_spaces = modifier.len() - modifier.trim_start().len();
        let trimmed_modifier = modifier.trim();
        let modifier_start = content_start + offset + leading_spaces;
        let modifier_end = modifier_start + trimmed_modifier.len();

        match trimmed_modifier {
            "ignore-case" => modifiers.ignore_case = true,
            _ => {
                return Err(Error::UnknownModifier(modifier_start as u16, modifier_end as u16, text.to_string()));
            }
        }

        offset += modifier.len() + 1; // +1 for the comma
    }

    Ok((modifiers, start))
}
