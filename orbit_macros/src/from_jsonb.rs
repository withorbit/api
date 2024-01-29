use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
	let data = syn::parse_macro_input!(input as ItemStruct);
	let ident = data.ident;

	let tokens = quote! {
		impl tokio_postgres::types::FromSql<'_> for #ident {
			fn from_sql(
				_: &tokio_postgres::types::Type,
				raw: &'_ [u8]
			) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
				Ok(serde_json::from_slice(&raw[1..])?)
			}

			fn accepts(ty: &tokio_postgres::types::Type) -> bool {
				ty == &tokio_postgres::types::Type::JSONB
			}
		}
	};

	tokens.into()
}
