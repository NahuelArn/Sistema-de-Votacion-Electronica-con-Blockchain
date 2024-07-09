#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reporte {
    use sistema_votacion::{Eleccion, ErrorSistema, SistemaVotacion, SistemaVotacionRef, Usuario};
    use ink::prelude::string::String;
    use sistema_votacion::CandidatoVotos;
    use ink::prelude::vec::Vec; // Importa Vec // Importa la macro vec!
    use ink::prelude::borrow::ToOwned;
    
    trait Funciones{
        fn get_elecciones_terminadas_x_trait(&self, id: u64) -> Result<Vec<Usuario>, ErrorSistema>;
        fn get_elecciones_finiquitadas_trait(&self) -> Vec<Eleccion>;
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SistemaVotacionFake{
        elecciones_finiquitadas: Vec<Eleccion>,
    }

    impl Funciones for SistemaVotacionFake{
        fn get_elecciones_terminadas_x_trait(&self, id: u64) -> Result<Vec<Usuario>, ErrorSistema>{
            if id as usize >= self.elecciones_finiquitadas.len() {
                return Err(ErrorSistema::EleccionInvalida{
                    msg: "La elección ingresada no existe.".to_owned()
                });
            }
            let elecciones_votantes = self.elecciones_finiquitadas[id as usize].votantes_aprobados.clone();
            Ok(elecciones_votantes)
        }

        fn get_elecciones_finiquitadas_trait(&self) -> Vec<Eleccion>{
            self.elecciones_finiquitadas.clone()
        } 
    }
    
    impl SistemaVotacionFake{
        pub fn new(elecciones_finiquitadas: Vec<Eleccion>) -> Self{
            Self{elecciones_finiquitadas}
        }
    }

    impl Funciones for SistemaVotacionRef{
        fn get_elecciones_terminadas_x_trait(&self, id: u64) -> Result<Vec<Usuario>, ErrorSistema>{
            self.get_elecciones_terminadas_x(id)
        }

        fn get_elecciones_finiquitadas_trait(&self) -> Vec<Eleccion>{
            self.get_elecciones_finiquitadas()
        }
    }

    #[ink(storage)]
    pub struct Reporte {
        #[cfg(not(test))]
        sistema: SistemaVotacionRef,

        #[cfg(test)]
        sistema: SistemaVotacionFake
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
        #[cfg(not(test))]
        #[ink(constructor)]
        pub fn new(sistema: SistemaVotacionRef) -> Self {
            Self { sistema }
        }

        #[cfg(test)]
        pub fn new_fake(sistema: SistemaVotacionFake) -> Self{
            Self{sistema}
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
        pub fn reporte_registro_de_votantes(&self, id: u64) -> Result<Vec<Usuario>, ErrorSistema> {
            self.sistema.get_elecciones_terminadas_x_trait(id)
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
        pub fn reporte_participacion(&self) -> Vec<Informe> {
            let mut cant_emit: u128;
            let mut cant_total: u128;
            let mut informes: Vec<Informe> = Vec::new();
            
            for eleccion in self.sistema.get_elecciones_finiquitadas_trait().iter() {
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
        pub fn reporte_resultado(&self) -> Result<Vec<CandidatoVotos>, ErrorSistema> {
            let mut votos: Vec<CandidatoVotos> = Vec::new();
            for eleccion in self.sistema.get_elecciones_finiquitadas_trait().iter() {
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

    #[derive(Clone, Debug, PartialEq)]
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

    //Opcion a consultar
    #[cfg(test)]
    mod reporte_registro_de_votantes_resultado_vec_con_usuarios{
        use sistema_votacion::ErrorSistema;
        
        #[derive(Clone, PartialEq, Debug)]
        struct UsuarioFake{
            dni: String
        }

        impl UsuarioFake{
            fn new(dni: String) -> Self{
                Self{dni}
            }
        }

        #[derive(Clone)]
        struct EleccionFake{
            id: u64,
            votantes_aprobados: Vec<UsuarioFake>
        }

        impl EleccionFake{
            fn new(id: u64, votantes_aprobados: Vec<UsuarioFake>) -> Self{
                Self{id, votantes_aprobados}
            }
        }

        struct SistemaVotacionFake{ //Struct mokeado para testear
            elecciones_finiquitadas: Vec<EleccionFake>,
        }

        impl SistemaVotacionFake{
            //Metodo new mokeado para testear
            pub fn new(elecciones_finiquitadas: Vec<EleccionFake>) -> Self {
                Self {elecciones_finiquitadas}
            }

            //Metodo mokeado para testear
            pub fn get_elecciones_terminadas_x(&self, id: u64) -> Result<Vec<UsuarioFake>, ErrorSistema> {
                if id as usize >= self.elecciones_finiquitadas.len() {
                    return Err(ErrorSistema::EleccionInvalida{
                        msg: "La elección ingresada no existe.".to_owned()
                    });
                }
                let elecciones_votantes = self.elecciones_finiquitadas[id as usize].votantes_aprobados.clone();
                Ok(elecciones_votantes)
            }

        }

        struct ReporteFake{ //Struct mokeado para testear la funcion de reporte de votantes que retorna un vec con usuarios
            sistema: SistemaVotacionFake
        }

        impl ReporteFake{
            //Metodo new mokeado para testear
            pub fn new(sistema: SistemaVotacionFake) -> Self{
                Self{sistema}
            }

            //Metodo mokeado para testear
            pub fn reporte_registro_de_votantes(&self, id: u64) -> Result<Vec<UsuarioFake>, ErrorSistema> {
                self.sistema.get_elecciones_terminadas_x(id)
            }
        }

            #[ink::test]
            fn test_reporte_registro_de_votantes_() {
                //Crear votantes
                let vot1 = UsuarioFake::new("111".to_string());
                let vot2 = UsuarioFake::new("222".to_string());
                let vot3 = UsuarioFake::new("333".to_string());
                let vot4 = UsuarioFake::new("444".to_string());
                let vot5 = UsuarioFake::new("555".to_string());
                //Crear elecciones finiquitadas
                let elec1 = EleccionFake::new(0, vec![vot1.clone(), vot3.clone(), vot5.clone()]);
                let elec2 = EleccionFake::new(1, vec![vot2.clone(), vot4.clone(), vot5.clone()]);
                let elecciones_finiquitadas = vec![elec1.clone(), elec2.clone()];
                //Crear sistema
                let sistema = SistemaVotacionFake::new(elecciones_finiquitadas.clone());
                //Crear reporte que contiene al sistema
                let reporte = ReporteFake::new(sistema);
                //Realizar reporte de registro de votantes
                let resul1 = reporte.reporte_registro_de_votantes(0);
                let resul2 = reporte.reporte_registro_de_votantes(1);
                //Testeo de resultados con usuarios
                assert_eq!(Ok(vec![vot1.clone(), vot3.clone(), vot5.clone()]), resul1);
                assert_eq!(Ok(vec![vot2.clone(), vot4.clone(), vot5.clone()]), resul2);
            }
    }

    #[cfg(test)]
    mod reporte_registro_de_votantes_resultado_vec_sin_usuarios{
        use sistema_votacion::ErrorSistema;
        
        #[derive(Clone, PartialEq, Debug)]
        struct UsuarioFake;

        #[derive(Clone)]
        struct EleccionFake{
            id: u64,
            votantes_aprobados: Vec<UsuarioFake>
        }

        impl EleccionFake{
            fn new(id: u64) -> Self{
                Self{id, votantes_aprobados: Vec::new()}
            }
        }
        struct SistemaVotacionFake{ //Struct mokeado para testear
            elecciones_finiquitadas: Vec<EleccionFake>,
        }

        impl SistemaVotacionFake{
            //Metodo new mokeado para testear
            pub fn new(elecciones_finiquitadas: Vec<EleccionFake>) -> Self {
                Self {elecciones_finiquitadas}
            }

            //Metodo mokeado para testear
            pub fn get_elecciones_terminadas_x(&self, id: u64) -> Result<Vec<UsuarioFake>, ErrorSistema> {
                if id as usize >= self.elecciones_finiquitadas.len() {
                    return Err(ErrorSistema::EleccionInvalida{
                        msg: "La elección ingresada no existe.".to_owned()
                    });
                }
                let elecciones_votantes = self.elecciones_finiquitadas[id as usize].votantes_aprobados.clone();
                Ok(elecciones_votantes)
            }

        }

        struct ReporteFake{ //Struct mokeado para testear la funcion de reporte de votantes que retorna un vec con usuarios
            sistema: SistemaVotacionFake
        }

        impl ReporteFake{
            //Metodo new mokeado para testear
            pub fn new(sistema: SistemaVotacionFake) -> Self{
                Self{sistema}
            }

            //Metodo mokeado para testear
            pub fn reporte_registro_de_votantes(&self, id: u64) -> Result<Vec<UsuarioFake>, ErrorSistema> {
                self.sistema.get_elecciones_terminadas_x(id)
            }
        }

        #[ink::test]
        fn test_reporte_registro_de_votantes_() {
            //Crear elecciones finiquitadas
            let elec1 = EleccionFake::new(0);
            let elec2 = EleccionFake::new(1);
            let elecciones_finiquitadas = vec![elec1.clone(), elec2.clone()];
            //Crear sistema
            let sistema = SistemaVotacionFake::new(elecciones_finiquitadas.clone());
            //Crear reporte que contiene al sistema
            let reporte = ReporteFake::new(sistema);
            //Realizar reporte de registro de votantes
            let resul1 = reporte.reporte_registro_de_votantes(0);
            let resul2 = reporte.reporte_registro_de_votantes(1);
            //Testeo de resultados sin usuarios
            assert_eq!(Ok(Vec::new()), resul1);
            assert_eq!(Ok(Vec::new()), resul2);
        }
    }

    #[cfg(test)]
    mod reporte_registro_de_votantes_resultado_error{
        use sistema_votacion::ErrorSistema;

        #[derive(Clone, PartialEq, Debug)]
        struct UsuarioFake;

        #[derive(Clone)]
        struct EleccionFake{
            id: u64,
            votantes_aprobados: Vec<UsuarioFake>
        }

        impl EleccionFake{
            fn new(id: u64) -> Self{
                Self{id, votantes_aprobados: Vec::new()}
            }
        }
        struct SistemaVotacionFake{ //Struct mokeado para testear
            elecciones_finiquitadas: Vec<EleccionFake>,
        }

        impl SistemaVotacionFake{
            //Metodo new mokeado para testear
            pub fn new(elecciones_finiquitadas: Vec<EleccionFake>) -> Self {
                Self {elecciones_finiquitadas}
            }

            pub fn new_default() -> Self{
                Self{elecciones_finiquitadas: Vec::new()}
            }

            //Metodo mokeado para testear
            pub fn get_elecciones_terminadas_x(&self, id: u64) -> Result<Vec<UsuarioFake>, ErrorSistema> {
                if id as usize >= self.elecciones_finiquitadas.len() {
                    return Err(ErrorSistema::EleccionInvalida{
                        msg: "La elección ingresada no existe.".to_owned()
                    });
                }
                let elecciones_votantes = self.elecciones_finiquitadas[id as usize].votantes_aprobados.clone();
                Ok(elecciones_votantes)
            }

        }

        struct ReporteFake{ //Struct mokeado para testear la funcion de reporte de votantes que retorna un vec con usuarios
            sistema: SistemaVotacionFake
        }

        impl ReporteFake{
            //Metodo new mokeado para testear
            pub fn new(sistema: SistemaVotacionFake) -> Self{
                Self{sistema}
            }

            //Metodo mokeado para testear
            pub fn reporte_registro_de_votantes(&self, id: u64) -> Result<Vec<UsuarioFake>, ErrorSistema> {
                self.sistema.get_elecciones_terminadas_x(id)
            }
        }

        #[ink::test]
        fn test_reporte_registro_de_votantes() {
            //Crear sistema
            let sistema = SistemaVotacionFake::new_default();
            //Crear reporte que contiene al sistema
            let mut reporte = ReporteFake::new(sistema);
            //Test de error por no haber elecciones finiquitadas
            assert_eq!(Err(ErrorSistema::EleccionInvalida {msg: "La elección ingresada no existe.".to_owned()}), reporte.reporte_registro_de_votantes(0));
            //Crear elecciones finiquitadas
            let elec1 = EleccionFake::new(0);
            let elec2 = EleccionFake::new(1);
            reporte.sistema.elecciones_finiquitadas = vec![elec1.clone(), elec2.clone()];
            //Tests de error por id inválida
            assert_eq!(Err(ErrorSistema::EleccionInvalida {msg: "La elección ingresada no existe.".to_owned()}), reporte.reporte_registro_de_votantes(2));
            assert_eq!(Err(ErrorSistema::EleccionInvalida {msg: "La elección ingresada no existe.".to_owned()}), reporte.reporte_registro_de_votantes(3));
        }
    }

    #[cfg(test)]
    mod reporte_participacion_resultado_vec_con_informes{
        use sistema_votacion::{CandidatoVotos, ErrorSistema};
        
        #[derive(Clone, PartialEq, Debug)]
        struct UsuarioFake{
            dni: String
        }

        impl UsuarioFake{
            fn new(dni: String) -> Self{
                Self{dni}
            }
        }
        //-------------------
        #[derive(Clone)]
        pub struct CandidatoVotosFake {
            candidato_nombre: String,
            candidato_dni: String,
            votos_recaudados: u64,
        }

        impl CandidatoVotosFake{
            fn new(candidato_nombre: String, candidato_dni: String) -> Self {
                CandidatoVotosFake {
                    candidato_nombre,
                    candidato_dni,
                    votos_recaudados: 0,
                }
            }

            fn get_votos_recaudados(&self) -> u64 {
                self.votos_recaudados
            }
        }

        #[derive(Clone)]
        struct EleccionFake{
            id: u64,
            cargo: String,
            votantes_aprobados: Vec<UsuarioFake>,
            votos: Vec<CandidatoVotosFake>
        }

        impl EleccionFake{
            fn new(id: u64, votantes_aprobados: Vec<UsuarioFake>, votos: Vec<CandidatoVotosFake>, cargo: String) -> Self{
                Self{id, cargo, votantes_aprobados ,votos}
            }

            pub fn get_eleccion_votos(&self) -> Vec<CandidatoVotosFake> {
                self.votos.clone()
            }

            pub fn get_id(&self) -> u64 {
                self.id
            }
            pub fn get_cargo(&self) -> String {
                self.cargo.clone()
            }
        }

        struct InformeFake {
            eleccion_id: u64, // Número alto de representación para un futuro sustento
            cargo: String,
            votos_emitidos: u64,
            votos_totales: u64,
            porcentaje: u128,
        }

        impl InformeFake{
            fn new(eleccion_id: u64, cargo: String, votos_emitidos: u64, votos_totales: u64, porcentaje: u128) -> Self {
                InformeFake {
                    eleccion_id,
                    cargo,
                    votos_emitidos,
                    votos_totales,
                    porcentaje,
                }
            }
        }

        struct SistemaVotacionFake{ //Struct mokeado para testear
            elecciones_finiquitadas: Vec<EleccionFake>,
        }

        impl SistemaVotacionFake{
            //Metodo new mokeado para testear
            pub fn new(elecciones_finiquitadas: Vec<EleccionFake>) -> Self {
                Self {elecciones_finiquitadas}
            }

            //Metodo mokeado para testear
            pub fn get_elecciones_finiquitadas(&self) -> Vec<EleccionFake> {
                self.elecciones_finiquitadas.clone()
            }

        }

        struct ReporteFake{ //Struct mokeado para testear la funcion de reporte de votantes que retorna un vec con usuarios
            sistema: SistemaVotacionFake
        }

        impl ReporteFake{
            //Metodo new mokeado para testear
            pub fn new(sistema: SistemaVotacionFake) -> Self{
                Self{sistema}
            }

            //Metodo mokeado para testear
            pub fn reporte_participacion(&mut self) -> Vec<InformeFake> {
                let mut cant_emit: u128;
                let mut cant_total: u128;
                let mut informes: Vec<InformeFake> = Vec::new();
                
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
                    let informe = InformeFake::new(eleccion.get_id(), eleccion.get_cargo(), cant_emit as u64, cant_total as u64, porcentaje);
                    informes.push(informe);
                }
                informes
            }
        }

        #[ink::test]
        fn test_reporte_participacion(){
            
        }
    }
    //

    #[cfg(test)]
    mod tests{
        use std::vec;

        use ink::{env::account_id, primitives::AccountId};
        use sistema_votacion::Fecha;

        use super::*;

        #[ink::test]
        fn test_reporte_registro_de_votantes(){
            //Crear elecciones
            let mut elec1 = Eleccion::new(0, "Un cargo1".to_string(), Timestamp::default(), Timestamp::default(), Fecha::new(2, 1, 1, 1, 1, 1), Fecha::new(2, 1, 1, 1, 1, 1));
            let mut elec2 = Eleccion::new(1, "Un cargo2".to_string(), Timestamp::default(), Timestamp::default(), Fecha::new(2, 1, 1, 1, 1, 1), Fecha::new(2, 1, 1, 1, 1, 1));

            //Crear sistema de votacion fake y reporte
            let mut sistema = SistemaVotacionFake::new(vec![elec1.clone(), elec2.clone()]);
            let mut reporte = Reporte::new_fake(sistema);

            //Testeos vecs vacios
            let resul1 = reporte.reporte_registro_de_votantes(0);
            let resul2 = reporte.reporte_registro_de_votantes(1);
            assert_eq!(Ok(Vec::new()), resul1);
            assert_eq!(Ok(Vec::new()), resul2);

            //Testeos Errores
            assert_eq!(Err(ErrorSistema::EleccionInvalida {msg: "La elección ingresada no existe.".to_owned()}), reporte.reporte_registro_de_votantes(2));
            assert_eq!(Err(ErrorSistema::EleccionInvalida {msg: "La elección ingresada no existe.".to_owned()}), reporte.reporte_registro_de_votantes(3));

            //Crear usuarios para agregarlos a las elecciones
            let us1 = Usuario::new(account_id, "Pepe".to_string(), "111".to_string());
            let us2 = Usuario::new(account_id, "Juan".to_string(), "222".to_string());
            let us3 = Usuario::new(account_id, "Lucia".to_string(), "333".to_string());
            let us4 = Usuario::new(account_id, "Alice".to_string(), "444".to_string());

            //Agregar usuarios a los votantes aceptados de las elecciones
            elec1.sett_votantes_aceptados(vec![us1.clone(), us2.clone()]);
            elec2.sett_votantes_aceptados(vec![us3.clone(), us4.clone()]);
            
            sistema = SistemaVotacionFake::new(vec![elec1.clone(), elec2.clone()]);
            reporte = Reporte::new_fake(sistema);

            //Testeos de vec con usuarios
            let esperado1 = vec![us1.clone(), us2.clone()];
            let esperado2 = vec![us3.clone(), us4.clone()];

            assert_eq!(Ok(esperado1), reporte.reporte_registro_de_votantes(0));
            assert_eq!(Ok(esperado2), reporte.reporte_registro_de_votantes(1));
        }

        #[ink::test]
        fn test_reporte_participacion(){
            //Crear sistema de votacion fake y reporte
            let mut sistema = SistemaVotacionFake::new(Vec::new());
            let mut reporte = Reporte::new_fake(sistema);

            //Testeos de vec vacios por falta de elecciones
            let resul = reporte.reporte_participacion();
            let vec_vacio: Vec<Informe> = Vec::new();
            assert_eq!(vec_vacio, resul);

            //Crear elecciones
            let mut elec1 = Eleccion::new(0, "Un cargo1".to_string(), Timestamp::default(), Timestamp::default(), Fecha::new(2, 1, 1, 1, 1, 1), Fecha::new(2, 1, 1, 1, 1, 1));
            let mut elec2 = Eleccion::new(1, "Un cargo2".to_string(), Timestamp::default(), Timestamp::default(), Fecha::new(2, 1, 1, 1, 1, 1), Fecha::new(2, 1, 1, 1, 1, 1));

            //Reasignamos al sistema y al reporte
            sistema = SistemaVotacionFake::new(vec![elec1.clone(), elec2.clone()]);
            reporte = Reporte::new_fake(sistema);

            //Testeos con vec con informes sin votos
            
            //Testeos con vec con informes con votos

            //Testeos de errores

        }

        #[ink::test]
        fn test_reporte_resultado(){



            //Testeo de Vec con votos

            //Testeo de error
        }
    }
}
