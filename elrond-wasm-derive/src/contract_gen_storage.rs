use super::arg_def::*;
use super::contract_gen_method::*;
use super::util::*;

fn generate_with_key_snippet(
	key_args: &[MethodArg],
	identifier: String,
	action: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
	let id_literal = array_literal(identifier.as_bytes());
	if key_args.is_empty() {
		// hardcode key
		quote! {
			let key: &'static [u8] = &#id_literal;
			#action
		}
	} else {
		let key_arg_names: Vec<syn::Pat> = key_args.iter().map(|arg| arg.pat.clone()).collect();
		quote! {
			elrond_wasm::with_storage_key(
				self.api.clone(),
				#id_literal.into(),
				( #(#key_arg_names),* ),
				|key| { #action },
			)
		}
	}
}

pub fn generate_getter_impl(m: &Method, identifier: String) -> proc_macro2::TokenStream {
	let msig = m.generate_sig();
	// let key_snippet = generate_key_snippet(&m.method_args.as_slice(), identifier);
	match m.return_type.clone() {
		syn::ReturnType::Default => panic!("getter should return some value"),
		_ => {
			let load_snippet = quote! {
				elrond_wasm::storage_get(self.api.clone(), key)
			};
			let with_key_snippet =
				generate_with_key_snippet(&m.method_args.as_slice(), identifier, &load_snippet);
			quote! {
				#msig {
					#with_key_snippet
				}
			}
		},
	}
}

pub fn generate_setter_impl(m: &Method, identifier: String) -> proc_macro2::TokenStream {
	let msig = m.generate_sig();
	if m.method_args.is_empty() {
		panic!("setter must have at least one argument, for the value");
	}
	if m.return_type != syn::ReturnType::Default {
		panic!("setter should not return anything");
	}
	let key_args = &m.method_args[..m.method_args.len() - 1];
	let value_arg = &m.method_args[m.method_args.len() - 1];
	let value_arg_name = value_arg.pat.clone();
	let store_snippet = quote! {
		elrond_wasm::storage_set(self.api.clone(), key, & #value_arg_name);
	};
	let with_key_snippet = generate_with_key_snippet(key_args, identifier, &store_snippet);
	quote! {
		#msig {
			#with_key_snippet
		}
	}
}

pub fn generate_is_empty_impl(m: &Method, identifier: String) -> proc_macro2::TokenStream {
	let msig = m.generate_sig();
	let is_empty_snippet = quote! {
		self.api.storage_load_len(&key[..]) == 0
	};
	let with_key_snippet =
		generate_with_key_snippet(&m.method_args.as_slice(), identifier, &is_empty_snippet);
	quote! {
		#msig {
			#with_key_snippet
		}
	}
}

/// Still using it for BorrowedMutStorage because it provides owned key objects
fn generate_key_snippet_old(
	key_args: &[MethodArg],
	identifier: String,
) -> proc_macro2::TokenStream {
	let id_literal = array_literal(identifier.as_bytes());
	if key_args.is_empty() {
		// hardcode key
		quote! {
			let key: &'static [u8] = &#id_literal;
		}
	} else {
		// build key from arguments
		let key_appends: Vec<proc_macro2::TokenStream> = key_args
			.iter()
			.map(|arg| {
				let arg_pat = &arg.pat;
				quote! {
					if let Result::Err(encode_error) = #arg_pat.dep_encode(&mut key) {
						self.api.signal_error(encode_error.message_bytes());
					}
				}
			})
			.collect();
		quote! {
			let mut key: Vec<u8> = #id_literal.to_vec();
			#(#key_appends)*
		}
	}
}

pub fn generate_borrow_impl(m: &Method, identifier: String) -> proc_macro2::TokenStream {
	let msig = m.generate_sig();
	let key_snippet = generate_key_snippet_old(&m.method_args.as_slice(), identifier);
	if m.method_args.is_empty() {
		// const key
		quote! {
			#msig {
				#key_snippet
				BorrowedMutStorage::with_const_key(self.api.clone(), key)
			}
		}
	} else {
		// generated key
		quote! {
			#msig {
				#key_snippet
				BorrowedMutStorage::with_generated_key(self.api.clone(), key)
			}
		}
	}
}
