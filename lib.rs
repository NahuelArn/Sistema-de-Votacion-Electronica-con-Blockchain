#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod reporte {
    use sistema_votacion::{Eleccion, ErrorSistema, Fecha, SistemaVotacion, SistemaVotacionRef, Usuario};
    use ink::{prelude::string::String};
    use sistema_votacion::CandidatoVotos;
    use ink::prelude::vec::Vec; // Importa Vec // Importa la macro vec!
    use ink::prelude::vec;
    use ink::prelude::borrow::ToOwned;
    trait Funciones{
        fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema>;
        fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion>;
    }
    //-------------------------------- A -----------------------------------
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SistemaVotacionFakeA;

    impl Funciones for SistemaVotacionFakeA{ //Caso de reporte de votantes aprobados sin votantes aprobados
        fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema>{
            Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1)))
        }

        fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion>{
            Vec::new()
        } 
    }

    impl SistemaVotacionFakeA{
        pub fn new() -> Self{
            Self{}
        }

    }
    //-----------------------------------------------------------------------
    //---------------------------------- B ----------------------------------
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SistemaVotacionFakeB;

    impl Funciones for SistemaVotacionFakeB{ //Caso de reporte de votantes aprobados con votantes aprobados
        fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema>{
            let mut elec = Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1));
            elec.set_votantes_registrados(vec![Usuario::new(AccountId::from([0x1; 32]), "Pepe".to_owned(), "111".to_owned()), Usuario::new(AccountId::from([0x2; 32]), "Juan".to_owned(), "222".to_owned())]);
            elec.set_votantes_aprobados(vec![Usuario::new(AccountId::from([0x3; 32]), "Lucas".to_owned(), "333".to_owned())]);
            Ok(elec)
        }

        fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion>{
            Vec::new()
        } 
    }

    impl SistemaVotacionFakeB{
        pub fn new() -> Self{
            Self{}
        }
    }
    //------------------------------------------------------------------------
    //---------------------------------- C -----------------------------------
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SistemaVotacionFakeC;

    impl Funciones for SistemaVotacionFakeC{ //Caso de reporte de votantes aprobados error
        fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema>{
            Err(ErrorSistema::EleccionInvalida)
        }

        fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion>{
            Vec::new()
        } 
    }

    impl SistemaVotacionFakeC{
        pub fn new() -> Self{
            Self{}
        }
    }
    //---------------------------------------------------------------------
    //-------------------------------- D ----------------------------------
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SistemaVotacionFakeD;

    impl Funciones for SistemaVotacionFakeD{ //Caso de reporte de participacion retorna informe
        fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema>{
            Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1)))
        }

        fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion>{
            let mut elec = Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1));
            let usuarios = vec![
                Usuario::new(AccountId::from([0x1; 32]), "Pepe".to_owned(), "111".to_owned()), 
                Usuario::new(AccountId::from([0x2; 32]), "Juan".to_owned(), "222".to_owned()), 
                Usuario::new(AccountId::from([0x3; 32]), "Lucia".to_owned(), "333".to_owned()), 
                Usuario::new(AccountId::from([0x4; 32]), "Franco".to_owned(), "444".to_owned())
            ];
            elec.set_votantes_aprobados(usuarios);

            let mut votos = vec![CandidatoVotos::new("Jorge".to_owned(), "999".to_owned()), CandidatoVotos::new("Mara".to_owned(), "888".to_owned())];
            votos[0].set_votos_recaudados(2);
            votos[1].set_votos_recaudados(1);

            elec.set_votos(votos);

            vec![elec]
        } 
    }

    impl SistemaVotacionFakeD{
        pub fn new() -> Self{
            Self{}
        }
    }
    //----------------------------------------------------------------------
    //------------------------------ E -------------------------------------
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SistemaVotacionFakeE;

    impl Funciones for SistemaVotacionFakeE{ //Caso de reporte 
        fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema>{
            Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1)))
        }

        fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion>{
            Vec::new()
        } 
    }

    impl SistemaVotacionFakeE{
        pub fn new() -> Self{
            Self{}
        }
    }
    //---------------------------------------------------------------------------
    //---------------------------------- F --------------------------------------
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SistemaVotacionFakeF;

    impl Funciones for SistemaVotacionFakeF{ //Caso de reporte de 
        fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema>{
            Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1)))
        }

        fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion>{
            let mut elec = Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1));

            let mut votos = vec![CandidatoVotos::new("Jorge".to_owned(), "999".to_owned()), CandidatoVotos::new("Mara".to_owned(), "888".to_owned())];
            votos[0].set_votos_recaudados(2);
            votos[1].set_votos_recaudados(1);

            elec.set_votos(votos);

            vec![elec]
        } 
    }

    impl SistemaVotacionFakeF{
        pub fn new() -> Self{
            Self{}
        }
    }
    //------------------------------------------------------------------------
    //---------------------------------- G -----------------------------------
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SistemaVotacionFakeG;

    impl Funciones for SistemaVotacionFakeG{ //Caso de reporte de 
        fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema>{
            Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1)))
        }

        fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion>{
            let mut elec = Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1));

            let mut votos = vec![
                CandidatoVotos::new("Jorge".to_owned(), "999".to_owned()), 
                CandidatoVotos::new("Mara".to_owned(), "888".to_owned()), 
                CandidatoVotos::new("Esteban".to_owned(), "777".to_owned())
            ];
            votos[0].set_votos_recaudados(5);
            votos[1].set_votos_recaudados(19);
            votos[2].set_votos_recaudados(3);

            elec.set_votos(votos);

            vec![elec]
        } 
    }

    impl SistemaVotacionFakeG{
        pub fn new() -> Self{
            Self{}
        }
    }
    //-----------------------------------------------------------------------------
    //--------------------------------------- H -----------------------------------
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SistemaVotacionFakeH;

    impl Funciones for SistemaVotacionFakeH{ //Caso de reporte de 
        fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema>{
            Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1)))
        }

        fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion>{
            Vec::new()
        } 
    }

    impl SistemaVotacionFakeH{
        pub fn new() -> Self{
            Self{}
        }
    }
    //--------------------------------------------------------------------------------
    //--------------------------------------- I --------------------------------------
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SistemaVotacionFakeI;

    impl Funciones for SistemaVotacionFakeI{ //Caso de reporte de 
        fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema>{
            Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1)))
        }

        fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion>{
            let elec = Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1));

            vec![elec]
        } 
    }

    impl SistemaVotacionFakeI{
        pub fn new() -> Self{
            Self{}
        }
    }
    //---------------------------------------------------------------------------------

    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    enum SistemaMockeado{
        A(SistemaVotacionFakeA), //Reporte de votantes retorna un informe con vecs vacios
        B(SistemaVotacionFakeB), //Reporte de votantes retorna un informe con vecs con datos 
        C(SistemaVotacionFakeC), //Reporte de votantes retorna un error
        D(SistemaVotacionFakeD), //Reporte participacion retorna informe
        E(SistemaVotacionFakeE), //Reporte participacion retorna error por inexistencia de eleccion
        F(SistemaVotacionFakeF), //Reporte participacion retorna error por division por 0
        G(SistemaVotacionFakeG), //Reporte resultado retorna vec de CandidatoVotos ordenado 
        H(SistemaVotacionFakeH), //Reporte resultado retorna error por inexistencia de eleccion
        I(SistemaVotacionFakeI), //Reporte resultado retorna error por falta de votos
    }

    impl Funciones for SistemaMockeado{
        fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema>{
            match self{
                SistemaMockeado::A(a) => a.get_elecciones_terminadas_especifica(id),
                SistemaMockeado::B(b) => b.get_elecciones_terminadas_especifica(id),
                SistemaMockeado::C(c) => c.get_elecciones_terminadas_especifica(id),
                SistemaMockeado::D(d) => d.get_elecciones_terminadas_especifica(id),
                SistemaMockeado::E(e) => e.get_elecciones_terminadas_especifica(id),
                SistemaMockeado::F(f) => f.get_elecciones_terminadas_especifica(id),
                SistemaMockeado::G(g) => g.get_elecciones_terminadas_especifica(id),
                SistemaMockeado::H(h) => h.get_elecciones_terminadas_especifica(id),
                SistemaMockeado::I(i) => i.get_elecciones_terminadas_especifica(id),
            }
        }

        fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion>{
            match self{
                SistemaMockeado::A(a) => a.get_elecciones_finiquitadas(),
                SistemaMockeado::B(b) => b.get_elecciones_finiquitadas(),
                SistemaMockeado::C(c) => c.get_elecciones_finiquitadas(),
                SistemaMockeado::D(d) => d.get_elecciones_finiquitadas(),
                SistemaMockeado::E(e) => e.get_elecciones_finiquitadas(),
                SistemaMockeado::F(f) => f.get_elecciones_finiquitadas(),
                SistemaMockeado::G(g) => g.get_elecciones_finiquitadas(),
                SistemaMockeado::H(h) => h.get_elecciones_finiquitadas(),
                SistemaMockeado::I(i) => i.get_elecciones_finiquitadas(),
            }
        }
    }

//-------------------------------------------------------------------------------------------------------------------------
    impl Funciones for SistemaVotacionRef{
        fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema>{
            self.get_elecciones_terminadas_especifica(id)
        }

        fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion>{
            self.get_elecciones_finiquitadas()
        }
    }

    #[ink(storage)]
    pub struct Reporte {
        #[cfg(not(test))]
        sistema: SistemaVotacionRef,

        #[cfg(test)]
        sistema: SistemaMockeado
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
        pub fn new_fake(sistema: SistemaMockeado) -> Self{
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
        pub fn reporte_registrados_aprobados_msg(&self, id: u64) -> Result<ReporteDetalleVotante, ErrorSistema>{
            self.reporte_registrados_aprobados(id)
        }

        fn reporte_registrados_aprobados(&self, id: u64) -> Result<ReporteDetalleVotante, ErrorSistema>{
            let eleccion_buscada: Eleccion = match self.sistema.get_elecciones_terminadas_especifica(id) {
                Ok(eleccion) => eleccion,
                Err(e) => return Err(e),
            };
            

            let vec_votantes_aprobados = eleccion_buscada.get_votantes_aprobados();
            let vec_votantes_registrados = eleccion_buscada.get_votantes_registrados();
            // Crear el informe detallado de votantes registrados y aprobados
            let informe_votantes = ReporteDetalleVotante::new(
                id,
                vec_votantes_registrados,
                vec_votantes_aprobados,
            );

            Ok(informe_votantes)
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
        pub fn reporte_participacion_msg(&self, id:u64) -> Result<Informe, ErrorSistema>{
            self.reporte_participacion(id)
        }

        fn reporte_participacion(&self, id:u64) -> Result<Informe, ErrorSistema> {
            let eleccion: Vec<Eleccion> = self.sistema.get_elecciones_finiquitadas();
            let eleccion_buscada = eleccion.iter().find(|eleccion| eleccion.get_id() == id).ok_or(ErrorSistema::ResultadosNoDisponibles)?;
            
            let mut cant_emit: u128 = 0;
            for votos in eleccion_buscada.get_eleccion_votos().iter(){
                cant_emit = cant_emit.checked_add(votos.get_votos_recaudados() as u128).expect("Error: Overflow en la suma de votos.");
            }
            // Calcular la cantidad de votos emitidos
            let cant_total = eleccion_buscada.get_votantes_aprobados().len() as u128;
            if cant_total == 0 || cant_emit == 0{
                return Err(ErrorSistema::ResultadosNoDisponibles);
            }
            let mut porcentaje: u128 = cant_emit.checked_mul(100).ok_or(ErrorSistema::ResultadosNoDisponibles)?;
            porcentaje = porcentaje.checked_div(cant_total).ok_or(ErrorSistema::ResultadosNoDisponibles)?;
            let informe = Informe::new(eleccion_buscada.get_id(), eleccion_buscada.get_cargo(), cant_emit as u64, cant_total as u64, porcentaje);
            Ok(informe)
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
        pub fn reporte_resultado_msg(&self, id: u64) -> Result<Vec<CandidatoVotos>, ErrorSistema>{
            self.reporte_resultado(id)
        }

        fn reporte_resultado(&self, id: u64) -> Result<Vec<CandidatoVotos>, ErrorSistema> {
            let mut votos: Vec<CandidatoVotos> = Vec::new();
            let eleccion: Vec<Eleccion> = self.sistema.get_elecciones_finiquitadas();
            let eleccion_buscada = eleccion.iter().find(|eleccion| eleccion.get_id() == id).ok_or(ErrorSistema::ResultadosNoDisponibles)?;
            
            for voto in eleccion_buscada.get_eleccion_votos().iter() {
                votos.push(voto.clone());
            }

            if votos.is_empty() {
                return Err(ErrorSistema::ResultadosNoDisponibles);
            }
            votos.sort_by(|a, b| b.get_votos_recaudados().cmp(&a.get_votos_recaudados()));
            Ok(votos)
        }

        #[cfg(test)]
        pub fn set_sistema(&mut self, sistema: SistemaMockeado){
            self.sistema = sistema;
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
    #[derive(Clone, Debug, PartialEq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct ReporteDetalleVotante{
        id_elecciones: u64,
        votantes_registrados: Vec<Usuario>,
        votantes_aprobados: Vec<Usuario>,
    }
    impl ReporteDetalleVotante{
        /// PERMITE CREAR UN NUEVO `REPORTE DETALLE VOTANTE`
        ///     
        /// # Uso
        ///     
        /// La función recibe los parámetros `id_elecciones`, `votantes_registrados` y `votantes_aprobados` para inicializar un `ReporteDetalleVotante`.
        /// 
        /// # Funcionalidad
        /// 
        /// La función inicializa un nuevo `ReporteDetalleVotante` con los valores seteados.
        /// 
        /// # Errores
        /// 
        /// No se esperan errores en la inicialización.
        fn new(id_elecciones: u64, votantes_registrados: Vec<Usuario>, votantes_aprobados: Vec<Usuario>) -> Self{
            ReporteDetalleVotante{
                id_elecciones,
                votantes_registrados,
                votantes_aprobados,
            }
        }
    }

    #[cfg(test)]
    mod tests{
        use std::vec;

        use super::*;

        #[ink::test]
        fn test_reporte_registro_de_votantes(){
            //Resultado informe con vecs vacios
            let sistema1 = SistemaVotacionFakeA::new();
            let mut reporte = Reporte::new_fake(SistemaMockeado::A(sistema1));
            assert_eq!(reporte.sistema.get_elecciones_finiquitadas(), Vec::new());
            assert_eq!(Ok(ReporteDetalleVotante::new(0, Vec::new(), Vec::new())), reporte.reporte_registrados_aprobados(0));
            //Resultado informe con vecs con datos
            let sistema2 = SistemaVotacionFakeB::new();
            reporte.set_sistema(SistemaMockeado::B(sistema2));
            let esperado = ReporteDetalleVotante::new(0,vec![Usuario::new(AccountId::from([0x1; 32]), "Pepe".to_owned(), "111".to_owned()), Usuario::new(AccountId::from([0x2; 32]), "Juan".to_owned(), "222".to_owned())] , vec![Usuario::new(AccountId::from([0x3; 32]), "Lucas".to_owned(), "333".to_owned())]);
            assert_eq!(reporte.sistema.get_elecciones_finiquitadas(), Vec::new());
            assert_eq!(Ok(esperado), reporte.reporte_registrados_aprobados(0));
            //Resultado informe con error
            let sistema3 = SistemaVotacionFakeC::new();
            reporte.set_sistema(SistemaMockeado::C(sistema3));
            assert_eq!(reporte.sistema.get_elecciones_finiquitadas(), Vec::new());
            assert_eq!(Err(ErrorSistema::EleccionInvalida), reporte.reporte_registrados_aprobados(0));

        }

        #[ink::test]
        fn test_reporte_participacion(){
            //Resultado informe
            let sistema1 = SistemaVotacionFakeD::new();
            let mut reporte = Reporte::new_fake(SistemaMockeado::D(sistema1));
            let elec = Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1));
            let esperado = Informe::new(elec.get_id(), elec.get_cargo(), 3, 4, 75);
            assert_eq!(Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1))), reporte.sistema.get_elecciones_terminadas_especifica(0));
            assert_eq!(Ok(esperado), reporte.reporte_participacion(0));
            //Resultado error por eleccion inexistente
            let sistema2 = SistemaVotacionFakeE::new();
            reporte.set_sistema(SistemaMockeado::E(sistema2));
            assert_eq!(Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1))), reporte.sistema.get_elecciones_terminadas_especifica(0));
            assert_eq!(Err(ErrorSistema::ResultadosNoDisponibles), reporte.reporte_participacion(0));
            //Resultado error por division por 0
            let sistema3 = SistemaVotacionFakeF::new();
            reporte.set_sistema(SistemaMockeado::F(sistema3));
            assert_eq!(Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1))), reporte.sistema.get_elecciones_terminadas_especifica(0));
            assert_eq!(Err(ErrorSistema::ResultadosNoDisponibles), reporte.reporte_participacion(0));
        }

        #[ink::test]
        fn test_reporte_resultado(){
            //Resultado vec con datos
            let sistema1 = SistemaVotacionFakeG::new();
            let mut reporte = Reporte::new_fake(SistemaMockeado::G(sistema1));
            
            let mut esperado = vec![ 
                CandidatoVotos::new("Mara".to_owned(), "888".to_owned()), 
                CandidatoVotos::new("Jorge".to_owned(), "999".to_owned()),
                CandidatoVotos::new("Esteban".to_owned(), "777".to_owned())
            ];
            esperado[1].set_votos_recaudados(5);
            esperado[0].set_votos_recaudados(19);
            esperado[2].set_votos_recaudados(3);
            assert_eq!(Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1))), reporte.sistema.get_elecciones_terminadas_especifica(0));
            assert_eq!(Ok(esperado), reporte.reporte_resultado(0));
            //Resultado error por inexistencia de eleccion
            let sistema2 = SistemaVotacionFakeH::new();
            reporte.set_sistema(SistemaMockeado::H(sistema2));
            assert_eq!(Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1))), reporte.sistema.get_elecciones_terminadas_especifica(0));
            assert_eq!(Err(ErrorSistema::ResultadosNoDisponibles), reporte.reporte_resultado(0));
            //Resultado error por falta de votos
            let sistema3 = SistemaVotacionFakeI::new();
            reporte.set_sistema(SistemaMockeado::I(sistema3));
            assert_eq!(Ok(Eleccion::new(0, "Un cargo".to_owned(), Timestamp::default(), Timestamp::default(), Fecha::new(1,1,1,1,1,1), Fecha::new(1,1,1,1,1,1))), reporte.sistema.get_elecciones_terminadas_especifica(0));
            assert_eq!(Err(ErrorSistema::ResultadosNoDisponibles), reporte.reporte_resultado(0));
        }
    }
}

