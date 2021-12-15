use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn ledbetter(_args: TokenStream, input: TokenStream) -> TokenStream {
	input
}
