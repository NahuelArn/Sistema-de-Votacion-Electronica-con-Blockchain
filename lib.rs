#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reporte {
    use sistema_votacion::SistemaVotacionRef;
    use sistema_votacion::Usuario;
    use sistema_votacion::Informe;
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
            self.sistema.get_participacion().clone()
        }
        /*
        Reporte de Resultado:: Muestra el número de votos recibidos por cada candidato y
        los resultados finales, una vez cerrada la elección. Este reporte deberá mostrar de
        manera descendente los votos, donde el primer candidato será el ganador de la
        elección.
         */
        #[ink(message)]
        pub fn reporte_resultado(&mut self) -> Vec<CandidatoVotos> {
            self.sistema.get_reporte_resultados().clone()
        }
    }
    
}
