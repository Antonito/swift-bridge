use std::any::Any;
use crate::bridged_type::{BridgeableType, BridgedType, TypePosition};
use crate::TypeDeclarations;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::Path;
use syn::spanned::Spanned;

/// Rust: Result<T, E>
/// Swift: RustResult<T, E>
///
/// We don't use Swift's `Result` type since when we tried we saw a strange error
///  `'Sendable' class 'ResultTestOpaqueRustType' cannot inherit from another class other than 'NSObject'`
///  which meant that we could not use the `public class ResultTestOpaqueRustType: ResultTestOpaqueRustTypeRefMut {`
///  pattern that we use to prevent calling mutable methods on immutable references.
///  We only saw this error after `extension: ResultTestOpaqueRustType: Error {}` .. which was
///  necessary because Swift's Result type requires that the error implements the `Error` protocol.
#[derive(Debug)]
pub(crate) struct BuiltInResult {
    pub ok_ty: Box<dyn BridgeableType>,
    pub err_ty: Box<dyn BridgeableType>,
}

impl BuiltInResult {
    pub(super) fn to_ffi_compatible_rust_type(&self, swift_bridge_path: &Path) -> TokenStream {
        let ok = self.ok_ty.to_ffi_compatible_rust_type(swift_bridge_path);
        let err = self.err_ty.to_ffi_compatible_rust_type(swift_bridge_path);

        println!("### ok: {}, err: {}", ok.to_string(), err.to_string());
        let type_name = syn::Ident::new(
            &format!("Result{}{}", "u8", "u8"),
            swift_bridge_path.span()
        );

        let wanted = "";

        // let wanted = quote! {
        //     #swift_bridge_path::result::#type_name
        // };

        //  types are primitives.
        //  See `swift-bridge/src/std_bridge/result`
        let result_kind = quote! {
            ResultPtrAndPtr
        };

        let s = quote! {
            #swift_bridge_path::result::#result_kind
        };

        println!("--> {} | vs | {}", s.to_string(), wanted.to_string());

        s
    }

    pub(super) fn convert_ffi_value_to_rust_value(
        &self,
        expression: &TokenStream,
        span: Span,
        swift_bridge_path: &Path,
        types: &TypeDeclarations,
    ) -> TokenStream {
        let convert_ok = self.ok_ty.convert_ffi_result_ok_value_to_rust_value(
            expression,
            swift_bridge_path,
            types,
        );

        let convert_err = self.err_ty.convert_ffi_result_err_value_to_rust_value(
            expression,
            swift_bridge_path,
            types,
        );

        quote_spanned! {span=>
            if #expression.is_ok {
                std::result::Result::Ok(#convert_ok)
            } else {
                std::result::Result::Err(#convert_err)
            }
        }
    }

    pub fn to_rust_type_path(&self) -> TokenStream {
        let ok = self.ok_ty.to_rust_type_path();
        let err = self.err_ty.to_rust_type_path();

        quote! { Result<#ok, #err> }
    }

    pub fn to_swift_type(&self, type_pos: TypePosition, types: &TypeDeclarations) -> String {
        format!(
            "RustResult<{}, {}>",
            self.ok_ty.to_swift_type(type_pos, types),
            self.err_ty.to_swift_type(type_pos, types),
        )
    }

    pub fn convert_swift_expression_to_ffi_compatible(
        &self,
        expression: &str,
        type_pos: TypePosition,
    ) -> String {
        let convert_ok = self
            .ok_ty
            .convert_swift_expression_to_ffi_type("ok", type_pos);
        let convert_err = self
            .err_ty
            .convert_swift_expression_to_ffi_type("err", type_pos);

        let type_name = format!("__private__Result{}And{}", self.ok_ty.to_c_type(), self.err_ty.to_c_type());

        format!(
            "{{ switch {val} {{ case .Ok(let ok): return {type_name}(is_ok: true, ok_or_err: {convert_ok}) case .Err(let err): return {type_name}(is_ok: false, ok_or_err: {convert_err}) }} }}()",
            val = expression
        )
    }

    pub(super) fn convert_rust_expression_to_ffi_type(
        &self,
        expression: &TokenStream,
        swift_bridge_path: &Path,
    ) -> TokenStream {
        let path = self.to_rust_type_path();

        let ok = self.ok_ty.to_rust_type_path();
        let err = self.err_ty.to_rust_type_path();

        let type_name = syn::Ident::new(
            &format!("Result{}{}", quote!(#ok), quote!(#err)),
            path.span()
        );

        let s = quote! {
            match #expression {
                Ok(val) => #swift_bridge_path::result::<#type_name>::Ok(val),
                Err(err) => #swift_bridge_path::result::<#type_name>::Err(err)
            }
        };

        println!("---> {}", s);

        s
    }

    pub fn to_c(&self) -> String {
        format!("struct __private__Result{}And{}", self.ok_ty.to_c_type(), self.err_ty.to_c_type())
    }
}

impl BuiltInResult {
    /// Go from `Result < A , B >` to a `BuiltInResult`.
    pub fn from_str_tokens(string: &str, types: &TypeDeclarations) -> Option<Self> {
        // A , B >
        let trimmed = string.trim_start_matches("Result < ");
        // A , B
        let trimmed = trimmed.trim_end_matches(" >");

        // [A, B]
        let mut ok_and_err = trimmed.split(",");
        let ok = ok_and_err.next()?.trim();
        let err = ok_and_err.next()?.trim();

        let ok = BridgedType::new_with_str(ok, types)?;
        let err = BridgedType::new_with_str(err, types)?;

        Some(BuiltInResult {
            ok_ty: Box::new(ok),
            err_ty: Box::new(err),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    /// Verify that we can parse a `Result<(), ()>`
    #[test]
    fn result_from_null_type() {
        let tokens = quote! { Result<(), ()> }.to_token_stream().to_string();

        let result = BuiltInResult::from_str_tokens(&tokens, &TypeDeclarations::default()).unwrap();

        assert!(result.ok_ty.is_null());
        assert!(result.err_ty.is_null());
    }
}
