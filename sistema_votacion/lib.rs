#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reporte {
    use sistema_votacion::SistemaVotacionRef;
    use sistema_votacion::Usuario;
    use ink::prelude::string::String;
    use sistema_votacion::CandidatoVotos;
    use ink::prelude::vec::Vec; // Importa Vec // Importa la macro vec!
    use sistema_votacion::ErrorSistema;
    use ink::prelude::borrow::ToOwned;
    #[ink(storage)]
    pub struct Reporte {
        sistema: SistemaVotacionRef,
    }

    impl Reporte {
        /// PERMITE INICIALIZAR EL CONTRATO CON UNA REFERENCIA AL SISTEMA DE VOTACIÓN
        /// 
        /// # Uso
        /// 
        /// La función recibe un parámetro `sistema` que es una referencia al sistema de votación y retorna una instancia del contrato `Reporte`.
        /// 
        /// # Funcionalidad
        /// 
        /// Inicializa el contrato `Reporte` con la referencia proporcionada al sistema de votación.
        /// 
        /// # Errores
        /// 
        /// No se esperan errores en la inicialización.
        #[ink(constructor)]
        pub fn new(sistema: SistemaVotacionRef) -> Self {
            Self { sistema }
        }

        /// PERMITE RECUPERAR LA LISTA DE VOTANTES REGISTRADOS Y APROBADOS PARA UNA ELECCIÓN DETERMINADA
        /// 
        /// # Uso
        /// 
        /// La función recibe el ID de la elección y retorna un `Vec<Usuario>` que contiene los votantes registrados y aprobados.
        /// 
        /// # Funcionalidad
        /// 
        /// La función llama al sistema de votación para obtener los votantes registrados y aprobados para la elección con el ID proporcionado.
        /// 
        /// # Errores
        /// 
        /// La función puede retornar un error si el ID de la elección no es válido.
        #[ink(message)]
        pub fn reporte_registro_de_votantes(&mut self, id: u64) -> Result<Vec<Usuario>, ErrorSistema> {
            self.sistema.get_elecciones_terminadas_x(id)
        }

        /// PERMITE RECUPERAR UN REPORTE DE PARTICIPACIÓN EN LAS ELECCIONES FINALIZADAS
        /// 
        /// # Uso
        /// 
        /// La función no recibe parámetros y retorna un `Vec<Informe>` que contiene información sobre la participación en cada elección finalizada.
        /// 
        /// # Funcionalidad
        /// 
        /// La función calcula la cantidad de votos emitidos y el porcentaje de participación para cada elección finalizada y retorna esta información en un `Informe`.
        /// 
        /// # Errores
        /// 
        /// La función puede retornar errores en caso de overflow al calcular los votos emitidos o el porcentaje de participación.
        #[ink(message)]
        pub fn reporte_participacion(&mut self) -> Vec<Informe> {
            let mut cant_emit: u128;
            let mut cant_total: u128;
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

        /// PERMITE RECUPERAR UN REPORTE DE RESULTADOS DE LAS ELECCIONES FINALIZADAS
        /// 
        /// # Uso
        /// 
        /// La función no recibe parámetros y retorna un `Vec<CandidatoVotos>` con la información de los votos recibidos por cada candidato, ordenados de manera descendente.
        /// 
        /// # Funcionalidad
        /// 
        /// La función obtiene los votos recibidos por cada candidato en las elecciones finalizadas y los ordena de manera descendente, mostrando al candidato con más votos primero.
        /// 
        /// # Errores
        /// 
        /// La función puede retornar un error si no se pueden obtener los resultados de las elecciones.
        #[ink(message)]
        pub fn reporte_resultado(&mut self) -> Result<Vec<CandidatoVotos>, ErrorSistema> {
            let mut votos: Vec<CandidatoVotos> = Vec::new();
            for eleccion in self.sistema.get_elecciones_finiquitadas().iter() {
                for voto in eleccion.get_eleccion_votos().iter() {
                    votos.push(voto.clone());
                }
            }
            if votos.is_empty() {
                return Err(ErrorSistema::ResultadosNoDisponibles{msg: "No hay resultados disponibles".to_owned()});
            }
            votos.sort_by(|a, b| b.get_votos_recaudados().cmp(&a.get_votos_recaudados()));
            Ok(votos)
        }
    }

    #[derive(Clone, Debug)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Informe {
        eleccion_id: u64, // Número alto de representación para un futuro sustento
        cargo: String,
        votos_emitidos: u64,
        votos_totales: u64,
        porcentaje: u128,
    }
    
    impl Informe {
        /// PERMITE CREAR UN NUEVO `INFORME`
        /// 
        /// # Uso
        /// 
        /// La función recibe los parámetros `eleccion_id`, `cargo`, `votos_emitidos`, `votos_totales` y `porcentaje` para inicializar un `Informe`.
        /// 
        /// # Funcionalidad
        /// 
        /// La función inicializa un nuevo `Informe` con los valores proporcionados.
        /// 
        /// # Errores
        /// 
        /// No se esperan errores en la inicialización.
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
