use super::{CodegenTest, ExpectedCHeader, ExpectedRustTokens, ExpectedSwiftCode};
use proc_macro2::TokenStream;
use quote::quote;

/// Test code generation for Rust function that returns Option<String>
mod extern_rust_fn_return_option_string {
    use super::*;

    fn bridge_module_tokens() -> TokenStream {
        quote! {
            mod ffi {
                extern "Rust" {
                    fn some_function () -> Option<String>;
                }
            }
        }
    }

    fn expected_rust_tokens() -> ExpectedRustTokens {
        ExpectedRustTokens::Contains(quote! {
            #[export_name = "__swift_bridge__$some_function"]
            pub extern "C" fn __swift_bridge__some_function() -> *mut swift_bridge::string::RustString {
                if let Some(val) = super::some_function() {
                    swift_bridge::option::_set_option_return(true);
                    swift_bridge::string::RustString(val).box_into_raw()
                } else {
                    swift_bridge::option::_set_option_return(false);
                    std::ptr::null::<swift_bridge::string::RustString>() as *mut swift_bridge::string::RustString
                }
            }
        })
    }

    fn expected_swift_code() -> ExpectedSwiftCode {
        ExpectedSwiftCode::ContainsAfterTrim(
            r#"
func some_function() -> Optional<RustString> {
    let val = __swift_bridge__$some_function(); if _get_option_return() { return RustString(ptr: val!); } else { return nil; }
}
"#,
        )
    }

    const EXPECTED_C_HEADER: ExpectedCHeader = ExpectedCHeader::ExactAfterTrim(
        r#"
void* __swift_bridge__$some_function(void);
    "#,
    );

    #[test]
    fn extern_rust_fn_return_option_string() {
        CodegenTest {
            bridge_module_tokens: bridge_module_tokens(),
            expected_rust_tokens: expected_rust_tokens(),
            expected_swift_code: expected_swift_code(),
            expected_c_header: EXPECTED_C_HEADER,
        }
        .test();
    }
}