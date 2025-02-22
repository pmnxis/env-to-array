#![doc = include_str!("../README.md")]

use base64::prelude::*;
use itertools::Itertools;
use proc_macro::TokenStream;
use syn::{parse::Parse, parse_macro_input, Token};

fn slice_to_array_token(input: &[u8]) -> TokenStream {
    format!("[{}]", input.iter().join(", "))
        .parse::<proc_macro2::TokenStream>()
        .expect("Failed to parse array")
        .into()
}

fn slice_to_usize_token(input: u64) -> TokenStream {
    format!("0x{:x}", input)
        .parse::<proc_macro2::TokenStream>()
        .expect("Failed to parse hex usize")
        .into()
}

fn slice_to_auto_sized_link_section_array(
    link_section_name: String,
    arr_name: String,
    input: &[u8],
) -> TokenStream {
    format!(
        "#[allow(unused, dead_code)]\n
        #[no_mangle]\n
        #[link_section = \"{}\"]\n
        static {}: [u8; {}] = [{}];",
        link_section_name,
        arr_name,
        input.len(),
        input.iter().join(", ")
    )
    .parse::<proc_macro2::TokenStream>()
    .expect("Failed to parse array")
    .into()
}

struct NameAndEnvInput {
    link_section_name: syn::LitStr,
    _comma0: Token![,],
    arr_name: syn::LitStr,
    _comma1: Token![,],
    env_var: syn::LitStr,
}

impl Parse for NameAndEnvInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            link_section_name: input.parse()?,
            _comma0: input.parse()?,
            arr_name: input.parse()?,
            _comma1: input.parse()?,
            env_var: input.parse()?,
        })
    }
}

#[cfg(feature = "bs58")]
/// Get from variable string, decode it from bs58 and write array as result
/// ```
/// const ID: [u8; 65] = env_to_array::bs58_to_array!("7Ax7AxYSahRegVSuU76JGWNxzdwVAPpaonY26V6JH17ToUQYSahRegVSuU76JGWNxzdwVAPpaonY26V6JH17ToUQ");
/// ```
#[proc_macro]
pub fn bs58_to_array(input: TokenStream) -> TokenStream {
    slice_to_array_token(
        bs58::decode(parse_macro_input!(input as syn::LitStr).value())
            .into_vec()
            .expect("Can't decode bs58")
            .as_slice(),
    )
}

#[cfg(feature = "bs64")]
/// Get from variable string, decode it from bs64 and write array as result
/// ```
/// const ID: [u8; 29] = env_to_array::bs64_to_array!("W7MmhbfqLQc4LbN0TUPfiflxSO6uVZ7E0NHueJ0=");
/// ```
#[proc_macro]
pub fn bs64_to_array(input: TokenStream) -> TokenStream {
    slice_to_array_token(
        BASE64_STANDARD
            .decode(parse_macro_input!(input as syn::LitStr).value())
            .expect("Can't decode bs64")
            .as_slice(),
    )
}

#[cfg(feature = "hex")]
/// Get value from variable string, decode it from hex and write array as result
/// ```
/// const ID: [u8; 31] = env_to_array::hex_to_array!("5bb32685b7ea2d07382db3744d43df89f97148eeae559ec4d0d1feefa2ee78");
/// ```
#[proc_macro]
pub fn hex_to_array(input: TokenStream) -> TokenStream {
    slice_to_array_token(
        hex::decode(parse_macro_input!(input as syn::LitStr).value())
            .expect("Can't decode hex")
            .as_slice(),
    )
}

#[cfg(feature = "bs58")]
/// Get from env variable string, decode it from bs58 and write array as result
/// ```
/// const ID: [u8; 17] = env_to_array::bs58_env_to_array!("_ENV_TO_ARRAY_BS58");
/// ```
#[proc_macro]
pub fn bs58_env_to_array(input: TokenStream) -> TokenStream {
    slice_to_array_token(
        bs58::decode(
            std::env::var(parse_macro_input!(input as syn::LitStr).value()).expect("Env variable"),
        )
        .into_vec()
        .expect("Can't decode bs58")
        .as_slice(),
    )
}

#[cfg(feature = "bs64")]
/// Get from env variable string, decode it from bs64 and write array as result
/// ```
/// const ID: [u8; 32] = env_to_array::bs64_env_to_array!("_ENV_TO_ARRAY_BS64");
/// ```
#[proc_macro]
pub fn bs64_env_to_array(input: TokenStream) -> TokenStream {
    slice_to_array_token(
        BASE64_STANDARD
            .decode(
                std::env::var(parse_macro_input!(input as syn::LitStr).value())
                    .expect("This env not found"),
            )
            .expect("Can't decode bs64")
            .as_slice(),
    )
}

#[cfg(feature = "hex")]
/// Get from env variable string, decode it from hex and write array as result
/// ```
/// const ID: [u8; 64] = env_to_array::hex_env_to_array!("_ENV_TO_ARRAY_HEX");
/// ```
#[proc_macro]
pub fn hex_env_to_array(input: TokenStream) -> TokenStream {
    slice_to_array_token(
        hex::decode(
            std::env::var(parse_macro_input!(input as syn::LitStr).value())
                .expect("This env not found"),
        )
        .expect("Can't decode hex")
        .as_slice(),
    )
}

#[cfg(feature = "hex")]
/// Get from env variable string, decode it from hex and write array as result
/// ```
/// const BOOTLOADER_OFFSET: usize = env_to_array::hex_env_to_usize!("_ENV_TO_ARRAY_HEX");
/// ```
#[proc_macro]
pub fn hex_env_to_usize(input: TokenStream) -> TokenStream {
    slice_to_usize_token(
        std::env::var(parse_macro_input!(input as syn::LitStr).value())
            .expect("This env not found")
            .parse::<u64>()
            .expect("Can't decode hex"),
    )
}

#[cfg(feature = "hex")]
/// Get from env variable string, decode it from hex and write array and sized array type as result
/// ```rs
/// env_to_array::patch_linker_section_from_hex_env!(".mp_fingerprint", "TEST_FINGER", "MP_FINGERPRINT_TOML_HEX");
/// ```
///
/// expand to ...
/// ```rs
/// #[allow(unused, dead_code)]
/// #[no_mangle]
/// #[link_section = ".mp_fingerprint"]
/// static TEST_FINGER: [u8; 14] = *b"SOME TOML HERE";
/// ```
#[proc_macro]
pub fn patch_linker_section_from_hex_env(inputs: TokenStream) -> TokenStream {
    let inputs = parse_macro_input!(inputs as NameAndEnvInput);
    slice_to_auto_sized_link_section_array(
        inputs.link_section_name.value(),
        inputs.arr_name.value(),
        hex::decode(std::env::var(inputs.env_var.value()).expect("This env not found"))
            .expect("Can't decode hex")
            .as_slice(),
    )
}

#[cfg(feature = "bs32")]
/// Get from variable string, decode it from bs58 and write array as result
/// ```
/// const ID: [u8; 5] = env_to_array::bs32_to_array!("Z0Z0Z0Z0");
/// ```
#[proc_macro]
pub fn bs32_to_array(input: TokenStream) -> TokenStream {
    slice_to_array_token(
        base32::decode(
            base32::Alphabet::Crockford,
            &parse_macro_input!(input as syn::LitStr).value(),
        )
        .expect("Can't decode bs32")
        .as_slice(),
    )
}

#[cfg(feature = "bs32")]
/// Get from env variable string, decode it from bs58 and write array as result
/// ```
/// const ID: [u8; 5] = env_to_array::bs32_env_to_array!("_ENV_TO_ARRAY_BS32");
/// ```
#[proc_macro]
pub fn bs32_env_to_array(input: TokenStream) -> TokenStream {
    slice_to_array_token(
        base32::decode(
            base32::Alphabet::Crockford,
            &std::env::var(parse_macro_input!(input as syn::LitStr).value()).expect("Env variable"),
        )
        .expect("Can't decode bs32")
        .as_slice(),
    )
}

#[cfg(feature = "bs85")]
/// Get from variable string, decode it from bs58 and write array as result
/// ```
/// const ID: [u8; 7] = env_to_array::bs85_to_array!("VPRomVPRn");
/// ```
#[proc_macro]
pub fn bs85_to_array(input: TokenStream) -> TokenStream {
    slice_to_array_token(
        base85::decode(&parse_macro_input!(input as syn::LitStr).value())
            .expect("Can't decode bs85")
            .as_slice(),
    )
}

#[cfg(feature = "bs85")]
/// Get from env variable string, decode it from bs58 and write array as result
/// ```
/// const ID: [u8; 7] = env_to_array::bs85_env_to_array!("_ENV_TO_ARRAY_BS85");
/// ```
#[proc_macro]
pub fn bs85_env_to_array(input: TokenStream) -> TokenStream {
    slice_to_array_token(
        base85::decode(
            &std::env::var(parse_macro_input!(input as syn::LitStr).value()).expect("Env variable"),
        )
        .expect("Can't decode bs85")
        .as_slice(),
    )
}
