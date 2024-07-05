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

#[cfg(test)]
mod tests {
    use ink::prelude::vec::Vec;
    use super::*;
    use ink::{env::test, primitives::AccountId};
    use sistema_votacion::{CandidatoVotos, ErrorSistema, Fecha, Rol, SistemaVotacion, SistemaVotacionRef, Usuario};

    //new de fecha en pub, new de usuario en pub

    //Este test testea que la funcion "reporte_registro_de_votantes" brinde los usuarios registrados y aceptados de una determinada eleccion
    #[ink::test]
    fn test_reporte_registro_de_votantes() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        //Creamos el sistema de votacion
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
        let mut sistema = SistemaVotacion::new("dante".to_string(), "7777".to_string());

        //Registramos usuarios al sistema
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        sistema.registrarse_en_sistema("Pepe".to_string(), "1111".to_string());
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        sistema.registrarse_en_sistema("Merlina".to_string(), "2222".to_string());
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
        sistema.registrarse_en_sistema("Federico".to_string(), "3333".to_string());

        //Aprobamos a bob y a alice
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
        sistema.aprobar_usuario_sistema(accounts.alice);
        sistema.aprobar_usuario_sistema(accounts.bob);

        //Creamos una eleccion
        sistema.crear_nueva_eleccion("Un cargo".to_string(), Fecha::new(12, 10, 2001, 20, 30, 00), Fecha::new(12, 10, 2001, 20, 30, 00));
        
        //Ponemos en espera de registro a bob y a alice en dicha eleccion
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        sistema.registrarse_a_eleccion(1, Rol::Votante);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        sistema.registrarse_a_eleccion(1, Rol::Votante);

        //Aprobamos el registro de bob y alice como votantes
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
        sistema.aprobar_votante_eleccion(1, "1111".to_string());
        sistema.aprobar_votante_eleccion(1, "2222".to_string());

        //Creamos el reporte con el sistema
        let reporte = Reporte::new(sistema);

        //Votantes esperados
        let vec_esperado = vec![Usuario::new(accounts.alice, "Merlina".to_string(), "2222".to_string()), Usuario::new(accounts.bob, "Pepe".to_string(), "1111".to_string())];

        //Testeos
        assert_eq!(Ok(vec_esperado), reporte.reporte_registro_de_votantes_priv(1));
        
    }
    
    #[ink::test]
    fn test_reporte_participacion() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        //Creamos el sistema de votacion
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
        let mut sistema = SistemaVotacion::new("dante".to_string(), "7777".to_string());

        //Registramos usuarios al sistema
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        sistema.registrarse_en_sistema("Pepe".to_string(), "1111".to_string());
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        sistema.registrarse_en_sistema("Merlina".to_string(), "2222".to_string());
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
        sistema.registrarse_en_sistema("Federico".to_string(), "3333".to_string());
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.frank);
        sistema.registrarse_en_sistema("Juan".to_string(), "4444".to_string());
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.eve);
        sistema.registrarse_en_sistema("Luz".to_string(), "5555".to_string());

        //Aprobamos a los usuarios
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
        sistema.aprobar_usuario_sistema(accounts.alice);
        sistema.aprobar_usuario_sistema(accounts.bob);
        sistema.aprobar_usuario_sistema(accounts.charlie);
        sistema.aprobar_usuario_sistema(accounts.frank);
        sistema.aprobar_usuario_sistema(accounts.eve);

        //Creamos una eleccion
        sistema.crear_nueva_eleccion("Un cargo".to_string(), Fecha::new(12, 10, 2001, 20, 30, 00), Fecha::new(12, 10, 2001, 20, 30, 00));

        //Agregamos 2 pendientes a candidatos a la eleccion
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.frank);
        sistema.registrarse_a_eleccion(1, Rol::Candidato);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.eve);
        sistema.registrarse_a_eleccion(1, Rol::Candidato);
        
        //Ponemos en espera de registro a bob, alice y charlie en dicha eleccion
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        sistema.registrarse_a_eleccion(1, Rol::Votante);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        sistema.registrarse_a_eleccion(1, Rol::Votante);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
        sistema.registrarse_a_eleccion(1, Rol::Votante);

        //Aprobamos a los votantes
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
        sistema.aprobar_votante_eleccion(1, "1111".to_string());
        sistema.aprobar_votante_eleccion(1, "2222".to_string());
        sistema.aprobar_votante_eleccion(1, "3333".to_string());

        //Aprobamos a los candidatos
        sistema.aprobar_candidato_eleccion(1, "4444".to_string());
        sistema.aprobar_candidato_eleccion(1, "5555".to_string());

        //bob y alice votan a frank
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        sistema.votar_eleccion(1, "4444".to_string());
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        sistema.votar_eleccion(1, "4444".to_string());

        //charlie vota a eve
        sistema.votar_eleccion(1, "5555".to_string());

        //Creamos el reporte con el sistema
        let reporte = Reporte::new(sistema);

        //Finalizar eleccion

        //Vector de Informe esperado

        //Testear

    }
    
    #[ink::test]
    fn test_reporte_resultado() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        //Creamos el sistema de votacion
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
        let mut sistema = SistemaVotacion::new("dante".to_string(), "7777".to_string());

        //Registramos usuarios al sistema
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        sistema.registrarse_en_sistema("Pepe".to_string(), "1111".to_string());
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        sistema.registrarse_en_sistema("Merlina".to_string(), "2222".to_string());
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
        sistema.registrarse_en_sistema("Federico".to_string(), "3333".to_string());
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.frank);
        sistema.registrarse_en_sistema("Juan".to_string(), "4444".to_string());

        //Aprobamos a los usuarios
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
        sistema.aprobar_usuario_sistema(accounts.alice);
        sistema.aprobar_usuario_sistema(accounts.bob);
        sistema.aprobar_usuario_sistema(accounts.charlie);
        sistema.aprobar_usuario_sistema(accounts.frank);
        sistema.aprobar_usuario_sistema(accounts.eve);

        //Creamos una eleccion
        sistema.crear_nueva_eleccion("Un cargo".to_string(), Fecha::new(12, 10, 2001, 20, 30, 00), Fecha::new(12, 10, 2001, 20, 30, 00));

        //Agregamos 2 pendientes a candidatos a la eleccion
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.frank);
        sistema.registrarse_a_eleccion(1, Rol::Candidato);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.eve);
        sistema.registrarse_a_eleccion(1, Rol::Candidato);
        
        //Ponemos en espera de registro a bob, alice y charlie en dicha eleccion
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        sistema.registrarse_a_eleccion(1, Rol::Votante);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        sistema.registrarse_a_eleccion(1, Rol::Votante);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
        sistema.registrarse_a_eleccion(1, Rol::Votante);

        //Aprobamos a los votantes
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
        sistema.aprobar_votante_eleccion(1, "1111".to_string());
        sistema.aprobar_votante_eleccion(1, "2222".to_string());
        sistema.aprobar_votante_eleccion(1, "3333".to_string());

        //Aprobamos a los candidatos
        sistema.aprobar_candidato_eleccion(1, "4444".to_string());
        sistema.aprobar_candidato_eleccion(1, "5555".to_string());

        //bob y alice votan a frank
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        sistema.votar_eleccion(1, "4444".to_string());
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        sistema.votar_eleccion(1, "4444".to_string());

        //charlie vota a eve
        sistema.votar_eleccion(1, "5555".to_string());

        //Creamos el reporte con el sistema
        let reporte = Reporte::new(sistema);

        //Finalizar eleccion

        //Vector de resultados esperados

        //Testear
    }

}
}
