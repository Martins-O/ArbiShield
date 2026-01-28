#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(unsafe_code)]

#[cfg(not(any(test, feature = "export-abi")))]
#[unsafe(no_mangle)]
pub extern "C" fn main() {}

#[cfg(feature = "export-abi")]
fn main() {
    arbishield::print_from_args();
}
