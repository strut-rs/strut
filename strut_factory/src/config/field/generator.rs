use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use super::input::{DeserializeFieldInput, DeserializeFieldVariant};

pub fn impl_deserialize_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeserializeFieldInput);
    let enum_name = &input.enum_name;
    let key_eq_function = &input.key_eq_function;
    let enum_visitor_name = format_ident!("{}Visitor", enum_name);

    let primary_idents: Vec<_> = input
        .variants
        .iter()
        .map(|variant| &variant.primary)
        .collect();
    let primary_names: Vec<_> = primary_idents.iter().map(|v| v.to_string()).collect();
    let from_str_arms = make_from_str_arms(&input.variants, key_eq_function);

    let expanded = quote! {
        #[allow(non_camel_case_types)]
        enum #enum_name {
            #(#primary_idents),*,
            __ignore,
        }

        impl #enum_name {
            /// Returns a field variant that matches the given user-provided string value. Applies
            /// the custom string matching method sequentially to all variants until finding a
            /// match. Returns the special `__ignore` variant if no matches are found.
            fn from_str(value: &str) -> Self {
                #(#from_str_arms)*
            }

            /// Simply represents the field variant as a string slice.
            fn as_str(&self) -> &'static str {
                match self {
                    #(Self::#primary_idents => #primary_names,)*
                    Self::__ignore => "__ignore",
                }
            }

            /// Polls the `next_value` from the given `MapAccess` reference and puts it into the
            /// given [`Option`]. If the [`Option`] is already [`Some`], returns an appropriate
            /// Serde error (duplicate field).
            fn poll<'de, A, T>(
                &self,
                from: &mut A,
                into: &mut Option<T>,
            ) -> Result<::serde::de::IgnoredAny, A::Error>
            where
                A: ::serde::de::MapAccess<'de>,
                T: ::serde::de::Deserialize<'de>,
            {
                if into.is_some() {
                    return Err(::serde::de::Error::duplicate_field(self.as_str()));
                }
                *into = Some(from.next_value()?);
                Ok(::serde::de::IgnoredAny)
            }

            /// Takes the value from the given [`Option`]. If the [`Option`] is [`None`], returns
            /// an appropriate Serde error (missing field).
            ///
            /// For optional values, use [`Option::unwrap_or_else`] or similar instead.
            fn take<T, E>(&self, from: Option<T>) -> Result<T, E>
            where
                E: ::serde::de::Error,
            {
                from.ok_or_else(|| ::serde::de::Error::missing_field(self.as_str()))
            }
        }

        impl<'de> ::serde::de::Deserialize<'de> for #enum_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::de::Deserializer<'de>,
            {
                deserializer.deserialize_identifier(#enum_visitor_name)
            }
        }

        struct #enum_visitor_name;

        impl ::serde::de::Visitor<'_> for #enum_visitor_name {
            type Value = #enum_name;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a configuration key")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                Ok(Self::Value::from_str(value))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                Ok(Self::Value::from_str(&value))
            }
        }
    };

    TokenStream::from(expanded)
}

fn make_from_str_arms(
    variants: &Vec<DeserializeFieldVariant>,
    key_eq_function: &syn::Path,
) -> Vec<TokenStream2> {
    let mut arms = Vec::new();

    // First arm
    if let Some(first_variant) = variants.first() {
        let mut arm = TokenStream2::new();

        let primary = &first_variant.primary;
        let primary_str = primary.to_string();

        arm.extend(quote! {
            if #key_eq_function(value, #primary_str)
        });

        for alias in first_variant.aliases.iter() {
            let alias_str = alias.to_string();

            arm.extend(quote! {
                || #key_eq_function(value, #alias_str)
            });
        }

        arm.extend(quote! {
            {
                Self::#primary
            }
        });

        arms.push(arm);
    }

    // Remaining arms
    for variant in variants.iter().skip(1) {
        let mut arm = TokenStream2::new();

        let primary = &variant.primary;
        let primary_str = primary.to_string();

        arm.extend(quote! {
            else if #key_eq_function(value, #primary_str)
        });

        for alias in variant.aliases.iter() {
            let alias_str = alias.to_string();

            arm.extend(quote! {
                || #key_eq_function(value, #alias_str)
            });
        }

        arm.extend(quote! {
            {
                Self::#primary
            }
        });

        arms.push(arm);
    }

    // Default arm
    arms.push(quote! { else { Self::__ignore } });

    arms
}
