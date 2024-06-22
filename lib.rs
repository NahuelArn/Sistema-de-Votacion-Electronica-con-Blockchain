#![cfg_attr(not(feature = "std"), no_std, no_main)]


// El código obvio está incompleto, pero lo subo por las dudas para que esté en la nube





#[ink::contract]
mod sistema_votacion
{
    ///////// USES /////////
    use ink::prelude::vec::Vec;
    use ink::prelude::string::String;
    use ink::prelude::borrow::ToOwned;



    ///////// SISTEMA /////////

    #[ink(storage)]
    pub struct SistemaVotacion
    {
        admin_id: AccountId,
        usuarios_registados: Vec<Usuario>, // Usuarios ya aprobados

        elecciones: Vec<Eleccion>,
        elecciones_finiquitadas: Vec<Eleccion>,
        elecciones_conteo_id: u64,

        resultados: Vec<ResultadosEleccion>, // Prefiero que esté en la elección pero para plantear

        peticiones_registro: Vec<Usuario> // Peticiones en espera de aprobación
    }

    impl SistemaVotacion 
    {

        //////////////////////////////////////// MESSAGES ////////////////////////////////////////



        //////////////////// SISTEMA //////////////////// 


        #[ink(constructor)]
        pub fn new(nombre_admin: String, dni_admin: String) -> Self {

            let admin_account_id = Self::env().caller();
            let admin_user = Usuario::new(admin_account_id, nombre_admin, dni_admin);

            Self {
                admin_id: admin_account_id,
                usuarios_registados: Vec::from([admin_user]),
                elecciones: Vec::new(),
                elecciones_finiquitadas: Vec::new(),
                elecciones_conteo_id: 0,
                resultados: Vec::new(),
                peticiones_registro: Vec::new()
            }
        }


        #[ink(message)]
        pub fn registrarse_en_sistema(&mut self, user_nombre: String, user_dni: String) -> Result<(), ErrorSistema>
        {
            let caller_id = Self::env().caller();
            self.consultar_inexistencia_usuario_en_sistema(caller_id)?;
        
            let user = Usuario::new(caller_id, user_nombre, user_dni);
            self.registrar_en_cola_de_sistema(user);

            Ok(())
        }


        #[ink(message)]
        pub fn get_peticiones_de_registro_sistema(&self) -> Result<Vec<Usuario>, ErrorSistema> 
        {
            self.validar_permisos(Self::env().caller(), "Sólo el administrador puede ver las peticiones de registro.".to_owned())?;

            Ok( self.peticiones_registro.clone() )
        }


        #[ink(message)]
        pub fn aprobar_usuario_sistema(&mut self, usuar_account_id: AccountId) -> Result<(), ErrorSistema>
        {
            self.validar_permisos(Self::env().caller(), "Sólo el administrador puede aprobar peticiones de usuarios para registro.".to_owned())?;
            self.consultar_peticion_sistema(usuar_account_id)?;

            self.aprobar_usuario(usuar_account_id);

            Ok(())
        }


        #[ink(message)]
        pub fn delegar_admin(&mut self, nuevo_admin_acc_id: AccountId, nuevo_admin_nombre: String, nuevo_admin_dni: String) -> Result<(), ErrorSistema>
        {
            self.validar_permisos(Self::env().caller(), "Sólo el administrador puede delegarle su rol a otra cuenta.".to_owned())?;

            self.corregir_estado_nuevo_admin(nuevo_admin_acc_id, nuevo_admin_nombre, nuevo_admin_dni);

            self.admin_id = nuevo_admin_acc_id;
            Ok(())
        }















        //////////////////// ELECCIONES ////////////////////


        #[ink(message)]
        pub fn crear_nueva_eleccion(&mut self, cargo: String, fecha_inicio: Fecha, fecha_cierre: Fecha) -> Result<(), ErrorSistema>
        {
            self.validar_permisos(Self::env().caller(), "Sólo el administrador puede crear nuevas elecciones.".to_owned())?;
            
            self.check_add_elecciones_id()?;
            let eleccion = Eleccion::new(
                self.elecciones_conteo_id, cargo,
                fecha_inicio.to_timestamp(), fecha_cierre.to_timestamp(),
                fecha_inicio, fecha_cierre
            );

            self.elecciones.push(eleccion);

            Ok(())
        }


        #[ink(message)]
        pub fn get_elecciones_actuales(&mut self) -> Result<Vec<EleccionInterfaz>, ErrorSistema> 
        {
            self.validar_caller_como_admin_o_usuario_aprobado(Self::env().caller())?;
            Ok( self.clonar_elecciones_actuales_a_interfaz(Self::env().block_timestamp()) )
        }


        #[ink(message)]
        pub fn get_elecciones_historial(&mut self) -> Result<Vec<EleccionInterfaz>, ErrorSistema>
        {
            self.validar_permisos(Self::env().caller(), "Sólo el administrador puede ver el historial de elecciones.".to_owned())?;

            let timestamp = Self::env().block_timestamp();

            let elecciones = self.clonar_elecciones_historicas_a_interfaz(timestamp);
            let elecciones = [elecciones, self.clonar_elecciones_actuales_a_interfaz(timestamp)].concat();

            Ok( elecciones )
        }
    

        #[ink(message)]
        pub fn registrarse_a_eleccion(&mut self, eleccion_id: u64, rol: Rol) -> Result<(), ErrorSistema> // Cómo identificar elección
        {
            let caller_user = self.validar_caller_como_usuario_aprobado(Self::env().caller(), "Sólo los usuarios pueden registrarse a las elecciones.".to_owned())?;

            let eleccion_index = self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoInscripcion,  Self::env().block_timestamp())?;
            
            return self.registrar_peticion_eleccion(caller_user, rol, eleccion_index);
        }


        #[ink(message)]
        pub fn get_candidatos_pendientes(&mut self, eleccion_id: u64) -> Result<Vec<Usuario>, ErrorSistema>
        {
            self.validar_permisos(Self::env().caller(), "Sólo el administrador puede ver la cola de candidatos pendientes para las elecciones.".to_owned())?;

            let eleccion_index = self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoInscripcion,  Self::env().block_timestamp())?;

            Ok( self.elecciones[eleccion_index].peticiones_candidatos.clone() )
        } 


        #[ink(message)]
        pub fn aprobar_candidato_eleccion(&mut self, eleccion_id: u64, candidato_id: AccountId) -> Result<(), ErrorSistema>
        {
            self.validar_permisos(Self::env().caller(), "Sólo el administrador puede aprobar candidatos para las elecciones.".to_owned())?;

            let eleccion_index = self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoInscripcion,  Self::env().block_timestamp())?;

            self.aprobar_candidato(eleccion_index, candidato_id)
        }


        #[ink(message)]
        pub fn votar_eleccion(&mut self, eleccion_id: u64, candidato_id: u8) -> Result<(), ErrorSistema>
        {
            let caller_user = self.validar_caller_como_usuario_aprobado(Self::env().caller(), "Sólo los usuarios pueden votar una elección.".to_owned())?;

            let eleccion_index = self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoVotacion,  Self::env().block_timestamp())?;

            self.validar_votante_aprobado_en_eleccion(caller_user.account_id, eleccion_index)?;

            if self.elecciones[eleccion_index].votos[candidato_id as usize].votos_recaudados.checked_add(1).is_none() { return Err( ErrorSistema::RepresentacionLimiteAlcanzada { msg: "Se alcanzó el límite de representación para este voto.".to_owned() }) }
        
            Ok(())
        }















        //////////////////////////////////////// PRIVATES ////////////////////////////////////////



        //////////////////// SISTEMA ////////////////////

        fn aprobar_usuario(&mut self, usuario_account_id: AccountId)
        {
            let index = self.get_usuario_en_peticiones_del_sistema(usuario_account_id);
            let user = self.peticiones_registro.remove(index.unwrap()); // Unwrap porque ya sé que existe en el vec
            self.usuarios_registados.push(user);
        }


        fn registrar_en_cola_de_sistema(&mut self, user: Usuario) {
            self.peticiones_registro.push(user);
        }


        fn corregir_estado_nuevo_admin(&mut self, new_admin_id: AccountId, new_admin_nombre: String, new_admin_dni: String)
        {
            if self.get_usuario_registrado_en_sistema(new_admin_id).is_some() { return; }

            if self.get_usuario_en_peticiones_del_sistema(new_admin_id).is_none()
            {
                let new_user = Usuario::new(new_admin_id, new_admin_nombre, new_admin_dni);
                self.registrar_en_cola_de_sistema(new_user);
            }

            self.aprobar_usuario(new_admin_id);
        }









        //////////////////// ELECCIONES ////////////////////

        fn registrar_peticion_eleccion(&mut self, user: Usuario, rol: Rol, eleccion_index: usize) -> Result<(), ErrorSistema>
        {
            self.validar_inexistencia_de_usuario_en_eleccion(user.account_id, rol.clone(), eleccion_index)?;

            match rol {
                Rol::Votante   => self.elecciones[eleccion_index].peticiones_votantes.push(user),
                Rol::Candidato => self.elecciones[eleccion_index].peticiones_candidatos.push(user)
            }
            
            Ok(())
        }


        fn clonar_elecciones_actuales_a_interfaz(&self, timestamp: u64) -> Vec<EleccionInterfaz>
        {
            let mut vec: Vec<EleccionInterfaz> = Vec::new();

            for i in 0 .. self.elecciones.len()
            {
                vec.push(
                    EleccionInterfaz::from_eleccion(
                        self.elecciones[i].get_estado_eleccion(timestamp),
                        self.elecciones[i].clone()
                    )
                );
            }

            vec
        }


        fn clonar_elecciones_historicas_a_interfaz(&self, timestamp: u64) -> Vec<EleccionInterfaz>
        {
            let mut vec: Vec<EleccionInterfaz> = Vec::new();

            for i in 0 .. self.elecciones_finiquitadas.len()
            {
                vec.push(
                    EleccionInterfaz::from_eleccion(
                        self.elecciones_finiquitadas[i].get_estado_eleccion(timestamp),
                        self.elecciones_finiquitadas[i].clone()
                    )
                );
            }

            vec
        }


        fn aprobar_candidato(&mut self, eleccion_index: usize, candidato_id: AccountId) -> Result<(), ErrorSistema>
        {
            if let Some(index) = self.get_usuario_en_peticiones_del_sistema(candidato_id)
            {
                let e = &mut self.elecciones[eleccion_index];
                e.candidatos_aprobados.push( e.peticiones_candidatos.remove(index) );

                return Ok(());
            }

            return match self.elecciones[eleccion_index].candidatos_aprobados.iter().any(|c| c.account_id == candidato_id) {
                true => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::CandidatoActualmenteAprobado { msg: "El AccountId del candidato ingresado ya está aprobado en la elección.".to_owned() } } ),
                false => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::CandidatoNoExiste { msg: "El AccountId del candidato ingresado no existe ni en las peticiones a candidato ni en los candidatos aprobados.".to_owned() } } )
            }
        }



















        //////////////////////////////////////// VALIDACIONES ////////////////////////////////////////



        //////////////////// SISTEMA ////////////////////


        fn validar_permisos(&self, caller_id: AccountId, err_msg: String) -> Result<(), ErrorSistema> {
            if !self.es_admin(caller_id) { return Err( ErrorSistema::NoSePoseenPermisos { msg: err_msg } ); }
            Ok(())
        }


        fn es_admin(&self, caller_id: AccountId) -> bool { caller_id == self.admin_id }


        fn validar_caller_como_admin_o_usuario_aprobado(&self, caller_id: AccountId) -> Result<(), ErrorSistema>
        {
            if self.es_admin(caller_id) { return Ok(()) }

            return match self.validar_usuario(caller_id) {
                Ok(_) => Ok(()),
                Err(e) => Err( e )
            }
        }


        fn validar_caller_como_usuario_aprobado(&self, caller_id: AccountId, admin_err_msg: String) -> Result<Usuario, ErrorSistema>
        {
            if self.es_admin(caller_id) { return Err( ErrorSistema::AccionUnicaDeUsuarios { msg: admin_err_msg } ); }

            self.validar_usuario(caller_id)
        }


        fn validar_usuario(&self, caller_id: AccountId) -> Result<Usuario, ErrorSistema>
        {
            let index = self.validar_usuario_en_sistema(caller_id)?;
            Ok( self.usuarios_registados[index].clone() )
        }


        fn consultar_inexistencia_usuario_en_sistema(&self, caller_id: AccountId) -> Result<(), ErrorSistema>
        {
            if self.es_admin(caller_id) { return Err( ErrorSistema::UsuarioYaRegistrado { msg: "Los administradores se registran al momento de instanciar el sistema, ó de delegar su rol.".to_owned() } ); }

            if self.existe_usuario_en_peticiones_del_sistema(caller_id) { return Err( ErrorSistema::UsuarioYaRegistradoEnPeticiones { msg: "El usuario ya se encuentra registrado en la cola de aprobación del sistema, deberá esperar a ser aprobado.".to_owned() } ); }
            if self.existe_usuario_registrado_en_sistema(caller_id) { return Err( ErrorSistema::UsuarioYaRegistrado { msg: "El usuario ya se encuentra registrado y aprobado en el sistema".to_owned() } ); }

            Ok(())
        }


        fn consultar_peticion_sistema(&self, user_id: AccountId) -> Result<(), ErrorSistema>
        {
            if self.existe_usuario_en_peticiones_del_sistema(user_id) { return Ok(()) }

            return match self.existe_usuario_registrado_en_sistema(user_id) { 
                true  => Err( ErrorSistema::UsuarioYaRegistrado { msg: "El usuario ya se encuentra registrado y aprobado en el sistema.".to_owned() } ),
                false => Err( ErrorSistema::NoExisteUsuario { msg: "El usuario no existe en el sistema.".to_owned() } )
            }
        }


        fn existe_usuario_en_peticiones_del_sistema(&self, caller_id: AccountId) -> bool {
            self.peticiones_registro.iter().any(|u| u.account_id == caller_id)
        }

        fn existe_usuario_registrado_en_sistema(&self, caller_id: AccountId) -> bool {
            self.usuarios_registados.iter().any(|u| u.account_id == caller_id)
        }


        fn get_usuario_registrado_en_sistema(&self, user_id: AccountId) -> Option<usize>
        {
            for i in 0 .. self.usuarios_registados.len() {
                if self.usuarios_registados[i].account_id == user_id { return Some(i); }
            }

            None
        }


        fn get_usuario_en_peticiones_del_sistema(&self, user_id: AccountId) -> Option<usize>
        {
            for i in 0 .. self.peticiones_registro.len() {
                if self.peticiones_registro[i].account_id == user_id { return Some(i); }
            }

            None
        }


        fn validar_usuario_en_sistema(&self, caller_id: AccountId) -> Result<usize, ErrorSistema>
        {
            if let Some(index) = self.get_usuario_registrado_en_sistema(caller_id) { return Ok(index); }

            return match self.existe_usuario_en_peticiones_del_sistema(caller_id) {
                true =>  Err( ErrorSistema::UsuarioNoAprobado { msg: "Usted se encuentra dentro de la cola de peticiones del sistema, debe esperar a ser aceptado.".to_owned() } ),
                false => Err( ErrorSistema::NoExisteUsuario { msg: "Usted no se encuentra registrado en el sistema.".to_owned() } )
            }
        }













        //////////////////// ELECCIONES ////////////////////


        fn check_add_elecciones_id(&mut self) -> Result<(), ErrorSistema> 
        {
            let result = self.elecciones_conteo_id.checked_add(1);

            if result.is_none() { return Err( ErrorSistema::RepresentacionLimiteAlcanzada { msg: "La máxima representación del tipo de dato fue alcanzada. Mantenimiento urgente.".to_owned() } ); }
        
            Ok(())
        }


        fn validar_eleccion(&mut self, eleccion_id: u64, estado_buscado: EstadoEleccion, timestamp: u64) -> Result<usize, ErrorSistema>
        {
            let eleccion_index = self.existe_eleccion(eleccion_id)?;
            self.consultar_estado_eleccion(estado_buscado, eleccion_index, timestamp)?;

            Ok( eleccion_index )
        }


        fn existe_eleccion(&self, eleccion_id: u64) -> Result<usize, ErrorSistema>
        {
            if eleccion_id >= self.elecciones_conteo_id { return Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::NoExisteEleccion { msg: "La id de elección ingresada no existe.".to_owned() } } ); }
            
            self.get_index_eleccion(eleccion_id)
        }


        fn get_index_eleccion(&self, eleccion_id: u64) -> Result<usize, ErrorSistema>
        {
            for i in 0 .. self.elecciones.len()
            {
                if self.elecciones[i].eleccion_id == eleccion_id 
                {
                    return Ok(i);
                }
            }

            Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::EleccionFinalizada { msg: "La id de elección ingresada ya finalizó.".to_owned() }} )
        }


        fn consultar_estado_eleccion(&mut self, estado_buscado: EstadoEleccion, eleccion_index: usize, timestamp: u64) -> Result<(), ErrorSistema>
        {
            let estado_eleccion = self.elecciones[eleccion_index].get_estado_eleccion(timestamp);

            if estado_buscado == estado_eleccion { return Ok(()); }

            return match estado_eleccion {
                EstadoEleccion::PeriodoInscripcion => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::EleccionEnProcesoInscripcion { msg: "La elección ingresada se encuentra en período de inscripción.".to_owned() } } ),
                EstadoEleccion::PeriodoVotacion    => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::EleccionEnProcesoVotacion    { msg: "La elección ingresada se encuentra en período de votación.".to_owned() } } ),
                EstadoEleccion::Cerrada            => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::EleccionFinalizada           { msg: "La elección ingresada se encuentra finiquitada.".to_owned() } } ),
            };
        }


        fn validar_inexistencia_de_usuario_en_eleccion(&self, caller_id: AccountId, rol: Rol, eleccion_index: usize) -> Result<(), ErrorSistema>
        {
            let e = &self.elecciones[eleccion_index];
            let mut aprob_err_msg = None;
            let mut pet_err_msg = None;


            if e.peticiones_votantes.iter().any(|p| p.account_id == caller_id) { pet_err_msg = Some( "Usted ya se encuentra en la cola de peticiones para votante, debe esperar a ser aprobado.".to_owned() ); }

            else if e.votantes_aprobados.iter().any(|p| p.account_id == caller_id) { aprob_err_msg = Some( "Usted ya se encuentra aprobado para votante.".to_owned() ); }
            
            else if e.peticiones_candidatos.iter().any(|p| p.account_id == caller_id) { pet_err_msg = Some( "Usted ya se encuentra en la cola de peticiones para candidato., debe esperar a ser aprobado".to_owned() ); }

            else if e.candidatos_aprobados.iter().any(|p| p.account_id == caller_id) { aprob_err_msg = Some( "Usted ya se encuentra aprobado para candidato.".to_owned() ); }


            if let Some(msg) = aprob_err_msg 
            { 
                return match rol {
                    Rol::Votante   => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::VotanteActualmenteAprobado { msg } } ),
                    Rol::Candidato => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::CandidatoActualmenteAprobado { msg } } )
                };
            }
            else if let Some(msg) = pet_err_msg
            {
                return match rol {
                    Rol::Votante   => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::VotanteActualmenteAprobado { msg } } ),
                    Rol::Candidato => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::CandidatoActualmenteAprobado { msg } } )
                };
            }

            Ok(())
        }


        fn validar_votante_aprobado_en_eleccion(&self, votante_id: AccountId, eleccion_index: usize) -> Result<(), ErrorSistema>
        {
            if self.elecciones[eleccion_index].votantes_aprobados.iter().any(|v| v.account_id == votante_id) { return Ok(()); }


            return match self.elecciones[eleccion_index].peticiones_votantes.iter().any(|v| v.account_id == votante_id) {
                true  => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::VotanteNoAprobado { msg: "Usted no fue aprobado para esta elección, no  tendrá permiso para votar.".to_owned() } } ),
                false => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::VotanteNoExiste { msg: "Usted nunca se registró a esta elección.".to_owned() } } )
            }            
        }
    }


    #[derive(Debug)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ErrorSistema
    {
        UsuarioYaRegistrado { msg: String },
        UsuarioYaRegistradoEnPeticiones { msg: String },
        NoExisteUsuario { msg: String },
        UsuarioNoAprobado { msg: String },
        NoSePoseenPermisos { msg: String },
        AccionUnicaDeUsuarios { msg: String },
        RepresentacionLimiteAlcanzada { msg: String },

        ErrorDeEleccion { error: ErrorEleccion },
    }


















    //////////////////////////////// ELECCIONES ////////////////////////////////


    #[derive(Clone, Debug)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct EleccionInterfaz
    {
        eleccion_id: u64,
        cargo: String,

        fecha_inicio: Fecha,
        fecha_cierre: Fecha,

        estado_eleccion: EstadoEleccion,
        candidatos_aprobados: Vec<Usuario>
    }

    impl EleccionInterfaz {
        fn new(
            eleccion_id: u64, cargo: String, fecha_inicio: Fecha, fecha_cierre: Fecha,
            estado_eleccion: EstadoEleccion, candidatos_aprobados: Vec<Usuario>
        ) -> Self
        {
            EleccionInterfaz { 
                eleccion_id, cargo, 
                fecha_inicio, fecha_cierre,
                estado_eleccion, candidatos_aprobados
            }
        }

        fn from_eleccion(estado_eleccion: EstadoEleccion, eleccion: Eleccion) -> EleccionInterfaz {
            EleccionInterfaz::new(
                eleccion.eleccion_id, eleccion.cargo,
                eleccion.fecha_inicio_interfaz,
                eleccion.fecha_cierre_interfaz,
                estado_eleccion, eleccion.candidatos_aprobados
            )
        }
    }




    #[derive(Clone, Debug)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    struct Eleccion
    {
        eleccion_id: u64, // Número alto de representación para un futuro sustento

        cargo: String,  // Decido String en vez de ENUM debido a la inmensa cantidad de cargos posibles, al fin y al cabo, quien se encarga de esto es el administrador electoral

        fecha_inicio: Timestamp, // Dato pedido del profe
        fecha_cierre: Timestamp,

        fecha_inicio_interfaz: Fecha,
        fecha_cierre_interfaz: Fecha,

        votos: Vec<CandidatoVotos>,  // No se deben poder getterar hasta que el Timestamp de cierre haya sido alcanzada

        candidatos_aprobados: Vec<Usuario>,
        peticiones_candidatos: Vec<Usuario>,

        votantes_aprobados: Vec<Usuario>,
        peticiones_votantes: Vec<Usuario>
    }

    impl Eleccion
    {
        fn new(eleccion_id:u64, cargo: String, fecha_inicio: Timestamp, fecha_cierre: Timestamp, fecha_inicio_interfaz: Fecha, fecha_cierre_interfaz: Fecha) -> Self {
            Eleccion {
                eleccion_id, cargo,
                fecha_inicio, fecha_cierre,
                fecha_inicio_interfaz, fecha_cierre_interfaz,

                votos: Vec::new(),

                candidatos_aprobados: Vec::new(), peticiones_candidatos: Vec::new(),
                votantes_aprobados: Vec::new(), peticiones_votantes: Vec::new()
            }
        }


        fn get_estado_eleccion(&self, timestamp: u64) -> EstadoEleccion
        {
            let f_ini = self.fecha_inicio;
            let f_crr = self.fecha_cierre;

            let estado: EstadoEleccion;

            if timestamp < f_ini { estado = EstadoEleccion::PeriodoInscripcion; }
            
            else if (timestamp > f_ini) && (timestamp < f_crr) { estado = EstadoEleccion::PeriodoVotacion; }

            else { estado = EstadoEleccion::Cerrada; }

            estado
        }
    }


    #[derive(Clone, Debug, PartialEq)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum EstadoEleccion { PeriodoInscripcion, PeriodoVotacion, Cerrada }


    #[derive(Debug)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ErrorEleccion
    {
        NoExisteEleccion { msg: String },

        EleccionEnProcesoInscripcion { msg: String },
        EleccionEnProcesoVotacion { msg: String },
        EleccionFinalizada { msg: String },

        CandidatoActualmenteAprobado { msg: String },
        CandidatoNoAprobado { msg : String },
        CandidatoNoExiste { msg: String },

        VotanteActualmenteAprobado { msg: String },
        VotanteNoAprobado { msg : String },
        VotanteNoExiste { msg: String },
    }















    //////////////////////////////// VOTOS Y RESULTADOS ////////////////////////////////


    #[derive(Clone, Debug)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct ResultadosEleccion
    {
        cargo: String,
        resultados: Vec<ResultadoCandidato>
    }


    #[derive(Clone, Debug)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct ResultadoCandidato
    {
        posicion: u8,
        candidato: CandidatoVotos
    }


    #[derive(Clone, Debug)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct CandidatoVotos
    {
        candidato_nombre: String,
        candidato_dni: String,
        votos_recaudados: u64,
    }
















    //////////////////////////////// USUARIOS ////////////////////////////////


    #[derive(Clone, Debug)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum Rol { Votante, Candidato }


    #[derive(Clone, Debug)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Usuario
    {
        account_id: AccountId,
        nombre: String,
        dni: String
    }


    impl Usuario
    {
        fn new(account_id: AccountId, nombre: String, dni: String) -> Self {
            Usuario { account_id, nombre, dni }
        }
    }

















    ////////////////////////////// Fecha /////////////////////////////

    #[derive(Clone, Debug)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Fecha { dia: u8, mes: u8, año: u32, hora: u8, min: u8, seg: u8 } // Año como unsigned debido a que sí o sí se tratarán fechas mayores a la actual


    impl Fecha 
    {
        fn new(dia: u8, mes: u8, año: u32, hora: u8, min: u8, seg: u8) -> Self //Result<Self, ErrorFecha> 
        { 
            /*let fecha =*/ Fecha { dia, mes, año, hora, min, seg }//;
            //fecha.validar_fecha()?;

            //Ok( fecha )
        }    

        /*
        fn validar_fecha(&self) -> Result<(), ErrorFecha>
        {
            let meses = Self::get_meses_del_año(self.año);
            let hoy = Utc::now();

            self.validar_tiempo(meses, hoy)?;

            Ok(())
        }


        fn validar_tiempo(&self, meses: [u8;12], hoy: DateTime<Utc>) -> Result<(), ErrorFecha>  // Considero que para crear una elección debe ser como mínimo el día siguiente. (El admin no activa la elección, se activa "sola")
        {
            if self.dia <= hoy.day() as u8 || self.dia <= meses[self.mes as usize] { return Err( ErrorFecha::DiaInvalido { msg: "El día ingresado es invalido.".to_owned() } ); }
            
            if self.mes < hoy.month() as u8 || self.mes > 12                       { return Err( ErrorFecha::MesInvalido { msg: "El mes ingresado es invalido.".to_owned() } ); }
            
            if self.año < hoy.year() as u32                                        { return Err( ErrorFecha::AñoInvalido { msg: "El año ingresado es invalido.".to_owned() } ); }

            if self.hora < hoy.hour() as u8 || self.hora > 23                      { return Err( ErrorFecha::HoraInvalida { msg: "La hora ingresada es incorrecta.".to_owned()} ); }

            if self.min < hoy.minute() as u8 || self.min > 59                      { return Err( ErrorFecha::MinInvalido { msg: "El minuto ingresado es incorrecto.".to_owned()} ); }

            if self.seg < hoy.second() as u8 || self.seg > 59                      { return Err( ErrorFecha::SegInvalido { msg: "El segundo ingresado es incorrecto.".to_owned()} ); }


            Ok(())
        }
        

        fn get_meses_del_año(año: u32) -> [u8; 12]
        {
            let feb = if Self::es_bisiesto(año) { 29 } else { 28 };

            [30, feb, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        }


        fn es_bisiesto(año: u32) -> bool {
            año % 4 == 0 && (año % 100 != 0 || año % 400 == 0)
        }
        */



        fn to_timestamp(&self) -> Timestamp
        {
            return 0;
            /*
            Utc.with_ymd_and_hms(
                self.año as i32, 
                self.mes as u32, 
                self.mes as u32, 
                self.hora as u32,
                self.min as u32,
                self.seg as u32
            )
            .unwrap() // Me doy el lujo de hacer unwrap debido a que ya efectué todas las validaciones necesarias al momento del "Fecha::new()"
            .timestamp() as u64
            */
        }

        /*
        fn to_date(timestamp: Timestamp) -> Fecha
        {
            let date = DateTime::from_timestamp(timestamp as i64, 0).unwrap(); // Hago unwrap debido a que si falla será por causa de un problema de código, no de usuario

            Fecha {
                dia: date.day() as u8,
                mes: date.month() as u8,
                año: date.year() as u32,
                hora: date.hour() as u8,
                min: date.minute() as u8,
                seg: date.second() as u8
            }
        }
        */
    }

    pub enum ErrorFecha
    {
        DiaInvalido { msg: String },
        MesInvalido { msg: String },
        AñoInvalido { msg: String },
        HoraInvalida { msg: String},
        MinInvalido { msg: String },
        SegInvalido { msg: String }
    }
}