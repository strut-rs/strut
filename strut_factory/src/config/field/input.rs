use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Or};
use syn::{Ident, Path};

pub struct DeserializeFieldInput {
    pub enum_name: Ident,
    pub key_eq_function: Path,
    pub variants: Vec<DeserializeFieldVariant>,
}

pub struct DeserializeFieldVariant {
    pub primary: Ident,
    pub aliases: Vec<Ident>,
}

impl Parse for DeserializeFieldInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let enum_name = input.parse()?;
        let _: Comma = input.parse()?;
        let key_eq_function = input.parse()?;
        let _: Comma = input.parse()?;

        let variants: Vec<_> =
            Punctuated::<DeserializeFieldVariant, Comma>::parse_terminated(input)?
                .into_iter()
                .collect();

        if variants.is_empty() {
            return Err(input.error("at least one variant is required"));
        }

        Ok(DeserializeFieldInput {
            enum_name,
            key_eq_function,
            variants,
        })
    }
}

impl Parse for DeserializeFieldVariant {
    fn parse(input: ParseStream) -> Result<Self> {
        let primary = input.parse()?;

        let mut aliases: Vec<Ident> = vec![];

        while input.peek(Or) {
            let _: Or = input.parse()?;
            let alias = input.parse()?;
            aliases.push(alias);
        }

        Ok(DeserializeFieldVariant {
            primary,
            aliases: aliases.into_iter().collect(),
        })
    }
}
