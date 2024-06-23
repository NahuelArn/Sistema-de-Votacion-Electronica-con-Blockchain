#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reporte {
    use sistema_votacion::SistemaVotacionRef;


    #[ink(storage)]
    pub struct Reporte {
        sistema: SistemaVotacionRef,
        hola: u32,
    }

    impl Reporte {
        #[ink(constructor)]
        pub fn new(sistema: SistemaVotacionRef) -> Self {
            Self { sistema, hola: 0 }
        }

        #[ink(message)]
        pub fn pruebarda(&mut self) {
            self.hola = !self.hola;
        }
    }
}
