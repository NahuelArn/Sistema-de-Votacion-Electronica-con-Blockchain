#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reporte {
    use sistema_votacion::SistemaVotacionRef;
    use sistema_votacion::Usuario;
    // use sistema_votacion::Informe;
    use ink::prelude::string::String;
    use sistema_votacion::CandidatoVotos;
    use ink::prelude::vec::Vec; // Importa Vec // Importa la macro vec!

    #[ink(storage)]
    pub struct Reporte {
        sistema: SistemaVotacionRef,
    }

    impl Reporte {
        #[ink(constructor)]
        pub fn new(sistema: SistemaVotacionRef) -> Self {
            Self { sistema }
        }

        /*
        Reporte de Registro de Votantes: Detalla los votantes registrados y aprobados para
        una determinada elección.
         */
        #[ink(message)]
        pub fn reporte_registro_de_votantes(&mut self, id: u64) -> Vec<Usuario> {
            self.sistema.get_elecciones_terminadas_x(id)
        }
        /*
        Reporte de Participación: Indica la cantidad de votos emitidos y el porcentaje de
        participación, una vez cerrada la elección.
         */

        #[ink(message)]
        pub fn reporte_participacion(&mut self) -> Vec<Informe> {
            let mut cant_emit:u128;
            let mut cant_total:u128;
            let mut informes: Vec<Informe> = Vec::new();
            
            for eleccion in self.sistema.get_elecciones_finiquitadas().iter() {
                cant_emit = 0;
                for votos in eleccion.get_eleccion_votos().iter() {
                    cant_emit = cant_emit.checked_add(votos.get_votos_recaudados() as u128)
                        .expect("Error: Overflow en la suma de votos.");
                }
                cant_total = eleccion.votantes_aprobados.len() as u128;
                let porcentaje = cant_emit.checked_mul(100)
                    .expect("Error: Overflow en la multiplicación.")
                    .checked_div(cant_total)
                    .expect("Error: División por cero.");
                let informe = Informe::new(eleccion.get_id(), eleccion.get_cargo(), cant_emit as u64, cant_total as u64, porcentaje);
                informes.push(informe);
            }
            informes
        }
        /*
        Reporte de Resultado:: Muestra el número de votos recibidos por cada candidato y
        los resultados finales, una vez cerrada la elección. Este reporte deberá mostrar de
        manera descendente los votos, donde el primer candidato será el ganador de la
        elección.
         */
        #[ink(message)]
        pub fn reporte_resultado(&mut self) -> Vec<CandidatoVotos> {
            // self.sistema.get_reporte_resultados().clone()
            let mut votos: Vec<CandidatoVotos> = Vec::new();
            for eleccion in self.sistema.get_elecciones_finiquitadas().iter() {
                for voto in eleccion.get_eleccion_votos().iter() {
                    votos.push(voto.clone());
                }
            }
            votos.sort_by(|a, b| b.get_votos_recaudados().cmp(&a.get_votos_recaudados()));
            votos
        }
    }

    #[derive(Clone, Debug)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Informe{
        eleccion_id: u64, // Número alto de representación para un futuro sustento
        cargo: String,
        votos_emitidos: u64,
        votos_totales: u64,
        porcentaje: u128,
    }
    impl Informe {
        fn new(eleccion_id: u64, cargo: String, votos_emitidos: u64, votos_totales: u64, porcentaje: u128) -> Self {
            Informe {
                eleccion_id,
                cargo,
                votos_emitidos,
                votos_totales,
                porcentaje,
            }
        }
    }
    
}
