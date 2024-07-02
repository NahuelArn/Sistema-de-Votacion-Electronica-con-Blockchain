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

        /// PERMITE QUE UN USUARIO SE REGISTRE EN LA COLA DE ESPERA DEL SISTEMA
        /// Se le pasan por parametros el nombre del usuario y su DNI
        /// Y toma como AccountId al id del usuario que llama a la funcion
        /// La funcion retorna un Result<(), ErrorSistema>
        /// los casos de error pueden ser si el usuario ya esta registrado en la cola de espera, si el usuario es el admin 
        /// o si el usuario ya fue aprobado en el sistema, si no se cumple ninguna de esas condiciones el usuario se registra en el sistema 
        #[ink(message)]
        pub fn registrarse_en_sistema(&mut self, user_nombre: String, user_dni: String) -> Result<(), ErrorSistema>
        {
            self.registrarse_en_sistema_priv(user_nombre, user_dni)
        }

        fn registrarse_en_sistema_priv(&mut self, user_nombre: String, user_dni: String) -> Result<(), ErrorSistema>
        {
            let caller_id = Self::env().caller();
            self.consultar_inexistencia_usuario_en_sistema(caller_id)?;
        
            let user = Usuario::new(caller_id, user_nombre, user_dni);
            self.registrar_en_cola_de_sistema(user);

            Ok(())
        }

        ///LE PERMITE AL ADMIN VER UNA LISTA DE TODOS LOS USUARIOS EN LA COLA DE ESPERA DEL SISTEMA
        /// La funcion no recibe parametros, y devuelve un Result<Vec<Usuario>,ErrorSistema>
        /// Retorna un error siempre que el usuario que invoque la funcion no sea el admin
        #[ink(message)]
        pub fn get_peticiones_de_registro_sistema(&self) -> Result<Vec<Usuario>, ErrorSistema> 
        {
            self.get_peticiones_de_registro_sistema_priv()
        }

        fn get_peticiones_de_registro_sistema_priv(&self) -> Result<Vec<Usuario>, ErrorSistema> 
        {
            self.validar_permisos(Self::env().caller(), "Sólo el administrador puede ver las peticiones de registro.".to_owned())?;

            Ok( self.peticiones_registro.clone() )
        }

        /// LE PERMITE AL ADMIN VALIDAR A UN USUARIO EN EL SISTEMA
        /// La funcion recibe como parametro el AccountId de un usuario
        /// Si quien invoca a la funcion es el admin, la funcion valida que el accountId por parametro este registrado en el sistema
        /// de ser el caso el usuario queda validado para acceder a las funcionalidades del mismo
        /// si la llamada la realiza un usuario no admin la funcion devuelve un ErrorSistema
        /// si el accountId ya fue validado o no existe en el sistema la funcion tambien devuelve un ErrorSistema
        #[ink(message)]
        pub fn aprobar_usuario_sistema(&mut self, usuar_account_id: AccountId) -> Result<(), ErrorSistema>
        {
            self.aprobar_usuario_sistema_priv(usuar_account_id)
        }

        fn aprobar_usuario_sistema_priv(&mut self, usuar_account_id: AccountId) -> Result<(), ErrorSistema>
        {
            self.validar_permisos(Self::env().caller(), "Sólo el administrador puede aprobar peticiones de usuarios para registro.".to_owned())?;
            self.consultar_peticion_sistema(usuar_account_id)?;

            self.aprobar_usuario(usuar_account_id);

            Ok(())
        }

        /// LE PERMITE AL ADMIN TRASPASAR SU ROL A OTRO USUARIO
        /// La funcion recibe por parametros el AccountId, nombre y dni del nuevo admin y retorna un Result<(),ErrorSistema>
        /// Si quien invoca la funcion es el admin la funcion registra al nuevo admin en caso de que no este registrado
        /// y despues reemplaza el accountId del admin actual por el accountId enviado por parametro
        /// La funcion retorna un ErrorSistema si el usuario que la invoca no es el admin
        #[ink(message)]
        pub fn delegar_admin(&mut self, nuevo_admin_acc_id: AccountId, nuevo_admin_nombre: String, nuevo_admin_dni: String) -> Result<(), ErrorSistema>
        {
            self.delegar_admin_priv(nuevo_admin_acc_id, nuevo_admin_nombre, nuevo_admin_dni)
        }

        fn delegar_admin_priv(&mut self, nuevo_admin_acc_id: AccountId, nuevo_admin_nombre: String, nuevo_admin_dni: String) -> Result<(), ErrorSistema>
        {
            self.validar_permisos(Self::env().caller(), "Sólo el administrador puede delegarle su rol a otra cuenta.".to_owned())?;

            self.corregir_estado_nuevo_admin(nuevo_admin_acc_id, nuevo_admin_nombre, nuevo_admin_dni);

            self.admin_id = nuevo_admin_acc_id;
            Ok(())
        }















        //////////////////// ELECCIONES ////////////////////

        /// LE PERMITE AL ADMIN CREAR UNA NUEVA ELECCION
        /// La funcion recibe por parametro el cargo, fecha de inicio y fecha de cierre para la eleccion y devuelve un Result<(), ErrorSistema>
        /// Si el usuario que invoca la funcion es el admin, se valida el incremento a los id de eleccion para evitar desbordes, se crea la nueva 
        /// eleccion y se agrega a la lista de elecciones actuales
        /// Se devuelve un ErrorSistema si quien invoca la funcion no es el admin
        #[ink(message)]
        pub fn crear_nueva_eleccion(&mut self, cargo: String, fecha_inicio: Fecha, fecha_cierre: Fecha) -> Result<(), ErrorSistema>
        {
            self.crear_nueva_eleccion_priv(cargo,fecha_inicio,fecha_cierre)
        }

        fn crear_nueva_eleccion_priv(&mut self, cargo: String, fecha_inicio: Fecha, fecha_cierre: Fecha) -> Result<(), ErrorSistema>
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

        /// LE PERMITE A CUALQUIER USUARIO APROBADO ACCEDER A UNA LISTA DE LAS ELECCIONES EN CURSO
        /// 
        /// La funcion no recibe parametros y retorna un Result<Vec<EleccionInterfaz>,ErrorSistema>
        /// 
        /// Si quien invoca la funcion es un usuario valido en el sistema se genera un lista de las elecciones actuales en formato de interfaz y se retorna.
        /// 
        /// La funcion devuelve un ErrorSistema si quien la invoca no esta registrado o validado en el sistema.
        /// 
        /// ...
        #[ink(message)]
        pub fn get_elecciones_actuales(&mut self) -> Result<Vec<EleccionInterfaz>, ErrorSistema> 
        {
            self.get_elecciones_actuales_priv()
        }

        fn get_elecciones_actuales_priv(&mut self) -> Result<Vec<EleccionInterfaz>, ErrorSistema>
        {
            self.validar_caller_como_admin_o_usuario_aprobado(Self::env().caller())?;
            Ok( self.clonar_elecciones_actuales_a_interfaz(Self::env().block_timestamp()) )
        }

        /// LE PERMITE A CUALQUIER USUARIO APROBADO VER UNA LISTA DE TODAS LAS ELECCIONES FINALIZADAS
        /// 
        /// La funcion no recibe parametros y retorna un Result<Vec<EleccionInterfaz>,ErrorSistema>
        /// 
        /// Si quien invoca la funcion es un usuario valido en el sistema se genera un lista de las elecciones finalizadas en formato de interfaz y se retorna.
        /// 
        /// La funcion devuelve un ErrorSistema si quien la invoca no esta registrado o validado en el sistema.
        /// ...
        #[ink(message)]
        pub fn get_elecciones_historial(&mut self) -> Result<Vec<EleccionInterfaz>, ErrorSistema>
        {
            self.get_elecciones_historial_priv()
        }

        fn get_elecciones_historial_priv(&mut self) -> Result<Vec<EleccionInterfaz>, ErrorSistema>
        {
            self.validar_caller_como_admin_o_usuario_aprobado(Self::env().caller())?;

            let timestamp = Self::env().block_timestamp();

            let elecciones = self.clonar_elecciones_historicas_a_interfaz(timestamp);
            let elecciones = [elecciones, self.clonar_elecciones_actuales_a_interfaz(timestamp)].concat();

            Ok( elecciones )
        }
    
        /// LE PERMITE A UN USUARIO APROBADO REGISTRARSE A UNA ELECCION
        /// 
        /// 
        /// 
        /// La funcion recibe por parametro el id de la eleccion para registrarse, el Rol en el que quiere presentarse y retorna un Result<(),ErrorSistema>
        /// 
        /// Si quien invoca la funcion es un usuario aprobado en el sistema y el id corresponde a una eleccion valida en periodo de inscripcion,
        /// el usuario que invoca la funcion es registrado a la espera de que el admin lo valide.
        /// 
        /// Si el admin invoca la funcion, el usuario que invoca la funcion no esta aprobado o no esta registrado la funcion devuelve un ErrorSistema.
        /// 
        /// La funcion tambien devuelve un ErrorSistema si el id de la eleccion no es valido o es de una eleccion que no esta en periodo de inscripcion.
        /// 
        /// ....
        #[ink(message)]
        pub fn registrarse_a_eleccion(&mut self, eleccion_id: u64, rol: Rol) -> Result<(), ErrorSistema> // Cómo identificar elección
        {
            self.registrarse_a_eleccion_priv(eleccion_id, rol)
        }

        fn registrarse_a_eleccion_priv(&mut self, eleccion_id: u64, rol: Rol) -> Result<(), ErrorSistema>
        {
            let caller_user = self.validar_caller_como_usuario_aprobado(Self::env().caller(), "Sólo los usuarios pueden registrarse a las elecciones.".to_owned())?;

            let eleccion_index = self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoInscripcion,  Self::env().block_timestamp())?;
            
            return self.registrar_peticion_eleccion(caller_user, rol, eleccion_index);
        }

        /// PERMITE AL ADMIN RECUPERAR LA LISTA DE TODOS LOS CANDIDATOS PENDIENTES
        /// 
        /// 
        /// 
        /// #Uso
        /// 
        /// La funcion recibe el id de la eleccion de la que se quieren obtener los candidatos pendientes y retorna un Result<Vec<Usuario>,ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// Se valida que el usuario que use la funcion sea el admin, si lo es se valida que el id de la eleccion corresponda a una eleccion calida en periodo de inscripcion
        /// y se devuelve una lista de los candidatos pendientes a aprobacion en esa eleccion
        /// 
        /// #Errores
        /// 
        /// los casos de error de la funcion son cuando el usuario que la invoca no es el admin, cuando el id de la eleccion no es valido o cuando el id de la eleccion no 
        /// corresponde a una eleccion en periodo de inscripcion
        /// 
        /// ... 
        #[ink(message)]
        pub fn get_candidatos_pendientes(&mut self, eleccion_id: u64) -> Result<Vec<Usuario>, ErrorSistema>
        {
            self.get_candidatos_pendientes_priv(eleccion_id)
        }

        fn get_candidatos_pendientes_priv(&mut self, eleccion_id: u64) -> Result<Vec<Usuario>, ErrorSistema>
        {
            self.validar_permisos(Self::env().caller(), "Sólo el administrador puede ver la cola de candidatos pendientes para las elecciones.".to_owned())?;

            let eleccion_index = self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoInscripcion,  Self::env().block_timestamp())?;

            Ok( self.elecciones[eleccion_index].peticiones_candidatos.clone() )
        }

        ///PERMITE AL ADMIN APROBAR UN CANDIDATO A UNA ELECCION
        /// 
        /// #Uso
        /// 
        /// La funcion recibe el id de la eleccion en la que se quiere aprobar un candidato y el dni del candidato a aprobar, retorna un Result<(),ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// Se valida que el usuario que invoca a la funcion el admin, en ese caso se valida que el id de la eleccion sea valido y que el dni pertenezca a un candidato
        /// esperando a ser validado en esta eleccion, si todas las condiciones se cumplen el usuario dueño de ese dni queda registrado como candidato en la eleccion
        /// 
        /// #Errores
        /// 
        /// Los casos de error de la funcion se dan cuando el usuario que la invoca no es admin, cuando el id de eleccion no es valido o no pertenece a una eleccion en 
        /// periodo de inscripcion y cuando el dni del candidato no pertenece a un usuario registrado en la eleccion o un usuario ya aprobado
        /// 
        /// .
        #[ink(message)]
        pub fn aprobar_candidato_eleccion(&mut self, eleccion_id: u64, candidato_dni: String) -> Result<(), ErrorSistema>
        {
            self.aprobar_candidato_eleccion_priv(eleccion_id, candidato_dni)
        }

        fn aprobar_candidato_eleccion_priv(&mut self, eleccion_id: u64, candidato_dni: String) -> Result<(), ErrorSistema>
        {
            self.validar_permisos(Self::env().caller(), "Sólo el administrador puede aprobar candidatos para las elecciones.".to_owned())?;

            let eleccion_index = self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoInscripcion,  Self::env().block_timestamp())?;
            let candidato_index = self.validar_candidato_en_pendientes(candidato_dni, eleccion_index)?;

            self.aprobar_candidato(candidato_index, eleccion_index);
            Ok(())
        }

        /// PERMITE AL USUARIO VOTAR EN UNA ELECCION EN LA QUE ESTE ACREDITADO, FALTA REVISAR LA EXISTENCIA DEL CANDIDATO
        #[ink(message)]
        pub fn votar_eleccion(&mut self, eleccion_id: u64, candidato_dni: String) -> Result<(), ErrorSistema>
        {
            self.votar_eleccion_priv(eleccion_id, candidato_dni)
        }

        fn votar_eleccion_priv(&mut self, eleccion_id: u64, candidato_dni: String) -> Result<(), ErrorSistema>
        {
            let caller_user = self.validar_caller_como_usuario_aprobado(Self::env().caller(), "Sólo los usuarios pueden votar una elección.".to_owned())?;

            let eleccion_index = self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoVotacion,  Self::env().block_timestamp())?;

            self.validar_votante_aprobado_en_eleccion(caller_user.account_id, eleccion_index)?;

            let candidato_index = self.validar_candidato_aprobado(candidato_dni, eleccion_index)?;

            self.registrar_voto_a_candidato(candidato_index, eleccion_index)
        }















        //////////////////////////////////////// PRIVATES ////////////////////////////////////////



        //////////////////// SISTEMA ////////////////////

        //SE APRUEBA UN USUARIO EN EL SISTEMA//
        fn aprobar_usuario(&mut self, usuario_account_id: AccountId)
        {
            let index = self.get_usuario_en_peticiones_del_sistema(usuario_account_id);
            let user = self.peticiones_registro.remove(index.unwrap()); // Unwrap porque ya sé que existe en el vec
            self.usuarios_registados.push(user);
        }

        //SE AGREGA UN USUARIO A LA COLA DE ESPERA DEL SISTEMA//
        fn registrar_en_cola_de_sistema(&mut self, user: Usuario) {
            self.peticiones_registro.push(user);
        }

        // EN CASO DE QUE EL ADMIN ID NO ESTA REGISTRADO LO REGISTRA//
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

        //AGREGA A UN USUARIO A LA COLA DE ESPERA PARA SER VOTANTE O CANDIDATO EN UNA ELECCION//
        //SI EL USUARIO YA ESTA REGISTRADO EN LA ELECCION SE INFORMA//
        fn registrar_peticion_eleccion(&mut self, user: Usuario, rol: Rol, eleccion_index: usize) -> Result<(), ErrorSistema>
        {
            self.validar_inexistencia_de_usuario_en_eleccion(user.account_id, rol.clone(), eleccion_index)?;

            match rol {
                Rol::Votante   => self.elecciones[eleccion_index].peticiones_votantes.push(user),
                Rol::Candidato => self.elecciones[eleccion_index].peticiones_candidatos.push(user)
            }
            
            Ok(())
        }

        //CREA UNA LISTA DE LAS ELECCIONES EN CURSO DE LA FORMA QUE SE LEE ENE LA INTERFAZ//
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

        //CREA UNA LISTA DE LAS ELECCIONES TERMINADAS DE LA FORMA QUE SE LEE ENE LA INTERFAZ//
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

        //APRUEBA UN CANDIDATO PARA PARTICIPAR EN UNA ELECCION//
        //SI YA FUE APROBADO O NO EXISTE EN EL SISTEMA SE INFORMA//
        fn aprobar_candidato(&mut self, candidato_index: usize, eleccion_index: usize) 
        {
            let e= &mut self.elecciones[eleccion_index];

            let candidato = e.peticiones_candidatos.remove(candidato_index);
            e.candidatos_aprobados.push( candidato.clone() );

            let candidato_votos = CandidatoVotos::new(candidato.nombre, candidato.dni);
            e.votos.push( candidato_votos );
        }

        fn registrar_voto_a_candidato(&mut self, candidato_index: usize, eleccion_index: usize) -> Result<(), ErrorSistema>
        {
            if self.elecciones[eleccion_index].votos[candidato_index].votos_recaudados.checked_add(1).is_none() { 
                return Err( ErrorSistema::RepresentacionLimiteAlcanzada { msg: "Se alcanzó el límite de representación para este voto.".to_owned() }) 
            }
             // (CREO) que si un candidato está en I posición del Vec de aprobados, siempre va a estar en I posicion del Vec de votos
            // creo esto porque en el mismo proceso que apruebo un candidato (La función de arriba) pusheo el candidato a aprobados y también a votos, por ende creo que van a quedar estructuras idénticas con distintos tipo de datos

            Ok(())
        }
    



















        //////////////////////////////////////// VALIDACIONES ////////////////////////////////////////



        //////////////////// SISTEMA ////////////////////

        //CONFIRMA LOS PERMISOS DEL ADMIN//
        fn validar_permisos(&self, caller_id: AccountId, err_msg: String) -> Result<(), ErrorSistema> {
            if !self.es_admin(caller_id) { return Err( ErrorSistema::NoSePoseenPermisos { msg: err_msg } ); }
            Ok(())
        }

        //INFORMA SI EL ID PROPORCIONADO ES EL ADMIN//
        fn es_admin(&self, caller_id: AccountId) -> bool { caller_id == self.admin_id }

        //VALIDA SI EL USUARIO ES EL ADMIN O ESTA APROBADO, INFORMA EN CASO CONTRARIO//
        fn validar_caller_como_admin_o_usuario_aprobado(&self, caller_id: AccountId) -> Result<(), ErrorSistema>
        {
            if self.es_admin(caller_id) { return Ok(()) }

            return match self.validar_usuario(caller_id) {
                Ok(_) => Ok(()),
                Err(e) => Err( e )
            }
        }

        //VALIDA QUE EL USUARIO ESTE APROBADO EN EL SISTEMA, DE SER EL CASO LO DEVUELVE//
        fn validar_caller_como_usuario_aprobado(&self, caller_id: AccountId, admin_err_msg: String) -> Result<Usuario, ErrorSistema>
        {
            if self.es_admin(caller_id) { return Err( ErrorSistema::AccionUnicaDeUsuarios { msg: admin_err_msg } ); }

            self.validar_usuario(caller_id)
        }

        //VALIDA QUE UN USUARIO ESTE REGISTRADO EN EL SISTEMA//
        //EN CASO DE EXISTIR DEVUELVE UNA COPIA DEL USUARIO//
        fn validar_usuario(&self, caller_id: AccountId) -> Result<Usuario, ErrorSistema>
        {
            let index = self.validar_usuario_en_sistema(caller_id)?;
            Ok( self.usuarios_registados[index].clone() )
        }

        //CONFIRMA LA NO EXISTENCIA DE UN USUARIO EN EL SISTEMA//
        //DE EXISTIR LO INFORMA//
        fn consultar_inexistencia_usuario_en_sistema(&self, caller_id: AccountId) -> Result<(), ErrorSistema>
        {
            if self.es_admin(caller_id) { return Err( ErrorSistema::UsuarioYaRegistrado { msg: "Los administradores se registran al momento de instanciar el sistema, ó de delegar su rol.".to_owned() } ); }

            if self.existe_usuario_en_peticiones_del_sistema(caller_id) { return Err( ErrorSistema::UsuarioYaRegistradoEnPeticiones { msg: "El usuario ya se encuentra registrado en la cola de aprobación del sistema, deberá esperar a ser aprobado.".to_owned() } ); }
            if self.existe_usuario_registrado_en_sistema(caller_id) { return Err( ErrorSistema::UsuarioYaRegistrado { msg: "El usuario ya se encuentra registrado y aprobado en el sistema".to_owned() } ); }

            Ok(())
        }

        //BUSCA UN USUARIO EN LA COLA DE PETICIONES DE REGISTRO//
        //EN CASO DE NO EXISTIR O DE YA ESTAR REGISTRADO LO INFORMA//
        fn consultar_peticion_sistema(&self, user_id: AccountId) -> Result<(), ErrorSistema>
        {
            if self.existe_usuario_en_peticiones_del_sistema(user_id) { return Ok(()) }

            return match self.existe_usuario_registrado_en_sistema(user_id) { 
                true  => Err( ErrorSistema::UsuarioYaRegistrado { msg: "El usuario ya se encuentra registrado y aprobado en el sistema.".to_owned() } ),
                false => Err( ErrorSistema::NoExisteUsuario { msg: "El usuario no existe en el sistema.".to_owned() } )
            }
        }

        //INFORMA SI UN DETERMINADO USUARIO EXISTE EN LA COLA DE PETICIONES DE REGISTRO//
        fn existe_usuario_en_peticiones_del_sistema(&self, caller_id: AccountId) -> bool {
            self.peticiones_registro.iter().any(|u| u.account_id == caller_id)
        }

        //INFORMA SI UN DETERMINADO USUARIO EXISTE EN LA COLA DE USUARIOS REGISTRADOS EN EL SISTEMA//
        fn existe_usuario_registrado_en_sistema(&self, caller_id: AccountId) -> bool {
            self.usuarios_registados.iter().any(|u| u.account_id == caller_id)
        }

        //SE BUSCA Y RETORNA LA POSICION EN LA COLA DE UN USUARIO REGISTRADO ESPECIFICO EN CASO DE EXISTIR//
        fn get_usuario_registrado_en_sistema(&self, user_id: AccountId) -> Option<usize>
        {
            for i in 0 .. self.usuarios_registados.len() {
                if self.usuarios_registados[i].account_id == user_id { return Some(i); }
            }

            None
        }

        //SE BUSCA Y RETORNA LA POSICION EN LA COLA DE UN USUARIO NO REGISTRADO ESPECIFICO EN CASO DE EXISTIR//
        fn get_usuario_en_peticiones_del_sistema(&self, user_id: AccountId) -> Option<usize>
        {
            for i in 0 .. self.peticiones_registro.len() {
                if self.peticiones_registro[i].account_id == user_id { return Some(i); }
            }

            None
        }

        //VALIDA LA EXISTENCIA DE UN USUARIO EN EL SISTEMA//
        //EN CASO DE ESTAR REGISTRADO DEVUELVE SU POSICION EN LA COLA//
        //EN CASO CONTRARIO INFORMA EL ESTADO ACTUAL DEL USUARIO//
        fn validar_usuario_en_sistema(&self, caller_id: AccountId) -> Result<usize, ErrorSistema>
        {
            if let Some(index) = self.get_usuario_registrado_en_sistema(caller_id) { return Ok(index); }

            return match self.existe_usuario_en_peticiones_del_sistema(caller_id) {
                true =>  Err( ErrorSistema::UsuarioNoAprobado { msg: "Usted se encuentra dentro de la cola de peticiones del sistema, debe esperar a ser aceptado.".to_owned() } ),
                false => Err( ErrorSistema::NoExisteUsuario { msg: "Usted no se encuentra registrado en el sistema.".to_owned() } )
            }
        }













        //////////////////// ELECCIONES ////////////////////

        //INCREMENTA EN 1 EL NUMERO DE IDS DE ELECCIONES, EN CASO DE DESBORDE LO INFORMA//
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

        //VALIDA LA EXISTENCIA DE UNA ELECCION Y DEVUELVE SU POSICION EN EL VEC EN CASO CONTRARIO LO INFORMA//
        fn existe_eleccion(&self, eleccion_id: u64) -> Result<usize, ErrorSistema>
        {
            if eleccion_id >= self.elecciones_conteo_id { return Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::NoExisteEleccion { msg: "La id de elección ingresada no existe.".to_owned() } } ); }
            
            self.get_index_eleccion(eleccion_id)
        }

        //SE BUSCA LA POSICION DE UNA ELECCION EN LAS EN PROGRESO, SI LA ENCUENTRA SE DEVUELVE SINO INFORMA QUE LA ELECCION YA TERMINO//
        //(SOLO SE USA EN CASO DE QUE LA ELECCION EXISTA)//
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

        //SE CONSULTA SI LA ELECCION ESTA EN EL ESTADO DESEADO EN CASO CONTRARIO SE INFORMA EL ESTADO ACTUAL DE LA ELECCION//
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

        //VALIDA QUE UN USUARIO NO EXISTA EN NINGUNA COLA DE ESPERA O LISTA DE VOTANTES/CANDIDATOS EN CASO CONTRARIO LO INFORMA//
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

        // VALIDA QUE UN USUARIO ESTE VALIDADO COMO VOTANTE EN CASO CONTRARIO LO INFORMA//
        fn validar_votante_aprobado_en_eleccion(&self, votante_id: AccountId, eleccion_index: usize) -> Result<(), ErrorSistema>
        {
            if self.elecciones[eleccion_index].votantes_aprobados.iter().any(|v| v.account_id == votante_id) { return Ok(()); }


            return match self.elecciones[eleccion_index].peticiones_votantes.iter().any(|v| v.account_id == votante_id) {
                true  => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::VotanteNoAprobado { msg: "Usted no fue aprobado para esta elección, no  tendrá permiso para votar.".to_owned() } } ),
                false => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::VotanteNoExiste { msg: "Usted nunca se registró a esta elección.".to_owned() } } )
            }            
        }

        //EL SISTEMA VALIDA QUE EL CANDIDATO A APROBAR ESTE EN LA LISTA DE CANDIDATOS PENDIENTES
        //DE NO SER EL CASO LO INFORMA
        fn validar_candidato_en_pendientes(&self, candidato_dni: String, eleccion_index: usize) -> Result<usize, ErrorSistema>
        {
            if let Some(index) = self.get_candidato_pendiente(candidato_dni.clone(), eleccion_index) { return Ok(index) }

            return match self.get_candidato_aprobado(candidato_dni, eleccion_index).is_some() {
                true  => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::CandidatoActualmenteAprobado { msg: "El candidato ingresado ya se encuentra actualmente aprobado.".to_owned() } } ),
                false => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::CandidatoNoExiste { msg: "El candidato ingresado no existe en la elección.".to_owned() } } ),
            }
        }

    
        //EL SISTEMA VALIDA QUE EL CANDIDATO SE EN CUANTRE EN LA LISTA DE CANDIDATOS APROBADOS
        //SI NO LO ESTA SE INFORMA
        fn validar_candidato_aprobado(&self, candidato_dni: String, eleccion_index: usize) -> Result<usize, ErrorSistema>
        {
            if let Some(index) = self.get_candidato_aprobado(candidato_dni.clone(), eleccion_index) { return Ok(index) }

            return match self.get_candidato_pendiente(candidato_dni, eleccion_index).is_some() {
                true  => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::CandidatoNoAprobado { msg: "El candidato ingresado está en espera de aprobación.".to_owned() } } ),
                false => Err( ErrorSistema::ErrorDeEleccion { error: ErrorEleccion::CandidatoNoExiste { msg: "El candidato ingresado no existe en la elección.".to_owned() } } ),
            }
        }


        //EL SISTEMA BUSCA AL CANDIDATO APROBADO EN LA LISTA Y DEVUELVE SU POSICION EN EL VECTOR
        //SI NO ESTA DEVUELVE NONE
        fn get_candidato_aprobado(&self, candidato_dni: String, eleccion_index: usize) -> Option<usize> 
        {
            for i in 0 .. self.elecciones[eleccion_index].candidatos_aprobados.len() {
                if self.elecciones[eleccion_index].candidatos_aprobados[i].dni == candidato_dni { return Some(i); }
            }

            None
        }

        
        //EL SISTEMA BUSCA A UN CANDIDATO EN LA LISTA DE CANDIDATOS PENDIENTES Y DEVUELVE SU POSICION EN EL VECTOR
        //SI NO ESTA DEVUELVE NONE
        fn get_candidato_pendiente(&self, candidato_dni: String, eleccion_index: usize) -> Option<usize>  
        {
            for i in 0 .. self.elecciones[eleccion_index].candidatos_aprobados.len() {
                if self.elecciones[eleccion_index].candidatos_aprobados[i].dni == candidato_dni { return Some(i); }
            }

            None
        }
    }



    #[derive(Debug,PartialEq)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
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


    #[derive(Clone, Debug,PartialEq)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
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

        votos: Vec<CandidatoVotos>,  // No se deben poder getterar hasta que el Timestamp de cierre haya sido alcanzado

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


    #[derive(Debug,PartialEq)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
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


    impl CandidatoVotos
    {
        fn new(candidato_nombre: String, candidato_dni: String) -> Self {
            CandidatoVotos { candidato_nombre, candidato_dni, votos_recaudados: 0 }
        }
    }













    //////////////////////////////// USUARIOS ////////////////////////////////


    #[derive(Clone, Debug)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum Rol { Votante, Candidato }


    #[derive(Clone, Debug,PartialEq)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
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

    #[derive(Clone, Debug,PartialEq)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
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

    #[cfg(test)]
    mod tests{

        use super::*;

        /////////////////TEST DEL SISTEMA (METODOS INK::MESSAGE)
        #[ink::test]
        fn test_sistema_registrar_validar(){
            let accounts=ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            let mut sistema=SistemaVotacion::new("tobias".to_string(), "43107333".to_string());
            assert_eq!(Err(ErrorSistema::UsuarioYaRegistrado { msg: "Los administradores se registran al momento de instanciar el sistema, ó de delegar su rol.".to_string() }),sistema.registrarse_en_sistema_priv("julian".to_string(), "12345678".to_string()));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(()),sistema.registrarse_en_sistema("julian".to_string(), "12345678".to_string()));
            assert_eq!(Err(ErrorSistema::UsuarioYaRegistradoEnPeticiones { msg: "El usuario ya se encuentra registrado en la cola de aprobación del sistema, deberá esperar a ser aprobado.".to_string() }),sistema.registrarse_en_sistema_priv("julian".to_string(), "12345678".to_string()));
            assert_eq!(Err(ErrorSistema::NoSePoseenPermisos{msg:"Sólo el administrador puede aprobar peticiones de usuarios para registro.".to_string()}),sistema.aprobar_usuario_sistema_priv(accounts.bob));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            assert_eq!(Ok(()),sistema.aprobar_usuario_sistema_priv(accounts.bob));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Err(ErrorSistema::UsuarioYaRegistrado { msg: "El usuario ya se encuentra registrado y aprobado en el sistema".to_owned() }),sistema.registrarse_en_sistema_priv("julian".to_string(), "12345678".to_string()));
        }

        #[allow(unused)]
        #[ink::test]
        fn test_sistema_obtener_peticiones()
        {
            let accounts=ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            let mut sistema=SistemaVotacion::new("tobais".to_string(), "43107333".to_string());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            sistema.registrarse_en_sistema_priv("bob".to_string(), "12345".to_string());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            sistema.registrarse_en_sistema_priv("alice".to_string(), "22222".to_string());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
            sistema.registrarse_en_sistema_priv("charlie".to_string(), "33333".to_string());
            assert_eq!(Err(ErrorSistema::NoSePoseenPermisos { msg: "Sólo el administrador puede ver las peticiones de registro.".to_string() }),sistema.get_peticiones_de_registro_sistema_priv());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            assert_eq!(Ok(sistema.peticiones_registro.clone()),sistema.get_peticiones_de_registro_sistema_priv());
        }

        #[ink::test]
        fn test_delegar_admin()
        {
            let accounts=ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            let mut sistema=SistemaVotacion::new("tobias".to_string(), "43107333".to_string());
            assert_eq!(Ok(()),sistema.delegar_admin_priv(accounts.bob, "bob".to_string(), "12345".to_string()));
            assert_eq!(Err(ErrorSistema::NoSePoseenPermisos{msg:"Sólo el administrador puede delegarle su rol a otra cuenta.".to_string()}),sistema.delegar_admin_priv(accounts.bob, "bob".to_string(), "12345".to_string()));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(()),sistema.delegar_admin_priv(accounts.django, "tobias".to_string(), "43107333".to_string()));
        }

        /////////////////TEST ELECCIONES (METODOS INK::MESSAGE)
        #[ink::test]
        fn test_crear_eleccion()
        {
            let accounts=ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            let mut sistema=SistemaVotacion::new("tobias".to_string(), "43107333".to_string());
            assert_eq!(Ok(()),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 12, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Err(ErrorSistema::NoSePoseenPermisos { msg:"Sólo el administrador puede crear nuevas elecciones.".to_string()  }),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 12, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
            sistema.elecciones_conteo_id= 18446744073709551615;
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            assert_eq!(Err(ErrorSistema::RepresentacionLimiteAlcanzada { msg: "La máxima representación del tipo de dato fue alcanzada. Mantenimiento urgente.".to_string() }),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 12, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
        }

        #[allow(unused)]
        #[ink::test]
        fn test_get_elecciones()
        {
            let accounts=ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            let mut sistema=SistemaVotacion::new("tobias".to_string(), "43107333".to_string());
            sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 12, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 });
            assert_eq!(Ok(sistema.clonar_elecciones_actuales_a_interfaz(0)),sistema.get_elecciones_actuales_priv());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            sistema.registrarse_en_sistema_priv("bob".to_string(), "12345".to_string());
            assert_eq!(Err(ErrorSistema::UsuarioNoAprobado { msg: "Usted se encuentra dentro de la cola de peticiones del sistema, debe esperar a ser aceptado.".to_string() }),sistema.get_elecciones_actuales_priv());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            sistema.aprobar_usuario_sistema_priv(accounts.bob);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(sistema.clonar_elecciones_actuales_a_interfaz(0)),sistema.get_elecciones_actuales_priv());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            assert_eq!(Err(ErrorSistema::NoExisteUsuario { msg: "Usted no se encuentra registrado en el sistema.".to_string() }),sistema.get_elecciones_actuales_priv());
            //Se necesita testear las elecciones que ya terminaron, pero no esta el timestamp//
        }

        #[allow(unused)]
        #[ink::test]
        fn test_registrarse_eleccion()
        {
            let accounts=ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            let mut sistema=SistemaVotacion::new("tobias".to_string(), "43107333".to_string());
            sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 12, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 });
            assert_eq!(Err(ErrorSistema::AccionUnicaDeUsuarios{msg:"Sólo los usuarios pueden registrarse a las elecciones.".to_string()}),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id, Rol::Candidato));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Err(ErrorSistema::NoExisteUsuario{msg:"Usted no se encuentra registrado en el sistema.".to_string()}),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id, Rol::Candidato));
            sistema.registrarse_en_sistema_priv("bob".to_string(), "12345".to_string());
            assert_eq!(Err(ErrorSistema::UsuarioNoAprobado{msg:"Usted se encuentra dentro de la cola de peticiones del sistema, debe esperar a ser aceptado.".to_string()}),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id, Rol::Candidato));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            //faltan todos los test para los que se necesita que el timestamp este funcionando
        }
    }

}


