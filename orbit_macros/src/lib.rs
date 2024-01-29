mod from_jsonb;
mod from_row;

use proc_macro::TokenStream;

#[proc_macro_derive(FromRow)]
pub fn from_row(input: TokenStream) -> TokenStream {
	from_row::expand(input)
}

#[proc_macro_derive(FromJsonb)]
pub fn from_jsonb(input: TokenStream) -> TokenStream {
	from_jsonb::expand(input)
}
