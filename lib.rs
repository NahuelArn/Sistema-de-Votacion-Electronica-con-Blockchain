#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod sistema_votacion {

    #[ink(storage)]
    pub struct SistemaVotacion 
    {
        value: bool
    }

    impl SistemaVotacion 
    {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        #[ink(message)]
        pub fn get(&self) -> bool { self.value }
    }
}