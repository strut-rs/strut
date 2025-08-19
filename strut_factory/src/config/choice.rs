use crate::common::parse::Args;
use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::collections::BTreeSet;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, DeriveInput, Error as SynError, Expr, ExprLit, ExprPath, Fields, Lit, Meta,
    MetaNameValue, Path, Result as SynResult, Token, Variant as SynVariant,
};

/// The recognized variant-level attribute name.
const ATTR_NAME: &str = "strut";

/// The recognized argument keys.
const ARG_NAME_EQ_FN: &str = "eq_fn";
const ARG_NAME_ALIAS: &str = "alias";

/// Generates an `impl<'de> Deserialize<'de>` block for the given `item`, which
/// must be a unit-only enum.
pub(crate) fn config_choice(input: DeriveInput) -> SynResult<TokenStream> {
    let Mapping {
        name,
        visitor_name,
        eq_fn,
        all_aliases,
        variants,
    } = require_supported_enum(&input)?;

    let visit_str_body = compose_visit_str_body(&name, variants, eq_fn, all_aliases);

    let expanded = quote! {
        const _: () = {
            impl<'de> ::serde::de::Deserialize<'de> for #name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::serde::de::Deserializer<'de>,
                {
                    deserializer.deserialize_str(#visitor_name)
                }
            }

            struct #visitor_name;

            impl<'de> ::serde::de::Visitor<'de> for #visitor_name {
                type Value = #name;

                fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    formatter.write_str("a string value")
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: ::serde::de::Error,
                {
                    #visit_str_body
                }
            }
        };
    };

    Ok(expanded)
}

/// Composes the body of the `visit_str` method inside the implementation of
/// the `serde::de::Visitor` trait on the annotated type.
fn compose_visit_str_body(
    name: &Ident,
    variants: Vec<Variant>,
    eq_fn: Option<Path>,
    all_aliases: BTreeSet<String>,
) -> TokenStream {
    let arms = compose_visit_str_arms(name, variants, eq_fn);

    if arms.is_empty() {
        return quote! {
            Err(serde::de::Error::unknown_variant(value, &[#(#all_aliases),*]))
        };
    }

    quote! {
        #(
            #arms
        )else*
        else {
            Err(serde::de::Error::unknown_variant(value, &[#(#all_aliases),*]))
        }
    }
}

/// Composes a collection of if-else arms for the `visit_str` method.
fn compose_visit_str_arms(
    name: &Ident,
    variants: Vec<Variant>,
    eq_fn: Option<Path>,
) -> Vec<TokenStream> {
    let mut arms = Vec::new();

    for variant in variants {
        if variant.aliases.is_empty() {
            continue;
        }

        let variant_name = &variant.name;
        let variant_aliases = &variant.aliases;

        let arm = if let Some(ref eq_fn) = eq_fn {
            quote! {
                if #(#eq_fn(value, #variant_aliases))||* {
                    Ok(#name::#variant_name)
                }
            }
        } else {
            quote! {
                if #(value.eq_ignore_ascii_case(#variant_aliases))||* {
                    Ok(#name::#variant_name)
                }
            }
        };

        arms.push(arm);
    }

    arms
}

/// Ensures that the input is a non-generic Rust enum that contains at least one
/// variant, with all variants being unit variants.
fn require_supported_enum(input: &DeriveInput) -> SynResult<Mapping> {
    // Must be an enum
    let data_enum = match input.data {
        syn::Data::Enum(ref data_enum) => data_enum,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "this macro only supports enums",
            ));
        }
    };

    // No generics
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            input.generics.clone(),
            "this macro only supports non-generic types",
        ));
    }

    // At least one variant
    if data_enum.variants.is_empty() {
        return Err(syn::Error::new_spanned(
            input,
            "this macro only supports non-empty enums",
        ));
    }

    // All variants must be unit variants
    for variant in &data_enum.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(SynError::new_spanned(
                variant,
                "this macro only supports unit variants",
            ));
        }
    }

    // Flag unsupported arguments in helper attribute on enum level
    let unsupported_args = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident(ATTR_NAME))
        .filter_map(|attr| attr.parse_args_with(Args::parse_terminated).ok())
        .flat_map(|attrs| attrs.into_iter())
        .filter(|arg| !arg.path().is_ident(ARG_NAME_EQ_FN));
    for arg in unsupported_args {
        return Err(SynError::new_spanned(arg, "this argument is not supported"));
    }

    // Flag unsupported arguments in helper attribute on variant level
    let unsupported_args = data_enum
        .variants
        .iter()
        .flat_map(|variant| variant.attrs.iter())
        .filter(|attr| attr.path().is_ident(ATTR_NAME))
        .filter_map(|attr| attr.parse_args_with(Args::parse_terminated).ok())
        .flat_map(|attrs| attrs.into_iter())
        .filter(|arg| !arg.path().is_ident(ARG_NAME_ALIAS));
    for arg in unsupported_args {
        return Err(SynError::new_spanned(arg, "this argument is not supported"));
    }

    Ok(extract_mapping(
        &input.ident,
        &input.attrs,
        &data_enum.variants,
    ))
}

/// Parses the given [`item_enum`](ItemEnum) into the
/// [internal representation](Mapping).
fn extract_mapping(
    name: &Ident,
    attrs: &[Attribute],
    variants: &Punctuated<SynVariant, Token![,]>,
) -> Mapping {
    // Extract enum name
    let name = name.clone();

    // Look for comparison function
    let eq_fn = extract_eq_fn(attrs);

    // Process variants
    let variants = variants.iter().map(extract_variant).collect();

    Mapping::new(name, eq_fn, variants)
}

/// Searches for the attribute named `ATTR_NAME` and extracts the value of a
/// name-value attribute with the key `eq_fn`. Given the attribute
/// `#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]`, will return
/// `Some` of `strut_deserialize::Slug::eq_as_slugs`.
///
/// If any of the expected elements are not found, returns `None`.
fn extract_eq_fn(attrs: &[Attribute]) -> Option<Path> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident(ATTR_NAME))
        .filter_map(|attr| attr.parse_args_with(Args::parse_terminated).ok())
        .flat_map(|attrs| attrs.into_iter())
        .filter(|arg| arg.path().is_ident(ARG_NAME_EQ_FN))
        .filter_map(extract_path)
        .last()
}

/// Digs **deep** into the given [`Meta`], and if it happens to be a
/// [name-value](Meta::NameValue) variant with a [`Path`] value (e.g.,
/// `key = some::path`) — extracts and returns that path value.
fn extract_path(arg: Meta) -> Option<Path> {
    match arg {
        Meta::NameValue(MetaNameValue {
            value: Expr::Path(ExprPath { path, .. }),
            ..
        }) => Some(path),
        _ => None,
    }
}

/// Parses the given [`variant`](SynVariant) into the
/// [internal representation](Variant).
fn extract_variant(variant: &SynVariant) -> Variant {
    // Extract variant name
    let variant_name = variant.ident.clone();

    // Prepare storage for aliases
    let mut aliases = BTreeSet::new();

    // Primary alias: snake_case version of the variant's name
    let primary_alias = variant_name.to_string().to_case(Case::Snake);
    aliases.insert(primary_alias);

    /*
    A lot to unpack here:

    - Iterates over all variant-level attributes.
    - Picks out those with expected name `ATTR_NAME`.
    - Digs into the attribute’s arguments.
    - Picks out those with expected key `ARG_NAME_ALIAS`.
    - Picks out those that are `name = value`.
    - Takes the arguments’ value if it is a string literal.
     */
    let additional_aliases = variant
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident(ATTR_NAME))
        .filter_map(|attr| attr.parse_args_with(Args::parse_terminated).ok())
        .flat_map(|attrs| attrs.into_iter())
        .filter(|arg| arg.path().is_ident(ARG_NAME_ALIAS))
        .filter_map(extract_string_literal);

    // Push additional aliases
    for alias in additional_aliases {
        aliases.insert(alias.to_case(Case::Snake));
    }

    Variant {
        name: variant_name,
        aliases,
    }
}

/// Digs **deep** into the given [`Meta`], and if it happens to be a
/// [name-value](Meta::NameValue) variant with a string literal value (e.g.,
/// `key = "value"`) — extracts and returns that string value without the
/// quotes (in this example the returned string would be `value`).
fn extract_string_literal(arg: Meta) -> Option<String> {
    match arg {
        Meta::NameValue(MetaNameValue {
            value:
                Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }),
            ..
        }) => Some(lit_str.value()),
        _ => None,
    }
}

/// Encodes the annotated enum with the pre-resolved bits and pieces needed for
/// the implementation.
struct Mapping {
    name: Ident,
    visitor_name: Ident,
    eq_fn: Option<Path>,
    all_aliases: BTreeSet<String>,
    variants: Vec<Variant>,
}

impl Mapping {
    fn new(name: Ident, eq_fn: Option<Path>, variants: Vec<Variant>) -> Self {
        // Compose the visitor name
        let visitor_name = Ident::new(&format!("{}Visitor", name), name.span());

        // Flatten the aliases
        let all_aliases = variants
            .iter()
            .flat_map(|v| v.aliases.iter())
            .map(|s| s.to_string())
            .collect();

        Self {
            name,
            visitor_name,
            eq_fn,
            all_aliases,
            variants,
        }
    }
}

/// Encodes a variant of the annotated enum with the pre-resolved bits and
/// pieces needed for the implementation.
struct Variant {
    name: Ident,
    aliases: BTreeSet<String>,
}
