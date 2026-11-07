//! Turns the parsed enum into an `impl whereexpr::WhereexprEnum for ...` block.

use crate::error::{Error, Result};
use crate::meta::parse_group;
use crate::parser::{parse_enum_input, EnumInput};
use proc_macro::{Span, TokenStream};

/// One variant, with the id it gets and the names that map to it.
struct EnumVariant {
    /// The variant identifier, as written (`Red`, or `r#Type`).
    ident: String,
    /// Position in the declaration: 0, 1, 2, ...
    id: u16,
    /// Names accepted by `as_u16_from_str`, already in lower case (ASCII).
    names: Vec<String>,
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// FNV-1a over the bytes of `name`, lowered to ASCII lower case. The generated code computes
/// the same hash at run time, so both sides must lower case the same way.
fn fnv64(name: &str) -> u64 {
    name.bytes()
        .fold(FNV_OFFSET, |hash, b| (hash ^ b.to_ascii_lowercase() as u64).wrapping_mul(FNV_PRIME))
}

/// The enum itself takes no options (they go on the variants).
fn check_enum_options(input: &EnumInput) -> Result<()> {
    for group in &input.attrs {
        if let Some(meta) = parse_group(group)?.first() {
            return Err(Error::new(
                meta.span,
                format!("unknown option `{}`; `#[whereexpr(...)]` options go on the variants (`aliases`)", meta.key),
            ));
        }
    }
    Ok(())
}

/// Collects the ids and names of every variant, rejecting a name used twice.
fn collect_variants(input: &EnumInput) -> Result<Vec<EnumVariant>> {
    if input.variants.len() > u16::MAX as usize + 1 {
        return Err(Error::new(input.name.span(), "too many variants (the maximum is 65536)"));
    }
    let mut variants = Vec::with_capacity(input.variants.len());
    // every name seen so far, with the variant that uses it
    let mut seen: Vec<(String, String)> = Vec::new();
    for (id, variant) in input.variants.iter().enumerate() {
        let ident = variant.name.to_string();
        let plain = ident.strip_prefix("r#").unwrap_or(&ident).to_string();
        let mut aliases: Option<Vec<(String, Span)>> = None;
        for group in &variant.attrs {
            for meta in parse_group(group)? {
                match meta.key.as_str() {
                    "aliases" => {
                        let list = meta.as_str_list()?;
                        if list.is_empty() {
                            return Err(Error::new(meta.value_span, "`aliases` needs at least one name"));
                        }
                        aliases.get_or_insert_with(Vec::new).extend(list.iter().cloned());
                    }
                    _ => {
                        return Err(Error::new(
                            meta.span,
                            format!("unknown option `{}`; expected `aliases`", meta.key),
                        ))
                    }
                }
            }
        }
        // aliases replace the default name (the variant name in lower case)
        let names = aliases.unwrap_or_else(|| vec![(plain.clone(), variant.name.span())]);
        let mut lowered = Vec::with_capacity(names.len());
        for (name, span) in names {
            if name.is_empty() {
                return Err(Error::new(span, "an alias cannot be empty"));
            }
            let name = name.to_ascii_lowercase();
            if let Some((_, owner)) = seen.iter().find(|(n, _)| *n == name) {
                let message = if *owner == plain {
                    format!("`{name}` is listed twice for variant `{plain}` (names ignore case)")
                } else {
                    format!("`{name}` is already used by variant `{owner}` (names ignore case)")
                };
                return Err(Error::new(span, message));
            }
            seen.push((name.clone(), plain.clone()));
            lowered.push(name);
        }
        variants.push(EnumVariant {
            ident,
            id: id as u16,
            names: lowered,
        });
    }
    Ok(variants)
}

/// Body of `as_u16_from_str(s: &str) -> Option<u16>` for all the variants of the enum.
///
/// Every name and alias (lower case) is hashed here, with FNV-1a, and becomes one arm of a
/// `match` on the hash of `s`, which the generated code computes over `s` lowered to ASCII
/// lower case. An arm then compares `s` with the name (ignoring ASCII case), so a different
/// string that happens to have the same hash is not accepted; names whose hashes collide
/// share an arm.
fn generate_as_u16_from_str(variants: &[EnumVariant]) -> String {
    // (hash, name, id), sorted by hash so that colliding names end up next to each other
    let mut entries: Vec<(u64, &str, u16)> = variants
        .iter()
        .flat_map(|v| v.names.iter().map(move |name| (fnv64(name), name.as_str(), v.id)))
        .collect();
    entries.sort();

    let mut arms = String::new();
    let mut i = 0;
    while i < entries.len() {
        let hash = entries[i].0;
        arms.push_str(&format!("0x{hash:016x}u64 => "));
        while i < entries.len() && entries[i].0 == hash {
            let (_, _, id) = entries[i];
            arms.push_str(&format!("Some({id}u16),\n"));
            i += 1;
        }
    }

    format!(
        "let bytes = <str>::as_bytes(s);
         let mut hash: u64 = 0x{FNV_OFFSET:016x}u64;
         let mut i = 0;
         while i < bytes.len() {{
             hash = (hash ^ (bytes[i].to_ascii_lowercase() as u64)).wrapping_mul(0x{FNV_PRIME:016x}u64);
             i += 1;
         }}
         match hash {{
             {arms}
             _ => None,
         }}"
    )
}

fn generate_as_u16(variants: &[EnumVariant]) -> String {
    let arms: String = variants.iter().map(|v| format!("Self::{} => {}u16,", v.ident, v.id)).collect();
    format!("match *self {{ {arms} }}")
}

pub(crate) fn derive(input: TokenStream) -> Result<TokenStream> {
    let input = parse_enum_input(input)?;
    check_enum_options(&input)?;
    let variants = collect_variants(&input)?;

    let code = format!(
        "#[automatically_derived]
         impl {impl_generics} WhereexprEnum for {name} {ty_generics} {where_clause} {{
             #[inline]
             fn as_u16(&self) -> u16 {{
                 {as_u16}
             }}
             fn as_u16_from_str(s: &str) -> Option<u16> {{
                 {from_str}
             }}
         }}",
        impl_generics = input.impl_generics,
        name = input.name,
        ty_generics = input.ty_generics,
        where_clause = input.where_clause,
        as_u16 = generate_as_u16(&variants),
        from_str = generate_as_u16_from_str(&variants),
    );
    Ok(code.parse().expect("whereexpr_proc_macro generated invalid code"))
}
