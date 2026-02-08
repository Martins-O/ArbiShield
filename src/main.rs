#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(unsafe_code)]

extern crate alloc;

use stylus_sdk::msg;
use stylus_sdk::prelude::*;
use stylus_sdk::stylus_proc::sol_storage;

sol_storage! {
    #[entrypoint]
    pub struct ArbiShield {
        address owner;
        bool initialized;
    }
}

#[external]
impl ArbiShield {
    pub fn init(&mut self) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err("Already initialized".into());
        }
        self.owner.set(msg::sender());
        self.initialized.set(true);
        Ok(())
    }

    pub fn owner(&self) -> alloy_primitives::Address {
        self.owner.get()
    }
}

#[cfg(not(any(test, feature = "export-abi")))]
#[unsafe(no_mangle)]
pub extern "C" fn main() {}

#[cfg(feature = "export-abi")]
fn main() {
    arbishield::print_from_args();
}
