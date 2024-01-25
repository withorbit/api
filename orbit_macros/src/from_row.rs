use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Item};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
	let ast = syn::parse::<DeriveInput>(input).expect("Failed to parse token stream");

	let name = ast.ident;
	let Data::Struct(data) = ast.data else {
		unreachable!()
	};

	let fields = data.fields.iter().map(|field| {
		let ident = field.ident.as_ref().unwrap();
		let ty = &field.ty;
		let col_name = format!(r##"{}"##, ident);

		quote! { #ident: row.get::<&str, #ty>(#col_name) }
	});

	let tokens = quote! {
		impl From<tokio_postgres::Row> for #name {
			fn from(row: tokio_postgres::Row) -> Self {
				Self {
					#(#fields),*
				}
			}
		}
	};

	let tokens: Item = syn::parse_quote!(#tokens);
	let tokens = quote!(#tokens);

	tokens.into()
}
