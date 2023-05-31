use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

#[proc_macro_derive(ToParams)]
pub fn to_params(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    let struct_name = input.ident;

    // Generate the implementation of ToRpcParams trait
    let expanded = quote! {
        impl jsonrpsee_core::traits::ToRpcParams for #struct_name {
            fn to_rpc_params(self) -> Result<Option<Box<serde_json::value::RawValue>>, jsonrpsee_core::Error> {
                let s = String::from_utf8(serde_json::to_vec(&self)?).expect("Valid UTF8 format");
                serde_json::value::RawValue::from_string(s).map(Some).map_err(jsonrpsee_core::Error::ParseError)
            }
        }
    };

    // Convert the generated code back into tokens
    TokenStream::from(expanded)
}
