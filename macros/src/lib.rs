use proc_macro::TokenStream;
use proc_macro2::{Span, Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Attribute, Error, Field, ItemImpl, ItemMod, ItemStruct, Meta, NestedMeta, Type, visit_mut::VisitMut, Visibility};

#[proc_macro_attribute]
pub fn ledbetter(args: TokenStream, input: TokenStream) -> TokenStream {
	if !args.is_empty() {
		let error = syn::Error::new(
			Span::call_site(),
			"outermost #[ledbetter] attribute cannot have arguments"
		);
		return error.to_compile_error().into();
	}

	let mut input = parse_macro_input!(input as ItemMod);
	let mut visitor = LedbetterVisitor::new(input.ident.clone());
	visitor.visit_item_mod_mut(&mut input);
	let mut tokens = input.to_token_stream();
	tokens.extend(visitor.to_tokens());
	tokens.into()
}

fn check_param_visibility(field: &Field) -> Result<(), Error> {
	match field.vis {
		Visibility::Public(_) => {}
		_ => {
			let err_msg = "ledbetter params must have pub visibility";
			return Err(Error::new_spanned(field, err_msg));
		}
	}
	Ok(())
}

fn check_param_type(ty: &Type) -> Result<(), Error> {
	let is_ok = match ty {
		Type::Path(path) => match path.path.get_ident() {
			Some(ident) =>
				ident == "u32" || ident == "u64" ||
					ident == "i32" || ident == "i64" ||
					ident == "f32" || ident == "f64",
			_ => false,
		},
		_ => false,
	};
	if !is_ok {
		let err_msg = "ledbetter params type must be one of: u32, u64, i32, i64, f32, f64";
		return Err(Error::new_spanned(ty, err_msg));
	}
	Ok(())
}

struct LedbetterParam {
	name: Ident,
	ty: Type,
}

struct LedbetterVisitor {
	mod_ident: Ident,
	params: Option<Vec<LedbetterParam>>,
	animation: Option<Type>,
	errors: Vec<Error>,
}

impl LedbetterVisitor {
	pub fn new(mod_ident: Ident) -> Self {
		LedbetterVisitor {
			mod_ident,
			params: None,
			animation: None,
			errors: Vec::new(),
		}
	}

	pub fn to_tokens(self) -> TokenStream2 {
		let mut tokens = TokenStream2::new();
		let LedbetterVisitor { mod_ident, params, animation, mut errors } = self;
		let global_ident = format_ident!("ANIMATION");

		if let Some(animation_ty) = animation {
			tokens.extend(quote! {
				static mut #global_ident: ledbetter::PixelAnimationGlobal<#mod_ident::#animation_ty> = ledbetter::PixelAnimationGlobal(None);

				#[export_name = "initLayoutSetNumStrips"]
				pub extern "C" fn init_layout_set_num_strips(n_strips: u32) {
					unsafe { #global_ident.init_layout_set_num_strips(n_strips as usize) };
				}

				#[export_name = "initLayoutSetStripLen"]
				pub extern "C" fn init_layout_set_strip_len(strip_idx: u32, length: u32) {
					unsafe { #global_ident.init_layout_set_strip_len(strip_idx as usize, length as usize) };
				}

				#[export_name = "initLayoutSetPixelLoc"]
				pub extern "C" fn init_layout_set_pixel_loc(strip_idx: u32, pixel_idx: u32, x: f32, y: f32) {
					unsafe { #global_ident.init_layout_set_pixel_loc(strip_idx as usize, pixel_idx as usize, x, y) };
				}

				#[export_name = "initLayoutDone"]
				pub extern "C" fn init_layout_done() {
					unsafe { #global_ident.init_layout_done() };
				}

				#[export_name = "tick"]
				pub extern "C" fn tick() {
					unsafe { #global_ident.tick() };
				}

				#[export_name = "getPixelVal"]
				pub extern "C" fn get_pixel_val(strip_idx: u32, pixel_idx: u32) -> u32 {
					unsafe { #global_ident.pixels(strip_idx as usize)[pixel_idx as usize] }
				}
			});
		} else {
			let msg = "No animation impl annotated"; // TODO
			errors.push(Error::new(Span::call_site(), msg));
		}

		if let Some(params) = params {
			for LedbetterParam { name, ty } in params {
				let param_ty = ty.into_token_stream();
				let getter_name = format_ident!("get_param_{}", name);
				let setter_name = format_ident!("set_param_{}", name);
				tokens.extend(quote! {
					#[no_mangle]
					pub extern "C" fn #getter_name() -> #param_ty {
						unsafe { #global_ident.params_mut() }.#name
					}

					#[no_mangle]
					pub extern "C" fn #setter_name(val: #param_ty) {
						unsafe { #global_ident.params_mut() }.#name = val;
					}
				});
			}
		}

		for err in errors {
			tokens.extend(err.to_compile_error());
		}
		tokens
	}

	fn visit_item_struct(&mut self, struct_item: &ItemStruct, attr: Attribute)
		-> Result<(), Error>
	{
		if !is_ledbetter_tag(attr.parse_meta()?, "params") {
			let msg = "expected #[ledbetter(params)] on struct item";
			return Err(Error::new_spanned(attr, msg));
		}
		if self.params.is_some() {
			let msg = "cannot have multiple #[ledbetter(params)] structs in one module";
			return Err(Error::new_spanned(attr, msg));
		}

		let params = struct_item.fields.iter()
			.map(|field| {
				let name: Ident = field.ident.clone().ok_or_else(|| {
					let msg = "ledbetter params field must be named";
					Error::new_spanned(field, msg)
				})?;
				check_param_type(&field.ty)?;
				check_param_visibility(field)?;
				Ok(LedbetterParam {
					name,
					ty: field.ty.clone(),
				})
			})
			.collect::<Result<Vec<_>, Error>>()?;
		self.params = Some(params);
		Ok(())
	}

	fn visit_item_impl(&mut self, impl_item: &ItemImpl, attr: Attribute)
		-> Result<(), Error>
	{
		if !is_ledbetter_tag(attr.parse_meta()?, "animation") {
			let msg = "expected #[ledbetter(animation)] on impl item";
			return Err(Error::new_spanned(attr, msg));
		}
		if self.animation.is_some() {
			let msg = "cannot have multiple #[ledbetter(animation)] impls in one module";
			return Err(Error::new_spanned(attr, msg));
		}

		if impl_item.trait_.is_none() {
			let msg = "#[ledbetter(animation)] must be on a PixelAnimation trait impl";
			return Err(Error::new_spanned(attr, msg));
		}
		self.animation = Some((*impl_item.self_ty).clone());

		Ok(())
	}
}

impl VisitMut for LedbetterVisitor {
	fn visit_item_impl_mut(&mut self, impl_item: &mut ItemImpl) {
		let ledbetter_attr_idx = impl_item.attrs.iter()
			.position(|attr| {
				attr.path.get_ident().map(|ident| ident == "ledbetter").unwrap_or(false)
			});
		if let Some(idx) = ledbetter_attr_idx {
			let attr = impl_item.attrs.remove(idx);
			if let Err(err) = self.visit_item_impl(impl_item, attr) {
				self.errors.push(err);
			}
		}
	}

	fn visit_item_struct_mut(&mut self, struct_item: &mut ItemStruct) {
		let ledbetter_attr_idx = struct_item.attrs.iter()
			.position(|attr| {
				attr.path.get_ident().map(|ident| ident == "ledbetter").unwrap_or(false)
			});
		if let Some(idx) = ledbetter_attr_idx {
			let attr = struct_item.attrs.remove(idx);
			if let Err(err) = self.visit_item_struct(struct_item, attr) {
				self.errors.push(err);
			}
		}
	}
}

fn is_ledbetter_tag(meta: Meta, arg: &str) -> bool {
	match meta {
		Meta::List(lst) => {
			if !lst.path.get_ident().map(|ident| ident == "ledbetter").unwrap_or(false) {
				return false;
			}
			if lst.nested.len() != 1 {
				return false;
			}
			match &lst.nested[0] {
				NestedMeta::Meta(Meta::Path(path)) =>
					path.get_ident().map(|ident| ident == arg).unwrap_or(false),
				_ => false,
			}
		}
		_ => false,
	}
}