use crate::common::parse::{parse_valid_item, require_empty_args};
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{braced, Attribute, Error as SynError, ReturnType, Signature, Token, Visibility};

/// Implements the [`main`](crate::main) procedural macro.
pub fn main(attr: TokenStream, item: TokenStream) -> Result<TokenStream, TokenStream> {
    // Parse the attribute arguments (to make sure there are none)
    require_empty_args(attr, &item)?;

    // Parse the item, which is expected to be an `async fn main` function
    let parsed_fn = parse_valid_item(&item, validate_parsed_fn)?;

    // Compose the output
    let expanded = expand_parsed_fn(parsed_fn);

    Ok(TokenStream::from(expanded))
}

/// Validates the parsed annotated main function.
fn validate_parsed_fn(parsed_fn: &MainFunction) -> Result<(), SynError> {
    // Ensure the function is called `main`
    if parsed_fn.signature.ident != "main" {
        return Err(SynError::new_spanned(
            &parsed_fn.signature.ident,
            "this attribute is only allowed on the `main` function",
        ));
    }

    // Ensure the function has no defined arguments
    if !parsed_fn.signature.inputs.is_empty() {
        return Err(SynError::new_spanned(
            &parsed_fn.signature.inputs,
            "this attribute is only allowed on the `main` function without arguments",
        ));
    }

    // Ensure the function is `async`
    if parsed_fn.signature.asyncness.is_none() {
        return Err(SynError::new_spanned(
            &parsed_fn.signature.fn_token,
            "this attribute is only allowed on the `async main` function",
        ));
    }

    // Ensure the function doesn’t have a return type
    if !matches!(parsed_fn.signature.output, ReturnType::Default) {
        return Err(SynError::new_spanned(
            &parsed_fn.signature.output,
            "this attribute is only allowed on the `main` function that doesn’t declare a return type",
        ));
    }

    Ok(())
}

// Transforms the parsed main function into the final result.
fn expand_parsed_fn(mut parsed_fn: MainFunction) -> TokenStream {
    // Remove the `async` keyword
    parsed_fn.signature.asyncness = None;

    // Find the last statement
    let (_last_statement_start, last_statement_end) = parsed_fn.last_statement();

    // Prepare the body definition
    let body_identifier = quote! { body };
    let body_content = parsed_fn.body();

    // The body definition is the original body transformed to an async block and assigned to a variable
    let body_definition = quote! {
        let #body_identifier = async #body_content;
    };

    // The last block of code is simply the invocation of the Strut app against the async body
    let last_block = quote_spanned! { last_statement_end =>
        strut::App::boot(#body_identifier);
    };

    parsed_fn.into_tokens(body_definition, last_block)
}

/// Represent the original `async fn main`, as written by the caller.
struct MainFunction {
    outer_attributes: Vec<Attribute>,
    visibility: Visibility,
    signature: Signature,
    brace: Brace,
    inner_attributes: Vec<Attribute>,
    statements: Vec<TokenStream>,
}

impl MainFunction {
    /// Returns a pair of [`Span`]s representing the first and last token,
    /// respectively, of the very last statement of this main function’s body.
    fn last_statement(&self) -> (Span, Span) {
        let mut last_stmt = self
            .statements
            .last()
            .cloned()
            .unwrap_or_default()
            .into_iter();

        let start = last_stmt
            .next()
            .map_or_else(Span::call_site, |tree| tree.span());
        let end = last_stmt.last().map_or(start, |tree| tree.span());

        (start, end)
    }

    /// Returns the [`Body`] of this main function.
    fn body(&self) -> Body<'_> {
        Body {
            brace: self.brace,
            statements: &self.statements,
        }
    }

    /// Serializes this main function into the final [`TokenStream`] to be
    /// outputted from this macro.
    fn into_tokens(self, body_definition: TokenStream, last_block: TokenStream) -> TokenStream {
        // Prepare storage
        let mut tokens = TokenStream::new();

        // Add the outer attributes
        for outer_attribute in self.outer_attributes {
            outer_attribute.to_tokens(&mut tokens);
        }

        // Add the inner attributes
        for mut inner_attribute in self.inner_attributes {
            // Transform to outer attribute, since we’re adding them on the outside
            inner_attribute.style = syn::AttrStyle::Outer;

            inner_attribute.to_tokens(&mut tokens);
        }

        // Add the signature
        self.visibility.to_tokens(&mut tokens);
        self.signature.to_tokens(&mut tokens);

        // Finally, add the new main function body
        self.brace.surround(&mut tokens, |tokens| {
            body_definition.to_tokens(tokens);
            last_block.to_tokens(tokens);
        });

        tokens
    }
}

impl Parse for MainFunction {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // Parse the outer attributes
        let outer_attributes = input.call(Attribute::parse_outer)?;

        // Parse the signature
        let visibility = input.parse()?;
        let signature = input.parse()?;

        // Parse the body into a content buffer, shifting the braces away
        let content;
        let brace = braced!(content in input);

        // Parse the inner attributes
        let inner_attributes = Attribute::parse_inner(&content)?;

        // Prepare storage for the function body statements
        let mut buffer = TokenStream::new();
        let mut statements = Vec::new();

        // Parse the body statements one token tree at a time (without parsing the subtree)
        while !content.is_empty() {
            // Shift away any semicolon
            if let Some(semicolon) = content.parse::<Option<Token![;]>>()? {
                // Add the semicolon to the statements
                semicolon.to_tokens(&mut buffer);
                statements.push(buffer);

                // Reset the buffer
                buffer = TokenStream::new();

                continue;
            }

            // With the semicolon out of the way, take the next token tree
            buffer.extend([content.parse::<TokenTree>()?]);
        }

        // Flush the buffer, just in case
        if !buffer.is_empty() {
            statements.push(buffer);
        }

        Ok(Self {
            outer_attributes,
            visibility,
            signature,
            brace,
            inner_attributes,
            statements,
        })
    }
}

/// Represents the body of a function: the statements and the pair of braces
/// around them.
struct Body<'a> {
    brace: Brace,
    statements: &'a [TokenStream],
}

impl ToTokens for Body<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.brace.surround(tokens, |tokens| {
            for statement in self.statements {
                statement.to_tokens(tokens);
            }
        });
    }
}
