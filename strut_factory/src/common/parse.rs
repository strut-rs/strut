use crate::common::error::HelpRenderWithTokens;
use proc_macro2::TokenStream;
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::{parse2, Error as SynError, Meta, Token};

/// A shorthand for a sequence of comma-delimited attribute arguments
pub type Args = Punctuated<Meta, Token![,]>;

/// Parses the given `attr` token stream and parses it as a
/// [list of arguments](Args) to an attribute macro invocation (e.g.,
/// `with, some, args` in `#[custom_attribute(with, some, args)]`).
///
/// Takes a reference to the annotated `item` in order to gracefully render
/// errors. Errors can be expected if parsing fails, or from the given
/// `validator` function. In case of an error, this function returns the
/// original `item`, [rendered along][render_with] with an invocation of
/// `compile_error!`.
///
/// [render_with]: HelpRenderWithTokens::render_with
pub fn parse_valid_args(
    attr: TokenStream,
    item: &TokenStream,
    validator: impl Fn(&Args) -> Result<(), SynError>,
) -> Result<Args, TokenStream> {
    let parsed_args = match Args::parse_terminated.parse2(attr) {
        Ok(parsed_args) => parsed_args,
        Err(error) => return Err(error.render_with(item.clone())),
    };

    match validator(&parsed_args) {
        Ok(_) => Ok(parsed_args),
        Err(error) => Err(error.render_with(item.clone())),
    }
}

/// Calls [`parse_valid_args`] with a validator that checks that [`Args`] are
/// empty.
pub fn require_empty_args(attr: TokenStream, item: &TokenStream) -> Result<(), TokenStream> {
    parse_valid_args(attr, item, validate_empty_args).map(|_| ())
}

/// Calls [`parse_valid_args`] with a no-op validator.
pub fn parse_args(attr: TokenStream, item: &TokenStream) -> Result<Args, TokenStream> {
    parse_valid_args(attr, item, |_| Ok(()))
}

/// Simply ensures that the given [`Args`] are empty.
fn validate_empty_args(args: &Args) -> Result<(), SynError> {
    if !args.is_empty() {
        return Err(SynError::new_spanned(
            args,
            "this attribute takes no arguments",
        ));
    }

    Ok(())
}

/// Parses the given `item` token stream and parses it as an item annotated with
/// an attribute macro (e.g., `#[custom_attribute]`).
///
/// Errors can be expected if parsing fails, or from the given `validator`
/// function. In case of an error, this function returns the original `item`,
/// [rendered along][render_with] with an invocation of `compile_error!`.
///
/// [render_with]: HelpRenderWithTokens::render_with
pub fn parse_valid_item<T>(
    item: &TokenStream,
    validator: impl Fn(&T) -> Result<(), SynError>,
) -> Result<T, TokenStream>
where
    T: Parse,
{
    let parsed_item = match parse2(item.clone()) {
        Ok(parsed_item) => parsed_item,
        Err(error) => return Err(error.render_with(item.clone())),
    };

    match validator(&parsed_item) {
        Ok(_) => Ok(parsed_item),
        Err(error) => Err(error.render_with(item.clone())),
    }
}

/// Calls [`parse_valid_item`] with a no-op validator.
pub fn parse_item<T>(item: &TokenStream) -> Result<T, TokenStream>
where
    T: Parse,
{
    parse_valid_item(item, |_| Ok(()))
}
