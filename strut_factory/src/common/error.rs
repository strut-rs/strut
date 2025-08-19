use proc_macro2::TokenStream;
use syn::Error as SynError;

pub trait HelpRenderWithTokens {
    /// When called on a [`SynError`], generates a `compile_error!` invocation
    /// from it, then appends that invocation directly after the given `tokens`.
    ///
    /// It is intended that the given `tokens` represent the original annotated
    /// item (struct, enum, etc.). This way, the compilation error plays nicely
    /// with IDEs.
    fn render_with(self, tokens: TokenStream) -> TokenStream;
}

impl HelpRenderWithTokens for SynError {
    fn render_with(self, mut tokens: TokenStream) -> TokenStream {
        tokens.extend(self.into_compile_error());

        tokens
    }
}
