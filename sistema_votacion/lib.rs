#![cfg_attr(not(feature = "std"), no_std, no_main)]
pub use self::sistema_votacion::*;

#[ink::contract]
mod sistema_votacion {
    ///////// USES /////////
    use ink::prelude::borrow::ToOwned;
    use ink::prelude::string::String;
    use ink::prelude::string::ToString;
    use ink::prelude::vec::Vec;

    ///////// SISTEMA /////////

    #[ink(storage)]
    pub struct SistemaVotacion {
        admin_id: AccountId,
        usuarios_registados: Vec<Usuario>, // Usuarios ya aprobados

        elecciones: Vec<Eleccion>,
        elecciones_finiquitadas: Vec<Eleccion>,
        elecciones_conteo_id: u64,

        peticiones_registro: Vec<Usuario>, // Peticiones en espera de aprobación
    }

    impl SistemaVotacion {
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
                peticiones_registro: Vec::new()
            }
        }

        /// PERMITE QUE UN USUARIO SE REGISTRE EN LA COLA DE ESPERA DEL SISTEMA
        /// Se le pasan por parametros el nombre del usuario y su DNI
        /// Y toma como AccountId al id del usuario que llama a la funcion
        /// La funcion retorna un Result<(), ErrorInterfaz>
        /// los casos de error pueden ser si el usuario ya esta registrado en la cola de espera, si el usuario es el admin 
        /// o si el usuario ya fue aprobado en el sistema, si no se cumple ninguna de esas condiciones el usuario se registra en el sistema 
        #[ink(message)]
        pub fn registrarse_en_sistema(&mut self, user_nombre: String, user_dni: String) -> Result<(), ErrorInterfaz>
        {
            self.registrarse_en_sistema_priv(user_nombre, user_dni)
        }

        fn registrarse_en_sistema_priv(&mut self, user_nombre: String, user_dni: String) -> Result<(), ErrorInterfaz>
        {
            let caller_id = Self::env().caller();
            if let Err(error) = self.consultar_inexistencia_usuario_en_sistema(caller_id) {
                return Err(ErrorInterfaz::new(error))
            }

            let user = Usuario::new(caller_id, user_nombre, user_dni);
            self.registrar_en_cola_de_sistema(user);

            Ok(())
        }

        ///LE PERMITE AL ADMIN VER UNA LISTA DE TODOS LOS USUARIOS EN LA COLA DE ESPERA DEL SISTEMA
        /// La funcion no recibe parametros, y devuelve un Result<Vec<Usuario>,ErrorInterfaz>
        /// Retorna un error siempre que el usuario que invoque la funcion no sea el admin
        #[ink(message)]
        pub fn get_peticiones_de_registro_sistema(&self) -> Result<Vec<Usuario>, ErrorInterfaz> 
        {
            self.get_peticiones_de_registro_sistema_priv()
        }

        fn get_peticiones_de_registro_sistema_priv(&self) -> Result<Vec<Usuario>, ErrorInterfaz> 
        {
            if let Err(error) = self.validar_permisos(Self::env().caller()) {
                return Err(ErrorInterfaz::new(error))
            }

            Ok(self.peticiones_registro.clone())
        }

        /// LE PERMITE AL ADMIN VALIDAR A UN USUARIO EN EL SISTEMA
        /// La funcion recibe como parametro el AccountId de un usuario
        /// Si quien invoca a la funcion es el admin, la funcion valida que el accountId por parametro este registrado en el sistema
        /// de ser el caso el usuario queda validado para acceder a las funcionalidades del mismo
        /// si la llamada la realiza un usuario no admin la funcion devuelve un ErrorInterfaz
        /// si el accountId ya fue validado o no existe en el sistema la funcion tambien devuelve un ErrorInterfaz
        #[ink(message)]
        pub fn aprobar_usuario_sistema(&mut self, usuar_account_id: AccountId) -> Result<(), ErrorInterfaz>
        {
            self.aprobar_usuario_sistema_priv(usuar_account_id)
        }

        fn aprobar_usuario_sistema_priv(&mut self, usuar_account_id: AccountId) -> Result<(), ErrorInterfaz>
        {
            if let Err(error) = self.validar_permisos(Self::env().caller()) {
                return Err(ErrorInterfaz::new(error))
            }

            if let Err(error) = self.consultar_peticion_sistema(usuar_account_id) {
                return Err(ErrorInterfaz::new(error))
            }

            self.aprobar_usuario(usuar_account_id);

            Ok(())
        }

        /// LE PERMITE AL ADMIN TRASPASAR SU ROL A OTRO USUARIO
        /// La funcion recibe por parametros el AccountId, nombre y dni del nuevo admin y retorna un Result<(),ErrorInterfaz>
        /// Si quien invoca la funcion es el admin la funcion registra al nuevo admin en caso de que no este registrado
        /// y despues reemplaza el accountId del admin actual por el accountId enviado por parametro
        /// La funcion retorna un ErrorInterfaz si el usuario que la invoca no es el admin
        #[ink(message)]
        pub fn delegar_admin(&mut self, nuevo_admin_acc_id: AccountId, nuevo_admin_nombre: String, nuevo_admin_dni: String) -> Result<(), ErrorInterfaz>
        {
            self.delegar_admin_priv(nuevo_admin_acc_id, nuevo_admin_nombre, nuevo_admin_dni)
        }

        fn delegar_admin_priv(&mut self, nuevo_admin_acc_id: AccountId, nuevo_admin_nombre: String, nuevo_admin_dni: String) -> Result<(), ErrorInterfaz>
        {
            if let Err(error) = self.validar_permisos(Self::env().caller()) {
                return Err(ErrorInterfaz::new(error))
            }

            self.corregir_estado_nuevo_admin(nuevo_admin_acc_id, nuevo_admin_nombre, nuevo_admin_dni);

            self.admin_id = nuevo_admin_acc_id;
            Ok(())
        }

        //////////////////// ELECCIONES ////////////////////

        /// LE PERMITE AL ADMIN CREAR UNA NUEVA ELECCION
        /// 
        /// #uso
        /// La funcion recibe por parametro el cargo, fecha de inicio y fecha de cierre para la eleccion y devuelve un Result<(), ErrorInterfaz>
        /// 
        /// #funcionalidad
        /// Si el usuario que invoca la funcion es el admin, y las fechas de inicio no es anterior al dia de la fecha y la de cierre no es anterior a la de inicio
        /// se valida el incremento a los id de eleccion para evitar desbordes, se crea la nueva eleccion y se agrega a la lista de elecciones actuales.
        /// 
        /// #Errores
        /// Se devuelve un ErrorInterfaz si quien invoca la funcion no es el admin, o si las fechas no cumplen las condiciones antes mencionadas
        /// 
        /// ...
        #[ink(message)]
        pub fn crear_nueva_eleccion(&mut self, cargo: String, fecha_inicio: Fecha, fecha_cierre: Fecha) -> Result<(), ErrorInterfaz>
        {
            self.crear_nueva_eleccion_priv(cargo,fecha_inicio,fecha_cierre)
        }

        fn crear_nueva_eleccion_priv(&mut self, cargo: String, fecha_inicio: Fecha, fecha_cierre: Fecha) -> Result<(), ErrorInterfaz>
        {
            if let Err(error) = self.validar_permisos(Self::env().caller()) {
                return Err(ErrorInterfaz::new(error))
            }

            if let Err(error) = fecha_inicio.validar_fecha() {
                return Err(ErrorInterfaz::new(ErrorSistema::FechaInicioInvalida(error)));
            }

            if let Err(error) = fecha_cierre.validar_fecha() {
                return Err(ErrorInterfaz::new(ErrorSistema::FechaCierreInvalida(error)));
            }

            if fecha_cierre.fecha_pasada(fecha_inicio.to_timestamp()) {
                return Err(ErrorInterfaz::new(ErrorSistema::FechaCierreAntesInicio));
            }

            if fecha_inicio.fecha_pasada(Self::env().block_timestamp()) {
                return Err(ErrorInterfaz::new(ErrorSistema::FechaInicioPasada));
            }

            if fecha_cierre.fecha_pasada(Self::env().block_timestamp()) {
                return Err(ErrorInterfaz::new(ErrorSistema::FechaCierrePasada));
            }
            
            let eleccion = Eleccion::new(
                self.elecciones_conteo_id,
                cargo,
                fecha_inicio.to_timestamp(),
                fecha_cierre.to_timestamp(),
                fecha_inicio,
                fecha_cierre,
            );

            if let Err(error) = self.check_add_elecciones_id() {
                return Err(ErrorInterfaz::new(error))
            }

            self.elecciones.push(eleccion);

            Ok(())
        }


        ///LE PERMITE AL ADMIN CERRAR UNA ELECCION FINALIZADA Y CONTAR LOS VOTOS
        ///
        ///#Uso
        ///Al un admin llamar a la funcion con un id de una eleccion cerrada, pero que todavia esta dentro de la lista de elecciones activas,
        ///esta es movida a la lista de elecciones finalizadas y los votos son contados.
        ///Los candidatos quedan ordenados por cantidad de votos, de mayor a menor, dentro de el campo de votos en la eleccion.
        ///El ganador tambien es devuelto con sus datos, como nombre, dni, y cantidad de votos.
        ///Si no hay cantidatos o votos se devuelve un resultado vacio.
        ///
        ///#Funcionalidad
        ///La funcion chequea si el caller es admin, despues encuentra la eleccion, si es que existe. Detecta si hay votos en la eleccion,
        ///si no hay se devuelve un error, y si hay los ordena por cantidad de votos, de mayor a menor. Por ultimo mueve la eleccion a la
        ///lista de elecciones finalizadas, y devuelve una copia de los datos del ganador.
        ///
        ///#Errores
        ///Devuelve un error por la falta de privilegios de admin de ErrorSistem::NoPoseenPermisos, y un ErrorEleccion
        ///para indicar una eleccion invalida.
        #[ink(message)]
        pub fn finalizar_y_contar_eleccion(&mut self, eleccion_id: u64) -> Result<CandidatoVotos, ErrorInterfaz>
        {
            self.finalizar_y_contar_eleccion_priv(eleccion_id)
        }

        fn finalizar_y_contar_eleccion_priv(&mut self, eleccion_id: u64) -> Result<CandidatoVotos, ErrorInterfaz>
        {
            if let Err(error) = self.validar_permisos(Self::env().caller()) {
                return Err(ErrorInterfaz::new(error))
            }

            let eleccion_index = match self.validar_eleccion(eleccion_id, EstadoEleccion::Cerrada,  Self::env().block_timestamp()) {
                Ok(index) => index,
                Err(error) => return Err(ErrorInterfaz::new(error))
            };
            let mut eleccion = self.elecciones.swap_remove(eleccion_index);
            if eleccion.votos.is_empty() {
                self.elecciones_finiquitadas.push(eleccion.clone());
                return Ok(CandidatoVotos::new("Vacio".to_owned(), "Vacio".to_owned()))
            }
            eleccion.votos.sort_by_key(|candidato| candidato.votos_recaudados);
            eleccion.votos.reverse();
            self.elecciones_finiquitadas.push(eleccion.clone());
            Ok(eleccion.votos[0].clone())
        }



        /// LE PERMITE A CUALQUIER USUARIO APROBADO ACCEDER A UNA LISTA DE LAS ELECCIONES EN CURSO
        /// 
        /// #Uso
        /// La funcion no recibe parametros y retorna un Result<Vec<EleccionInterfaz>,ErrorInterfaz>
        /// 
        /// #Funcionalidad
        /// Si quien invoca la funcion es un usuario valido en el sistema se genera un lista de las elecciones actuales en formato de interfaz y se retorna.
        /// 
        /// #Errores
        /// La funcion devuelve un ErrorInterfaz si quien la invoca no esta registrado o validado en el sistema.
        /// 
        /// ...
        #[ink(message)]
        pub fn get_elecciones_actuales(&mut self) -> Result<Vec<EleccionInterfaz>, ErrorInterfaz> 
        {
            self.get_elecciones_actuales_priv()
        }

        fn get_elecciones_actuales_priv(&mut self) -> Result<Vec<EleccionInterfaz>, ErrorInterfaz>
        {
            if let Err(error) = self.validar_caller_como_admin_o_usuario_aprobado(Self::env().caller()) {
                return Err(ErrorInterfaz::new(error))
            }
            Ok(self.clonar_elecciones_actuales_a_interfaz(Self::env().block_timestamp()))
        }

        /// LE PERMITE A CUALQUIER USUARIO APROBADO VER UNA LISTA DE TODAS LAS ELECCIONES FINALIZADAS
        /// 
        /// #Uso
        /// La funcion no recibe parametros y retorna un Result<Vec<EleccionInterfaz>,ErrorInterfaz>
        /// 
        /// #Funcionalidad
        /// Si quien invoca la funcion es un usuario valido en el sistema se genera un lista de las elecciones finalizadas en formato de interfaz y se retorna.
        /// 
        /// #Errores
        /// La funcion devuelve un ErrorInterfaz si quien la invoca no esta registrado o validado en el sistema.
        /// 
        /// ...
        #[ink(message)]
        pub fn get_elecciones_historial(&mut self) -> Result<Vec<EleccionInterfaz>, ErrorInterfaz>
        {
            self.get_elecciones_historial_priv()
        }

        fn get_elecciones_historial_priv(&mut self) -> Result<Vec<EleccionInterfaz>, ErrorInterfaz>
        {
            if let Err(error) = self.validar_caller_como_admin_o_usuario_aprobado(Self::env().caller()) {
                return Err(ErrorInterfaz::new(error))
            }

            let timestamp = Self::env().block_timestamp();

            let elecciones = self.clonar_elecciones_historicas_a_interfaz();
            let elecciones = [
                elecciones,
                self.clonar_elecciones_actuales_a_interfaz(timestamp),
            ]
            .concat();

            Ok(elecciones)
        }

        /// LE PERMITE A UN USUARIO APROBADO VER UNA LISTA DE LOS VOTANTES APROBADOS DE UNA ELECCION
        #[ink(message)]
        pub fn get_elecciones_terminadas_especifica(&self, id: u64) -> Result<Eleccion, ErrorSistema> {
            if id as usize >= self.elecciones_finiquitadas.len() {
                return Err(ErrorSistema::EleccionInvalida);
            }
            let elecciones_buscada = self.elecciones_finiquitadas[id as usize].clone();
            Ok(elecciones_buscada)
        }

        // fn get_elecciones_terminadas_x_priv(&self, id: u64) -> Result<Vec<Usuario>, ErrorInterfaz> {
        //     if id as usize >= self.elecciones_finiquitadas.len() {
        //         return Err(ErrorInterfaz::new(ErrorSistema::EleccionInvalida));
        //     }
        //     let elecciones_votantes = self.elecciones_finiquitadas[id as usize].votantes_aprobados.clone();
        //     Ok(elecciones_votantes)
        // }
        #[ink(message)]
        pub fn get_elecciones_finiquitadas(&self) -> Vec<Eleccion> {
            self.get_elecciones_finiquitadas_priv()
        }
        
        fn get_elecciones_finiquitadas_priv(&self) -> Vec<Eleccion> {
            self.elecciones_finiquitadas.clone()
        }

        /// LE PERMITE A UN USUARIO APROBADO REGISTRARSE A UNA ELECCION
        /// 
        /// 
        /// #Uso
        /// La funcion recibe por parametro el id de la eleccion para registrarse, el Rol en el que quiere presentarse y retorna un Result<(),ErrorInterfaz>
        /// 
        /// #Funcionalidad
        /// Si quien invoca la funcion es un usuario aprobado en el sistema y el id corresponde a una eleccion valida en periodo de inscripcion,
        /// el usuario que invoca la funcion es registrado a la espera de que el admin lo valide.
        /// 
        /// #Errores
        /// Si el admin invoca la funcion, el usuario que invoca la funcion no esta aprobado o no esta registrado la funcion devuelve un ErrorInterfaz.
        /// 
        /// La funcion tambien devuelve un ErrorInterfaz si el id de la eleccion no es valido o es de una eleccion que no esta en periodo de inscripcion.
        /// 
        /// ....
        #[ink(message)]
        pub fn registrarse_a_eleccion(
            &mut self,
            eleccion_id: u64,
            rol: Rol,
        ) -> Result<(), ErrorInterfaz> // Cómo identificar elección
        {
            self.registrarse_a_eleccion_priv(eleccion_id, rol)
        }

        fn registrarse_a_eleccion_priv(&mut self, eleccion_id: u64, rol: Rol) -> Result<(), ErrorInterfaz>
        {
            let caller_user = match self.validar_caller_como_usuario_aprobado(Self::env().caller()) {
                Ok(user) => user,
                Err(error) => return Err(ErrorInterfaz::new(error))
            };

            let eleccion_index = match self.validar_eleccion(
                eleccion_id,
                EstadoEleccion::PeriodoInscripcion,
                Self::env().block_timestamp(),
            ) {
                Ok(index) => index,
                Err(error) => return Err(ErrorInterfaz::new(error))
            };

            match self.registrar_peticion_eleccion(caller_user, rol, eleccion_index) {
                Ok(_) => Ok(()),
                Err(error) => Err(ErrorInterfaz::new(error))
            }
        }

        /// PERMITE AL ADMIN RECUPERAR LA LISTA DE TODOS LOS CANDIDATOS PENDIENTES
        /// 
        /// 
        /// 
        /// #Uso
        /// 
        /// La funcion recibe el id de la eleccion de la que se quieren obtener los candidatos pendientes y retorna un Result<Vec<Usuario>,ErrorInterfaz>
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
        pub fn get_candidatos_pendientes(&mut self, eleccion_id: u64) -> Result<Vec<Usuario>, ErrorInterfaz>
        {
            self.get_candidatos_pendientes_priv(eleccion_id)
        }

        fn get_candidatos_pendientes_priv(&mut self, eleccion_id: u64) -> Result<Vec<Usuario>, ErrorInterfaz>
        {
            if let Err(error) = self.validar_permisos(Self::env().caller()) {
                return Err(ErrorInterfaz::new(error))
            }

            let eleccion_index = match self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoInscripcion,  Self::env().block_timestamp()) {
                Ok(index) => index,
                Err(error) => return Err(ErrorInterfaz::new(error))
            };

            Ok( self.elecciones[eleccion_index].peticiones_candidatos.clone() )
        }


        #[ink(message)]
        pub fn get_votantes_pendientes(&mut self, eleccion_id: u64) -> Result<Vec<Usuario>, ErrorInterfaz>
        {
            self.get_votantes_pendientes_priv(eleccion_id)
        }

        fn get_votantes_pendientes_priv(&mut self, eleccion_id: u64) -> Result<Vec<Usuario>, ErrorInterfaz>
        {
            if let Err(error) = self.validar_permisos(Self::env().caller()) {
                return Err(ErrorInterfaz::new(error))
            }

            let eleccion_index = match self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoInscripcion,  Self::env().block_timestamp()) {
                Ok(index) => index,
                Err(error) => return Err(ErrorInterfaz::new(error))
            };

            Ok( self.elecciones[eleccion_index].peticiones_votantes.clone() )
        }   
        

        ///PERMITE AL ADMIN APROBAR UN CANDIDATO A UNA ELECCION
        /// 
        /// #Uso
        /// 
        /// La funcion recibe el id de la eleccion en la que se quiere aprobar un candidato y el dni del candidato a aprobar, retorna un Result<(),ErrorInterfaz>
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
        pub fn aprobar_candidato_eleccion(&mut self, eleccion_id: u64, candidato_dni: String) -> Result<(), ErrorInterfaz>
        {
            self.aprobar_candidato_eleccion_priv(eleccion_id, candidato_dni)
        }

        fn aprobar_candidato_eleccion_priv(&mut self, eleccion_id: u64, candidato_dni: String) -> Result<(), ErrorInterfaz>
        {
            if let Err(error) = self.validar_permisos(Self::env().caller()) {
                return Err(ErrorInterfaz::new(error))
            }

            let eleccion_index = match self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoInscripcion,  Self::env().block_timestamp()) {
                Ok(index) => index,
                Err(error) => return Err(ErrorInterfaz::new(error))
            };
            let candidato_index = match self.validar_candidato_en_pendientes(candidato_dni, eleccion_index) {
                Ok(index) => index,
                Err(error) => return Err(ErrorInterfaz::new(error))
            };

            self.aprobar_candidato(candidato_index, eleccion_index);
            Ok(())
        }

        ///PERMITE AL ADMIN APROBAR UN VOTANTE A UNA ELECCION
        #[ink(message)]
        pub fn aprobar_votante_eleccion(&mut self, eleccion_id: u64, votante_dni: String) -> Result<(), ErrorInterfaz>
        {
            self.aprobar_votante_eleccion_priv(eleccion_id, votante_dni)
        }

        fn aprobar_votante_eleccion_priv(&mut self, eleccion_id: u64, votante_dni: String) -> Result<(), ErrorInterfaz>
        {
            if let Err(error) = self.validar_permisos(Self::env().caller()) {
                return Err(ErrorInterfaz::new(error))
            }

            let eleccion_index = match self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoInscripcion,  Self::env().block_timestamp()) {
                Ok(index) => index,
                Err(error) => return Err(ErrorInterfaz::new(error))
            };
            let votante_index = match self.validar_votante_en_pendientes(votante_dni, eleccion_index) {
                Ok(index) => index,
                Err(error) => return Err(ErrorInterfaz::new(error))
            };

            self.aprobar_votante(votante_index, eleccion_index);
            Ok(())
        }


        /// PERMITE AL USUARIO VOTAR EN UNA ELECCION EN LA QUE ESTE ACREDITADO, FALTA REVISAR LA EXISTENCIA DEL CANDIDATO
        /// 
        /// #Uso
        /// 
        /// La funcion recibe el id de una eleccion y el dni del candidato a votar, retorna un Result<(), ErrorInterfaz>
        /// 
        /// #Funcionalidad
        /// 
        /// Se valida que quien invoca a la funcion sea un usuario aprobado que el id de la eleccion pertenezca a una eleccion en periodo de votacion,
        /// luego de esto se valida que el usuario este aprobado como votante en esa eleccion y que el candidato este postulado, si se cumplen estas condiciones
        /// se registra el voto.
        /// 
        /// #Errores
        /// 
        /// Los casos de error pueden darse si quien invoca la funcion es el admin o si el usuario no esta aprobado como votante en la eleccion, 
        /// si la eleccion no esta en periodo de votacion o si el candidato no esta postulado y aprobado
        /// 
        /// ...
        #[ink(message)]
        pub fn votar_eleccion(&mut self, eleccion_id: u64, candidato_dni: String) -> Result<(), ErrorInterfaz>
        {
            self.votar_eleccion_priv(eleccion_id, candidato_dni)
        }

        fn votar_eleccion_priv(&mut self, eleccion_id: u64, candidato_dni: String) -> Result<(), ErrorInterfaz>
        {
            let caller_user = match self.validar_caller_como_usuario_aprobado(Self::env().caller()) {
                Ok(user) => user,
                Err(error) => return Err(ErrorInterfaz::new(error))
            };

            let eleccion_index = match self.validar_eleccion(eleccion_id, EstadoEleccion::PeriodoVotacion,  Self::env().block_timestamp()) {
                Ok(index) => index,
                Err(error) => return Err(ErrorInterfaz::new(error))
            };

            if self.elecciones[eleccion_index].votantes_votados.iter().any(|v| v.account_id == caller_user.account_id) {
                return Err(ErrorInterfaz::new(ErrorSistema::VotanteYaVoto))
            }

            if let Err(error) = self.validar_votante_aprobado_en_eleccion(caller_user.account_id, eleccion_index) {
                return Err(ErrorInterfaz::new(error))
            }

            let candidato_index = match self.validar_candidato_aprobado(candidato_dni, eleccion_index) {
                Ok(index) => index,
                Err(error) => return Err(ErrorInterfaz::new(error))
            };

            match self.registrar_voto_a_candidato(candidato_index, eleccion_index) {
                Ok(_) => Ok(()),
                Err(error) => Err(ErrorInterfaz::new(error))
            }
        }

        //////////////////////////////////////// PRIVATES ////////////////////////////////////////

        //////////////////// SISTEMA ////////////////////

        ///SE APRUEBA UN USUARIO EN EL SISTEMA
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion busca el AccountId proporcionado en la lista de peticiones de registro del sistema y se guarda la posicion del mismo
        /// despues elemina al user de la lista de peticiones y lo agrega a la lista de usuarios validos
        /// 
        /// #Errores
        /// 
        /// La funcion no maneja errores ya que se filtran anteriormente
        /// 
        /// ...
        fn aprobar_usuario(&mut self, usuario_account_id: AccountId)
        {
            let index = self.get_usuario_en_peticiones_del_sistema(usuario_account_id);
            let user = self.peticiones_registro.remove(index.unwrap()); // Unwrap porque ya sé que existe en el vec
            self.usuarios_registados.push(user);
        }

        /// SE AGREGA UN USUARIO A LA COLA DE ESPERA DEL SISTEMA
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un Usuario
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion agrega al usuario recibido a la lista de peticiones de registro del sistema
        /// 
        /// ...
        fn registrar_en_cola_de_sistema(&mut self, user: Usuario) {
            self.peticiones_registro.push(user);
        }

        /// EN CASO DE QUE EL ADMIN ID NO ESTA REGISTRADO LO REGISTRA
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId, un nombre y un dni
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion revisa que el accountId recibido este registrado o en peticiones del sistema, si no se encuntra en ninguna lista
        /// lo registra y valida si esta en la lista de peticiones solo lo valida.
        /// 
        /// ...
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

        /// AGREGA A UN USUARIO A LA COLA DE ESPERA PARA SER VOTANTE O CANDIDATO EN UNA ELECCION
        ///
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un Usuario, un Rol y usize, retorna un Result<(),ErrorSistema>
        /// 
        ///#Funcionalidad 
        /// 
        /// La funcion valida que el usuario no exista en la eleccion recibida y lo agrega a la lista de espera adecuada dependiendo de rol recibido
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan si el usuario ya esta registrado y aprobado en esta eleccion ya sea como votante o como candidato
        /// 
        /// ...
        fn registrar_peticion_eleccion(&mut self, user: Usuario, rol: Rol, eleccion_index: usize) -> Result<(), ErrorSistema>
        {
            self.validar_inexistencia_de_usuario_en_eleccion(user.account_id, eleccion_index)?;

            match rol {
                Rol::Votante => self.elecciones[eleccion_index]
                    .peticiones_votantes
                    .push(user),
                Rol::Candidato => self.elecciones[eleccion_index]
                    .peticiones_candidatos
                    .push(user),
            }

            Ok(())
        }

        ///CREA UNA LISTA DE LAS ELECCIONES EN CURSO DE LA FORMA QUE SE LEE EN LA INTERFAZ
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un time stamp y retorna un Vec<EleccionInterfaz>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion crea un Vec<EleccionInterfaz> usando el la lista de elecciones actuales del sistema
        /// 
        /// #Errores
        /// 
        /// La funcion no maneja errores
        /// 
        /// ...
        fn clonar_elecciones_actuales_a_interfaz(&self, timestamp: u64) -> Vec<EleccionInterfaz>
        {
            let mut vec: Vec<EleccionInterfaz> = Vec::new();

            for i in 0..self.elecciones.len() {
                vec.push(EleccionInterfaz::from_eleccion(
                    self.elecciones[i].get_estado_eleccion(timestamp),
                    self.elecciones[i].clone(),
                    None
                ));
            }

            vec
        }

        ///CREA UNA LISTA DE LAS ELECCIONES TERMINADAS DE LA FORMA QUE SE LEE ENE LA INTERFAZ
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un time stamp y retorna un Vec<EleccionInterfaz>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion crea un Vec<EleccionInterfaz> usando el la lista de elecciones pasadas del sistema
        /// 
        /// #Errores
        /// 
        /// La funcion no maneja errores
        /// 
        /// ...
        fn clonar_elecciones_historicas_a_interfaz(&self) -> Vec<EleccionInterfaz>
        {
            let mut vec: Vec<EleccionInterfaz> = Vec::new();

            for i in 0..self.elecciones_finiquitadas.len() {
                vec.push(EleccionInterfaz::from_eleccion(
                    EstadoEleccion::Finalizada,
                    self.elecciones_finiquitadas[i].clone(),
                    Some(self.elecciones_finiquitadas[i].votos.clone())
                ));
            }

            vec
        }

        ///APRUEBA UN CANDIDATO PARA PARTICIPAR EN UNA ELECCION
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe dos usize 
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion usa los usize proporcionados para buscar un candidato en la lista de candidatos pendientes y una eleccion en la lista de elecciones actuales,
        /// lo elimina de esa lista y lo agrega a la lista de candidatos aprobados en la eleccion, tambien lo crea en la lista de votosxcandidato para que se lo pueda 
        /// votar 
        /// 
        /// #Errores
        /// 
        /// La funcion no maneja errores
        /// 
        /// ...
        fn aprobar_candidato(&mut self, candidato_index: usize, eleccion_index: usize) 
        {
            let e = &mut self.elecciones[eleccion_index];

            let candidato = e.peticiones_candidatos.remove(candidato_index);
            e.candidatos_aprobados.push(candidato.clone());

            let candidato_votos = CandidatoVotos::new(candidato.nombre, candidato.dni);
            e.votos.push(candidato_votos);
        }

        ///APRUEBA UN VOTANTE PARA PARTICIPAR EN UNA ELECCION
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe dos usize 
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion usa los usize proporcionados para buscar un votante en la lista de votantes pendientes y una eleccion en la lista de elecciones actuales,
        /// lo elimina de esa lista y lo agrega a la lista de votantes aprobados en la eleccion. 
        /// 
        /// #Errores
        /// 
        /// La funcion no maneja errores
        /// 
        /// ...
        fn aprobar_votante(&mut self, votante_index: usize, eleccion_index: usize) 
        {
            let e = &mut self.elecciones[eleccion_index];

            let votante = e.peticiones_votantes.remove(votante_index);
            e.votantes_aprobados.push(votante.clone());
        }

        
        /// SE REGISTRA UN VOTO DE UN VOTANTE VALIDO A UN CANDIDATO VALIDO
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe dos usize, retorna un Result<(),ErrorSistema>  
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion le permite a un votante valido votar por un candidato valido, luego de emitir el voto el votante es agregado a la lista de votantes que ya votaron
        /// de la eleccion
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan cuando el votante no es parte de los votantes habilitados a la eleccion o si ya se alcanzo el numero de votos maximos para un candidato
        /// 
        /// ...
        fn registrar_voto_a_candidato(&mut self, candidato_index: usize, eleccion_index: usize) -> Result<(), ErrorSistema>
        {
            if let Some(num) = self.elecciones[eleccion_index].votos[candidato_index].votos_recaudados.checked_add(1) { 
                self.elecciones[eleccion_index].votos[candidato_index].votos_recaudados = num;
            } else {
                return Err(ErrorSistema::RepresentacionLimiteAlcanzada) 
            }

            if let Some(votante_index) = self.elecciones[eleccion_index].votantes_aprobados.iter().position(|v| v.account_id == Self::env().caller()) {
                let votante = self.elecciones[eleccion_index].votantes_aprobados[votante_index].clone();
                self.elecciones[eleccion_index].votantes_votados.push(votante);
            } else {
                return Err( ErrorSistema::ErrorDeEleccion(ErrorEleccion::VotanteNoExiste))
            }

            Ok(())
        }
    



        //////////////////////////////////////// VALIDACIONES ////////////////////////////////////////

        //////////////////// SISTEMA ////////////////////

        ///CONFIRMA LOS PERMISOS DEL ADMIN
        /// 
        /// #Uso
        /// La funcion es de uso interno del sistema, recibe un AccountId y un String y retorna un Result<(),ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion valida que el AccountId recibido sea el admin del sistema
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan si el AccountId recibido no es el admin, en  ese caso se devuelve el string recibido dentro del ErrorSistema
        /// 
        /// ...
        fn validar_permisos(&self, caller_id: AccountId) -> Result<(), ErrorSistema> {
            if !self.es_admin(caller_id) { return Err( ErrorSistema::NoSePoseenPermisos); }
            Ok(())
        }

        ///INFORMA SI EL ID PROPORCIONADO ES EL ADMIN
        fn es_admin(&self, caller_id: AccountId) -> bool { caller_id == self.admin_id }

        /// VALIDA SI EL USUARIO ES EL ADMIN O ESTA APROBADO
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId y retorna un Result<(),ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion valida que el AccountId recibido sea de el admin o de un usuario aprobado en el sistema
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan si el AccountId recibido es de un usuario que auno no fue aprobado en el sistema o
        /// de un usuario que aun no se registro en el sistema
        /// 
        /// ...
        fn validar_caller_como_admin_o_usuario_aprobado(&self, caller_id: AccountId) -> Result<(), ErrorSistema>
        {
            if self.es_admin(caller_id) { return Ok(()) }

            match self.validar_usuario(caller_id) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }

        /// VALIDA QUE EL USUARIO ESTE APROBADO EN EL SISTEMA, DE SER EL CASO LO DEVUELVE
        /// 
        /// #Uso 
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId y un string y retorna un Result<(),ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion valida que el AccountId recibido sea el de un usuario aprobado en el sistema
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan si el AccountId proporcionado pertence a el adminm a un usuario no aprobado en el sistema o 
        /// a un usuario no registrado en el sistema
        /// 
        /// ...
        fn validar_caller_como_usuario_aprobado(&self, caller_id: AccountId) -> Result<Usuario, ErrorSistema>
        {
            if self.es_admin(caller_id) { return Err( ErrorSistema::AccionUnicaDeUsuarios); }

            self.validar_usuario(caller_id)
        }

        /// VALIDA QUE UN USUARIO ESTE REGISTRADO EN EL SISTEMA
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId y retorna un Result<Usuario,ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion usa el AccountId recibido para buscar la posicion de un Usuario en el sistema y devuelve una copia de ese usuario
        /// 
        /// #Errores
        /// 
        /// Los casos de error de esta funcion son los mismo que los de la fn validar_usuario_en_sistema
        fn validar_usuario(&self, caller_id: AccountId) -> Result<Usuario, ErrorSistema>
        {
            let index = self.validar_usuario_en_sistema(caller_id)?;
            Ok(self.usuarios_registados[index].clone())
        }

        ///CONFIRMA LA NO EXISTENCIA DE UN USUARIO EN EL SISTEMA
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId y retorna un Result<(),ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion revisa en todas las lista del sistema para asegurarse de que el AccountId proporcionado no exista en ninguna
        /// 
        /// #Erroes
        /// 
        /// Los casos de error se dan cuando el AccountId se encuentra en alguna de las lista
        /// 
        /// ...
        fn consultar_inexistencia_usuario_en_sistema(&self, caller_id: AccountId) -> Result<(), ErrorSistema>
        {
            if self.es_admin(caller_id) { return Err( ErrorSistema::AdminYaRegistrado); }

            if self.existe_usuario_en_peticiones_del_sistema(caller_id) { return Err( ErrorSistema::UsuarioYaRegistradoEnPeticiones); }
            if self.existe_usuario_registrado_en_sistema(caller_id) { return Err( ErrorSistema::UsuarioYaRegistrado); }

            Ok(())
        }

        ///BUSCA UN USUARIO EN LA COLA DE PETICIONES DE REGISTRO
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId y retorna un Result<(),ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion checkea que un AccountId este dentro de la lista de peticiones de registro del sistema
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan cuando el AccountId recibido ya forma parte de la lista de usuarios aprobados en el sistema o 
        /// cuando el usuario no existe en el sistema
        /// 
        /// ...
        fn consultar_peticion_sistema(&self, user_id: AccountId) -> Result<(), ErrorSistema>
        {
            if self.existe_usuario_en_peticiones_del_sistema(user_id) { return Ok(()) }

            match self.existe_usuario_registrado_en_sistema(user_id) { 
                true  => Err( ErrorSistema::UsuarioYaRegistrado),
                false => Err( ErrorSistema::NoExisteUsuario)
            }
        }

        ///INFORMA SI UN DETERMINADO USUARIO EXISTE EN LA COLA DE PETICIONES DE REGISTRO
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId y retorna un bool
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion retorna true cuando un usuario esta en la lista de peticiones de registro del sistema y false cuando no esta
        /// 
        /// ...
        fn existe_usuario_en_peticiones_del_sistema(&self, caller_id: AccountId) -> bool {
            self.peticiones_registro
                .iter()
                .any(|u| u.account_id == caller_id)
        }

        ///INFORMA SI UN DETERMINADO USUARIO EXISTE EN LA COLA DE USUARIOS REGISTRADOS EN EL SISTEMA
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId y retorna un bool
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion retorna true cuando un usuario esta en la lista de usuarios registrados del sistema y false cuando no esta
        fn existe_usuario_registrado_en_sistema(&self, caller_id: AccountId) -> bool {
            self.usuarios_registados
                .iter()
                .any(|u| u.account_id == caller_id)
        }

        /// SE BUSCA Y RETORNA LA POSICION EN LA COLA DE UN USUARIO REGISTRADO ESPECIFICO EN CASO DE EXISTIR
        /// 
        /// #Use
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId y retorna un Option<usize>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion busca la posicion del AccountId recibido en la lista de usuarios registrados en el sistema y si el accountId existe retorna su posicion
        /// en caso contrario retorna None
        /// 
        /// ...
        fn get_usuario_registrado_en_sistema(&self, user_id: AccountId) -> Option<usize>
        {
            for i in 0 .. self.usuarios_registados.len() {
                if self.usuarios_registados[i].account_id == user_id { return Some(i); }
            }

            None
        }

        /// SE BUSCA Y RETORNA LA POSICION EN LA COLA DE UN USUARIO NO REGISTRADO ESPECIFICO EN CASO DE EXISTIR
        /// 
        /// #Use
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId y retorna un Option<usize>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion busca la posicion del AccountId recibido en la lista de usuarios pendientes en el sistema y si el accountId existe retorna su posicion
        /// en caso contrario retorna None
        /// 
        /// ...
        fn get_usuario_en_peticiones_del_sistema(&self, user_id: AccountId) -> Option<usize>
        {
            for i in 0 .. self.peticiones_registro.len() {
                if self.peticiones_registro[i].account_id == user_id { return Some(i); }
            }

            None
        }

        ///VALIDA LA EXISTENCIA DE UN USUARIO EN EL SISTEMA
        /// 
        /// #Use
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId y retorna un Result<usize,ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion busca la posicion en la lista de usuarios registrados en el sistema del AccountId recibido y la retorna
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan cuando el AccountId aun no fue aprobado en el sistema o cuando el AccountId no existe en el sistema
        /// 
        /// ...
        fn validar_usuario_en_sistema(&self, caller_id: AccountId) -> Result<usize, ErrorSistema>
        {
            if let Some(index) = self.get_usuario_registrado_en_sistema(caller_id) { return Ok(index); }

            match self.existe_usuario_en_peticiones_del_sistema(caller_id) {
                true =>  Err( ErrorSistema::UsuarioNoAprobado),
                false => Err( ErrorSistema::NoExisteUsuario)
            }
        }

        //////////////////// ELECCIONES ////////////////////

        ///INCREMENTA EN 1 EL NUMERO DE IDS DE ELECCIONES, EN CASO DE DESBORDE LO INFORMA
        /// 
        /// #Errores
        /// 
        /// El caso de error de la funcion se da cuando se alcanzo el numero maximo de representacion con un u64
        /// 
        /// ...
        fn check_add_elecciones_id(&mut self) -> Result<(), ErrorSistema> 
        {
            if let Some(resultado) = self.elecciones_conteo_id.checked_add(1) {
                self.elecciones_conteo_id = resultado;
                Ok(())
            } else {
                Err(ErrorSistema::RepresentacionLimiteAlcanzada)
            }
        }

        ///DEVUELVE LA POSICION EN LA LISTA DE ELECCIONES DE UNA ELECCION ESPECIFICA
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe dos u64 y un EstadoEleccion y retorna un Result<usize,ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion busca la posicion en la lista de elecciones actuales de el id de eleccion recibido (U64) y si esta en el EstadoEleccion deseado los retorna
        /// 
        /// #Errores
        /// 
        /// Los casos de error de esta funcion son los de las funciones fn existe_eleccion y fn consultar_estado_eleccion
        /// 
        /// ...
        fn validar_eleccion(&mut self,eleccion_id: u64,estado_buscado: EstadoEleccion,timestamp: u64,) -> Result<usize, ErrorSistema> 
        {
            let eleccion_index = self.existe_eleccion(eleccion_id)?;
            self.consultar_estado_eleccion(estado_buscado, eleccion_index, timestamp)?;

            Ok(eleccion_index)
        }

        ///VALIDA LA EXISTENCIA DE UNA ELECCION Y DEVUELVE SU POSICION EN EL VEC
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un u64 y retorna un Result<usize,ErrorSistema> 
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion checkea que el u64 pertenezca a los id de eleccion posibles en el sistema y retorna la posicion de la eleccion en la lista de elecciones
        /// del sistema
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan si el u64 es mayor al el valor de eleccion_conteo_id del sistema, si este valor es 0 o si el id de la eleccion pertenece a una 
        /// eleccion ya finalizada
        /// 
        /// ...
        fn existe_eleccion(&self, eleccion_id: u64) -> Result<usize, ErrorSistema>
        {
            if eleccion_id > self.elecciones_conteo_id.saturating_sub(1) || self.elecciones_conteo_id == 0 { 
                Err(ErrorSistema::ErrorDeEleccion(ErrorEleccion::NoExisteEleccion))
            } else {
                self.get_index_eleccion(eleccion_id)
            }
        }

        ///SE BUSCA LA POSICION DE UNA ELECCION EN LAS EN PROGRESO
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un u64 y retorna un Result<usize,ErrorSistema> 
        /// 
        /// #Funcionalidad
        /// 
        /// la eleccion busca entre todas las elecciones actuales del sistema alguna con el mismo id que el recibido, si la encuentra devuelve la posicion de la misma
        /// 
        /// #Errores
        /// 
        /// El unico caso de error se da si la eleccion buscada no esta en la lista
        fn get_index_eleccion(&self, eleccion_id: u64) -> Result<usize, ErrorSistema>
        {
            for i in 0 .. self.elecciones.len()
            {
                if self.elecciones[i].eleccion_id == eleccion_id 
                {
                    return Ok(i);
                }
            }

            Err(ErrorSistema::ErrorDeEleccion(ErrorEleccion::EleccionFinalizada))
        }

        ///SE CONSULTA SI LA ELECCION ESTA EN EL ESTADO DESEADO EN CASO CONTRARIO SE INFORMA EL ESTADO ACTUAL DE LA ELECCION
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe dos u64 y un usize, retorna un Result<(),ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion revisa si el id de eleccion proporcionado portenece a una eleccion en el estado recibido
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan cuando el estado de la eleccion no es el buscado
        /// 
        /// ...
        fn consultar_estado_eleccion(&mut self, estado_buscado: EstadoEleccion, eleccion_index: usize, timestamp: u64) -> Result<(), ErrorSistema>
        {
            let estado_eleccion = self.elecciones[eleccion_index].get_estado_eleccion(timestamp);

            if estado_buscado == estado_eleccion {
                return Ok(());
            }

            match estado_eleccion {
                EstadoEleccion::PeriodoInscripcion => Err(ErrorSistema::ErrorDeEleccion(ErrorEleccion::EleccionEnProcesoInscripcion)),
                EstadoEleccion::PeriodoVotacion => Err(ErrorSistema::ErrorDeEleccion(ErrorEleccion::EleccionEnProcesoVotacion)),
                EstadoEleccion::Cerrada => Err(ErrorSistema::ErrorDeEleccion(ErrorEleccion::EleccionCerrada)),
                EstadoEleccion::Finalizada => Err(ErrorSistema::ErrorDeEleccion(ErrorEleccion::EleccionFinalizada)),
            }
        }

        // VALIDA QUE UN USUARIO NO EXISTA EN NINGUNA COLA DE ESPERA O LISTA DE VOTANTES/CANDIDATOS 
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId y un usize y retorna un Result<(),ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion valida que un usuario no exista en ninguna lista de una eleccion
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan cuando el accountId ingresado pertenece a alguna lista dentro de la eleccion ya sea de pendientes a confirmacion o de usuarios confirmados
        /// 
        /// ...      
        fn validar_inexistencia_de_usuario_en_eleccion(&self, caller_id: AccountId, eleccion_index: usize) -> Result<(), ErrorSistema>
        {
            let e = &self.elecciones[eleccion_index];

            if e.peticiones_votantes.iter().any(|p| p.account_id == caller_id) {
                Err(ErrorSistema::ErrorDeEleccion(ErrorEleccion::VotanteEnPendiente))
            } else if e.votantes_aprobados.iter().any(|p| p.account_id == caller_id) {
                Err(ErrorSistema::ErrorDeEleccion(ErrorEleccion::VotanteActualmenteAprobado))
            } else if e.peticiones_candidatos.iter().any(|p| p.account_id == caller_id) {
                Err(ErrorSistema::ErrorDeEleccion(ErrorEleccion::CandidatoEnPendiente))
            } else if e.candidatos_aprobados.iter().any(|p| p.account_id == caller_id) {
                Err(ErrorSistema::ErrorDeEleccion(ErrorEleccion::CandidatoActualmenteAprobado))
            } else {
                Ok(())
            }
        }

        /// VALIDA QUE UN USUARIO ESTE VALIDADO COMO VOTANTE
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un AccountId y un usize y retorna un Result<(),ErrorSistema>
        /// 
        /// #Funcinalidad
        /// 
        /// La funcion valida que el AccountId recibido pertenezca a la eleccion que se encuentra en la posicion recibida y este aprobado como votante en ella
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan cuando el accountId recibido no pertenece a un votante aprobado en la eleccion
        /// 
        /// ...
        fn validar_votante_aprobado_en_eleccion(&self, votante_id: AccountId, eleccion_index: usize) -> Result<(), ErrorSistema>
        {
            if self.elecciones[eleccion_index].votantes_aprobados.iter().any(|v| v.account_id == votante_id) { return Ok(()); }


            return match self.elecciones[eleccion_index].peticiones_votantes.iter().any(|v| v.account_id == votante_id) {
                true  => Err( ErrorSistema::ErrorDeEleccion(ErrorEleccion::VotanteNoAprobado)),
                false => Err( ErrorSistema::ErrorDeEleccion(ErrorEleccion::VotanteNoExiste))
            };
        }

        ///EL SISTEMA VALIDA QUE EL CANDIDATO A APROBAR ESTE EN LA LISTA DE CANDIDATOS PENDIENTES
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un String y un usize y retorna un Result<usize,ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion busca el string recibido en la eleccion recibida y retorna la pocision del candidato con su dni= al String recibido
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan si el String pretenece a un candidato ya aprobado o no pertenece a ningun candidato
        /// 
        /// ...
        fn validar_candidato_en_pendientes(&self, candidato_dni: String, eleccion_index: usize) -> Result<usize, ErrorSistema>
        {
            if let Some(index) = self.get_candidato_pendiente(candidato_dni.clone(), eleccion_index) { return Ok(index) }

            match self.get_candidato_aprobado(candidato_dni, eleccion_index).is_some() {
                true  => Err( ErrorSistema::ErrorDeEleccion(ErrorEleccion::CandidatoActualmenteAprobado)),
                false => Err( ErrorSistema::ErrorDeEleccion(ErrorEleccion::CandidatoNoExiste)),
            }
        }

        ///EL SISTEMA VALIDA QUE EL VOTANTE A APROBAR ESTE EN LA LISTA DE VOTANTES PENDIENTES
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un String y un usize y retorna un Result<usize,ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion busca el string recibido en la eleccion recibida y retorna la pocision del votante con su dni= al String recibido
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan si el String pretenece a un votante ya aprobado o no pertenece a ningun votante
        /// 
        /// ...
        fn validar_votante_en_pendientes(&self, votante_dni: String, eleccion_index: usize) -> Result<usize, ErrorSistema>
        {
            if let Some(index) = self.get_votante_pendiente(votante_dni.clone(), eleccion_index) { return Ok(index) }

            match self.get_votante_aprobado(votante_dni, eleccion_index).is_some() {
                true  => Err( ErrorSistema::ErrorDeEleccion(ErrorEleccion::VotanteActualmenteAprobado)),
                false => Err( ErrorSistema::ErrorDeEleccion(ErrorEleccion::VotanteNoExiste)),
            }
        }

    
        ///EL SISTEMA VALIDA QUE EL CANDIDATO SE ENCUENTRE EN LA LISTA DE CANDIDATOS APROBADOS
        /// 
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un String y un usize y retorna un Result<usize,ErrorSistema>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion busca el string recibido en la eleccion recibida y retorna la pocision del candidato con su dni= al String recibido
        /// 
        /// #Errores
        /// 
        /// Los casos de error se dan si el String pretenece a un candidato no aprobado o no pertenece a ningun candidato
        /// 
        /// ...
        fn validar_candidato_aprobado(&self, candidato_dni: String, eleccion_index: usize) -> Result<usize, ErrorSistema>
        {
            if let Some(index) = self.get_candidato_aprobado(candidato_dni.clone(), eleccion_index) { return Ok(index) }

            match self.get_candidato_pendiente(candidato_dni, eleccion_index).is_some() {
                true  => Err( ErrorSistema::ErrorDeEleccion(ErrorEleccion::CandidatoNoAprobado)),
                false => Err( ErrorSistema::ErrorDeEleccion(ErrorEleccion::CandidatoNoExiste)),
            }
        }

        ///EL SISTEMA BUSCA AL CANDIDATO APROBADO EN LA LISTA Y DEVUELVE SU POSICION EN EL VECTOR
        ///
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un String y un usize y retorna un Option<usize>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion busca en la lista de candidatos aprobados de una eleccion uno que tenga el dni = al string recibido y retorna su posicion,
        /// si no lo encuntra retorna un None
        /// 
        /// ...
        fn get_candidato_aprobado(&self, candidato_dni: String, eleccion_index: usize) -> Option<usize> 
        {
            for i in 0 .. self.elecciones[eleccion_index].candidatos_aprobados.len() {
                if self.elecciones[eleccion_index].candidatos_aprobados[i].dni == candidato_dni { return Some(i); }
            }

            None
        }

        ///EL SISTEMA BUSCA AL VOTANTE APROBADO EN LA LISTA Y DEVUELVE SU POSICION EN EL VECTOR
        ///
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un String y un usize y retorna un Option<usize>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion busca en la lista de votantes aprobados de una eleccion uno que tenga el dni = al string recibido y retorna su posicion,
        /// si no lo encuntra retorna un None
        /// 
        /// ...
        fn get_votante_aprobado(&self, votante_dni: String, eleccion_index: usize) -> Option<usize> 
        {
            for i in 0 .. self.elecciones[eleccion_index].votantes_aprobados.len() {
                if self.elecciones[eleccion_index].votantes_aprobados[i].dni == votante_dni { return Some(i); }
            }

            None
        }

        
        ///EL SISTEMA BUSCA A UN CANDIDATO EN LA LISTA DE CANDIDATOS PENDIENTES Y DEVUELVE SU POSICION EN EL VECTOR
        ///
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un String y un usize y retorna un Option<usize>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion busca en la lista de candidatos pendientes de una eleccion uno que tenga el dni = al string recibido y retorna su posicion,
        /// si no lo encuntra retorna un None
        /// 
        /// ...
        fn get_candidato_pendiente(&self, candidato_dni: String, eleccion_index: usize) -> Option<usize>  
        {
            for i in 0 .. self.elecciones[eleccion_index].peticiones_candidatos.len() {
                if self.elecciones[eleccion_index].peticiones_candidatos[i].dni == candidato_dni { return Some(i); }
            }

            None
        }

        ///EL SISTEMA BUSCA A UN VOTANTE EN LA LISTA DE VOTANTES PENDIENTES Y DEVUELVE SU POSICION EN EL VECTOR
        ///
        /// #Uso
        /// 
        /// La funcion es de uso interno del sistema, recibe un String y un usize y retorna un Option<usize>
        /// 
        /// #Funcionalidad
        /// 
        /// La funcion busca en la lista de votantes pendientes de una eleccion uno que tenga el dni = al string recibido y retorna su posicion,
        /// si no lo encuntra retorna un None
        /// 
        /// ...
        fn get_votante_pendiente(&self, votante_dni: String, eleccion_index: usize) -> Option<usize>  
        {
            for i in 0 .. self.elecciones[eleccion_index].peticiones_votantes.len() {
                if self.elecciones[eleccion_index].peticiones_votantes[i].dni == votante_dni { return Some(i); }
            }

            None
        }
    }



    #[derive(Debug,PartialEq)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ErrorSistema
    {
        UsuarioYaRegistrado,
        AdminYaRegistrado,
        UsuarioYaRegistradoEnPeticiones,
        NoExisteUsuario,
        UsuarioNoAprobado,
        NoSePoseenPermisos,
        AccionUnicaDeUsuarios,
        RepresentacionLimiteAlcanzada,
        FechaInicioInvalida(ErrorFecha),
        FechaCierreInvalida(ErrorFecha),
        FechaInicioPasada,
        FechaCierrePasada,
        FechaCierreAntesInicio,
        EleccionInvalida,
        VotanteYaVoto,
        ResultadosNoDisponibles,
        ErrorDeEleccion(ErrorEleccion),
    }

    impl ToString for ErrorSistema {
        fn to_string(&self) -> String 
        {
            match self {
                ErrorSistema::UsuarioYaRegistrado => "El usuario ya se encuentra registrado y aprobado en el sistema".to_owned(),
                ErrorSistema::AdminYaRegistrado => "Los administradores se registran al momento de instanciar el sistema, ó de delegar su rol.".to_owned(),
                ErrorSistema::UsuarioYaRegistradoEnPeticiones => "El usuario ya se encuentra registrado en la cola de aprobación del sistema, deberá esperar a ser aprobado.".to_owned(),
                ErrorSistema::NoExisteUsuario => "El usuario no existe en el sistema.".to_owned(),
                ErrorSistema::UsuarioNoAprobado => "Usted se encuentra dentro de la cola de peticiones del sistema, debe esperar a ser aceptado.".to_owned(),
                ErrorSistema::NoSePoseenPermisos => "Solo el administrador puede realizar esta accion.".to_owned(),
                ErrorSistema::AccionUnicaDeUsuarios => "Solo los usuarios pueden realizar esta accion.".to_owned(),
                ErrorSistema::RepresentacionLimiteAlcanzada => "La máxima representación del tipo de dato fue alcanzada. Contacte al administrador para mantenimiento urgente.".to_owned(),
                ErrorSistema::FechaInicioInvalida(error) => error.to_string(),
                ErrorSistema::FechaCierreInvalida(error) => error.to_string(),
                ErrorSistema::FechaInicioPasada => "La fecha de incio de la eleccion es anterior al dia actual.".to_owned(),
                ErrorSistema::FechaCierrePasada => "La fecha de cierre de la eleccion es anterior al dia actual.".to_owned(),
                ErrorSistema::FechaCierreAntesInicio => "La fecha de cierre de la eleccion es anterior a la fecha de inicio.".to_owned(),
                ErrorSistema::EleccionInvalida => "La elección ingresada no existe.".to_owned(),
                ErrorSistema::VotanteYaVoto => "El votante ya ha votado.".to_owned(),
                ErrorSistema::ErrorDeEleccion(error) => error.to_string(),
                ErrorSistema::ResultadosNoDisponibles => "Los resultados de la elección no están disponibles.".to_owned(),
            }
        }
    }

    #[derive(Debug,PartialEq)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct ErrorInterfaz {
        error: ErrorSistema,
        texto: String
    }

    impl ErrorInterfaz {
        fn new(error: ErrorSistema) -> Self
        {
            let texto = error.to_string();
            ErrorInterfaz { error, texto }
        }
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
        candidatos_aprobados: Vec<Usuario>,
        resultados: Option<Vec<CandidatoVotos>>
    }

    impl EleccionInterfaz {
        fn new(
            eleccion_id: u64,
            cargo: String,
            fecha_inicio: Fecha,
            fecha_cierre: Fecha,
            estado_eleccion: EstadoEleccion,
            candidatos_aprobados: Vec<Usuario>,
            resultados: Option<Vec<CandidatoVotos>>
        ) -> Self {
            EleccionInterfaz {
                eleccion_id,
                cargo,
                fecha_inicio,
                fecha_cierre,
                estado_eleccion,
                candidatos_aprobados,
                resultados
            }
        }
        ///CREAR UNA ELECCION INTERFAZ A PARTIR DE UNA ELECCION INTERNA DEL SISTEMA
        fn from_eleccion(estado_eleccion: EstadoEleccion, eleccion: Eleccion, resultados: Option<Vec<CandidatoVotos>>) -> EleccionInterfaz {
            EleccionInterfaz::new(
                eleccion.eleccion_id,
                eleccion.cargo,
                eleccion.fecha_inicio_interfaz,
                eleccion.fecha_cierre_interfaz,
                estado_eleccion,
                eleccion.candidatos_aprobados,
                resultados
            )
        }
    }




    #[derive(Clone, Debug,PartialEq)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Eleccion
    {
        eleccion_id: u64, // Número alto de representación para un futuro sustento

        cargo: String, // Decido String en vez de ENUM debido a la inmensa cantidad de cargos posibles, al fin y al cabo, quien se encarga de esto es el administrador electoral

        fecha_inicio: Timestamp, // Dato pedido del profe
        fecha_cierre: Timestamp,

        fecha_inicio_interfaz: Fecha,
        fecha_cierre_interfaz: Fecha,

        votos: Vec<CandidatoVotos>, // No se deben poder getterar hasta que el Timestamp de cierre haya sido alcanzado

        candidatos_aprobados: Vec<Usuario>,
        peticiones_candidatos: Vec<Usuario>,

        votantes_aprobados: Vec<Usuario>,
        peticiones_votantes: Vec<Usuario>,
        votantes_votados: Vec<Usuario>
    }

    impl Eleccion {
        pub fn new(
            eleccion_id: u64,
            cargo: String,
            fecha_inicio: Timestamp,
            fecha_cierre: Timestamp,
            fecha_inicio_interfaz: Fecha,
            fecha_cierre_interfaz: Fecha,
        ) -> Self {
            Eleccion {
                eleccion_id,
                cargo,
                fecha_inicio,
                fecha_cierre,
                fecha_inicio_interfaz,
                fecha_cierre_interfaz,

                votos: Vec::new(),

                candidatos_aprobados: Vec::new(),
                peticiones_candidatos: Vec::new(),
                votantes_aprobados: Vec::new(),
                peticiones_votantes: Vec::new(),
                votantes_votados: Vec::new(),
            }
        }


        ///DEVUELVE UN ESTADOELECCION EN BASE A UN TIMESTAMP RECIBIDO
        fn get_estado_eleccion(&self, timestamp: u64) -> EstadoEleccion
        {
            if self.fecha_inicio > timestamp {
                EstadoEleccion::PeriodoInscripcion
            } else if self.fecha_cierre <= timestamp {
                EstadoEleccion::Cerrada
            } else {
                EstadoEleccion::PeriodoVotacion
            }
        }
        pub fn get_eleccion_votos(&self) -> Vec<CandidatoVotos> {
            self.votos.clone()
        }
        // pub fn get_dimf_votantes_aprobados(&self)-> usize{
        //     self.votantes_aprobados.len()
        // }
        pub fn get_votantes_aprobados(&self) -> Vec<Usuario> {
            self.votantes_aprobados.clone()
        }
        pub fn get_votantes_registrados(&self) -> Vec<Usuario> {
            self.peticiones_votantes.clone()
        }
        pub fn get_id(&self) -> u64 {
            self.eleccion_id
        }
        pub fn get_cargo(&self) -> String {
            self.cargo.clone()
        }
         pub fn set_votantes_registrados(&mut self, usuarios: Vec<Usuario>){
            self.peticiones_votantes = usuarios;
        }

        pub fn set_votantes_aprobados(&mut self, usuarios: Vec<Usuario>){
            self.votantes_aprobados = usuarios;
        }

        pub fn set_votos(&mut self, votos: Vec<CandidatoVotos>){
            self.votos = votos;
        }
    }


    #[derive(Clone, Debug, PartialEq)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum EstadoEleccion { PeriodoInscripcion, PeriodoVotacion, Cerrada, Finalizada }


    #[derive(Debug,PartialEq)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ErrorEleccion
    {
        NoExisteEleccion,

        EleccionEnProcesoInscripcion,
        EleccionEnProcesoVotacion,
        EleccionCerrada,
        EleccionFinalizada,

        CandidatoActualmenteAprobado,
        CandidatoEnPendiente,
        CandidatoNoAprobado,
        CandidatoNoExiste,

        VotanteActualmenteAprobado,
        VotanteEnPendiente,
        VotanteNoAprobado,
        VotanteNoExiste,
    }

    impl ToString for ErrorEleccion {
        fn to_string(&self) -> String
        {
            match self {
                ErrorEleccion::NoExisteEleccion => "La id de elección ingresada no existe.".to_owned(),
                ErrorEleccion::EleccionEnProcesoInscripcion => "La elección ingresada se encuentra en período de inscripción.".to_owned(),
                ErrorEleccion::EleccionEnProcesoVotacion => "La elección ingresada se encuentra en período de votación.".to_owned(),
                ErrorEleccion::EleccionCerrada => "La elección ingresada se encuentra cerrada.".to_owned(),
                ErrorEleccion::EleccionFinalizada => "La eleccion ingresada se encuentra finalizada.".to_owned(),
                ErrorEleccion::CandidatoActualmenteAprobado => "El candidato ingresado ya se encuentra actualmente aprobado.".to_owned(),
                ErrorEleccion::CandidatoEnPendiente => "El candidato ingresado ya se encuentra en la cola de peticiones para candidato y debe esperar a ser aprobado".to_owned(),
                ErrorEleccion::CandidatoNoAprobado => "El candidato ingresado está en espera de aprobación.".to_owned(),
                ErrorEleccion::CandidatoNoExiste => "El candidato ingresado no existe en la elección.".to_owned(),
                ErrorEleccion::VotanteActualmenteAprobado => "El votante ingresado ya se encuentra actualmente aprobado.".to_owned(),
                ErrorEleccion::VotanteEnPendiente => "El votante ingresado ya se encuentra en la cola de peticiones para votante y debe esperar a ser aprobado".to_owned(),
                ErrorEleccion::VotanteNoAprobado => "El votante ingresado no fue aprobado para esta elección, no tendrá permiso para votar.".to_owned(),
                ErrorEleccion::VotanteNoExiste => "El votante ingresado no existe en la elección.".to_owned(),
            }
        }
    }

    //////////////////////////////// VOTOS Y RESULTADOS ////////////////////////////////

    #[derive(Clone, Debug, PartialEq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct CandidatoVotos {
        candidato_nombre: String,
        candidato_dni: String,
        votos_recaudados: u64,
    }

    impl CandidatoVotos {
        pub fn new(candidato_nombre: String, candidato_dni: String) -> Self {
            CandidatoVotos {
                candidato_nombre,
                candidato_dni,
                votos_recaudados: 0,
            }
        }
        pub fn get_votos_recaudados(&self) -> u64 {
            self.votos_recaudados
        }
        pub fn set_votos_recaudados(&mut self, cantidad: u64){
            self.votos_recaudados = cantidad;
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
        dni: String,
    }

    impl Usuario {
        pub fn new(account_id: AccountId, nombre: String, dni: String) -> Self {
            Usuario {
                account_id,
                nombre,
                dni,
            }
        }
    }

    ////////////////////////////// Fecha /////////////////////////////

    #[derive(Clone, Debug,PartialEq)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Fecha { dia: u8, mes: u8, año: u32, hora: u8, min: u8, seg: u8 } // Año como unsigned debido a que sí o sí se tratarán fechas mayores a la actual


    impl Fecha 
    {
        pub fn new(dia: u8, mes: u8, año: u32, hora: u8, min: u8, seg: u8) -> Self
        { 
            Fecha { dia, mes, año, hora, min, seg }
        }
        ///VALIDA QUE LA FECHA INGRESADA EN NUMEROS SEA UNA FECHA CORRECTA Y EXISTENTE, TENIENDO EN CUENTA AÑOS BISIESTOS
        ///
        ///#Uso
        ///Llamar a esta funcion sobre una fecha devuelve un Result con un Ok vacio, en caso de ser valida, o un ErrorFecha en caso de no serlo.
        ///
        ///#Funcionalidad
        ///Chequea que el mes este dentro del rango 1-12, que el dia este dentro de su rango dependiendo del mes actual y si el año es bisiesto o no.
        ///Tambien chequea que la hora, los minutos, y los segundos esten dentro de sus correspondientes rangos.
        ///
        ///#Errores
        ///Devuelve variantes de ErrorFecha, dependiendo del primer error encontrado. Esto quiere decir que si hay mas de un error en una fecha ingresada,
        ///devolvera solo el primer error. Si se valida nuevamente la fecha con el error solucionado, el siguiente error se devolvera, si es que queda alguno.
        ///Los nombres de los errores siguen el patron Dia/Mes/Hora/Min/Seg Invalido, dependiendo de cual es el campo invalido.
        ///El año no puede ser invalido, por lo que no hay error que lo represente.
        fn validar_fecha(&self) -> Result<(), ErrorFecha>
        {
            if self.dia == 0 {
                return Err(ErrorFecha::DiaInvalido);
            }

            if !match self.mes {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => self.dia <= 31,
                4 | 6 | 9 | 11 => self.dia <= 30,
                2 => self.dia <= if self.es_bisiesto() { 29 } else { 28 },
                _ => { return Err(ErrorFecha::MesInvalido); }
            } {
                return Err(ErrorFecha::DiaInvalido);
            }

            if self.hora > 23 {
                return Err(ErrorFecha::HoraInvalida);
            }

            if self.min > 59 {
                return Err(ErrorFecha::MinInvalido);
            }

            if self.seg > 59 {
                return Err(ErrorFecha::SegInvalido);
            }

            Ok(())
        }


        ///DEVUELVE SI EL AÑO DE LA FECHA ES BISIESTO
        ///
        ///#Uso
        ///Llamar a esta funcion sobre una fecha te devuelve un bool, donde true significa que el año si es bisiesto, y false que no
        ///
        ///#Funcionalidad
        ///Para saber si un año es bisiesto, se ve si es divisible por 4 pero a si mismo no divisible por 100,
        ///o si es divisible por 100 y por 400
        fn es_bisiesto(&self) -> bool {
            (self.año % 4 == 0 && self.año % 100 != 0) || (self.año % 100 == 0 && self.año % 400 == 0)
        }


        ///CONVIERTE LA FECHA A UNIX EPOCH TIMESTAMP EN MILISEGUNDOS
        ///
        ///#Uso
        ///Llamar a esta funcion sobre una fecha devuelve un u64 que es la representacion de la fecha en Unix Epoch Timestamp.
        ///Este formato guarda una fecha en forma de los milisegundos pasados desde el primero de enero de 1970 a las 00:00:00 UTC.
        ///
        ///#Funcionalidad
        ///El algoritmo usado fue traducido de uno de los algoritmos presentados en este articulo:
        ///https://blog.reverberate.org/2020/05/12/optimizing-date-algorithms.html
        ///
        ///Todos los datos de la fecha son pasados a u64 previamente a hacer las operaciones para simplificar el codigo siguiente
        ///y evitar multiples conversiones en el medio.
        ///
        ///Se usan las funciones saturating para sumar, restar, dividir, y multiplicar. Estas funciones realizan las operaciones normales,
        ///pero impiden el overflow y el underflow, simplemente limitando los valores al rango valido para u64.
        ///Overflow y underflow solo deberia suceder en casos extremos (años tan altos que la tierra ni existiria)
        ///, o hasta en algunas operaciones es imposible si la fecha fue validada.
        ///Estas funciones fueron usadas para complacer al linter, que mira hasta los casos extremos.
        ///Los casos donde no se obtiene el timestamp correcto, son los años anteriores a 1970, ya que eso seria un timestamp
        ///negativo. En este caso, por las funciones que previenen el overflow y el underflow, todos los años anteriores a 1970 dan un
        ///timestamp de 0, que es suficiente para este sistema.
        ///En caso de necesitar años anteriores a 1970, el algoritmo esta capacitado, y solo se necesitaria cambiar los tipos de u64 a i64.
        ///
        ///El algoritmo en operaciones normales seria el siguiente:
        ///
        ///let año_ajustado = año + 4800;
        ///let febreros = año_ajustado - if mes <= 2 { 1 } else { 0 };
        ///let dias_intercalar = 1 + (febreros / 4) - (febreros / 100) + (febreros / 400);
        ///let dias = 365 * año_ajustado + dias_intercalar + tabla[(mes - 1) as usize] + dia - 1;
        ///((dias - 2472692) * 86400 + hora * 3600 + min * 60 + seg) * 1000
        ///
        ///Este algoritmo calcula la cantidad de febreros y en la variable de dias_intercalar se calculan 
        ///los febreros que tienen dias extras al estar en años bisiesto. Todos estos dias se suman a los dias
        ///normales que suceden en cada año pasado (años * 365), a el dia actual guardado en la fecha, y a la cantidad de dias
        ///que van transcurriendo en el año, segun el mes, usando la tabla ya que estos valores se saben.
        ///Luego de todos estos calculos, se devuelve los dias, horas, minutos, y segundos, todos pasados a segundos y luego pasados
        ///a milisegundos.
        fn to_timestamp(&self) -> u64
        {
            let dia: u64 = self.dia.into();
            let mes: u64 = self.mes.into();
            let año: u64 = self.año.into();
            let seg: u64 = self.seg.into();
            let min: u64 = self.min.into();
            let hora: u64 = self.hora.into();

            let tabla = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

            let año_ajustado = año.saturating_add(4800);
            let febreros = año_ajustado.saturating_sub(if mes <= 2 { 1 } else { 0 });
            let dias_intercalar = 1_u64.saturating_add(febreros.saturating_div(4)).saturating_sub(febreros.saturating_div(100)).saturating_add(febreros.saturating_div(400));
            let dias = año_ajustado.saturating_mul(365).saturating_add(dias_intercalar).saturating_add(tabla[(mes.saturating_sub(1)) as usize]).saturating_add(dia).saturating_sub(1);
            dias.saturating_sub(2472692).saturating_mul(86400).saturating_add(hora.saturating_mul(3600)).saturating_add(min.saturating_mul(60)).saturating_add(seg).saturating_mul(1000)
        }


        ///DEVUELVE SI LA FECHA ES ANTERIOR A LA PASADA POR PARAMETRO
        ///
        ///#Uso
        ///Llamar a esta funcion sobre una fecha, pasando un timestamp, devuelve un bool, el cual señaliza si la fecha es anterior
        ///o igual a la fecha representada por el timestamp pasado. Esto muestra si es una fecha que ya paso cierta otra fecha.
        ///
        ///#Funcionalidad
        ///Se compara el timestamp de la fecha (en milisegundos) con otro timestamp, para ver si el primero es menor o igual al segundo.
        ///Los timestamps estan en formato Unix Epoch Timestamp. Este formato guarda una fecha en forma de los milisegundos pasados desde el 
        ///primero de enero de 1970 a las 00:00:00 UTC.
        fn fecha_pasada(&self, timestamp: u64) -> bool {
            self.to_timestamp() <= timestamp
        }
    }


    #[derive(PartialEq, Debug)] #[ink::scale_derive(Encode, Decode, TypeInfo)] #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ErrorFecha
    {
        DiaInvalido,
        MesInvalido,
        HoraInvalida,
        MinInvalido,
        SegInvalido,
    }

    impl ToString for ErrorFecha {
        fn to_string(&self) -> String {
            match self {
                ErrorFecha::DiaInvalido => "El día ingresado es invalido.".to_owned(),
                ErrorFecha::MesInvalido => "El mes ingresado es invalido.".to_owned(),
                ErrorFecha::HoraInvalida => "La hora ingresada es incorrecta.".to_owned(),
                ErrorFecha::MinInvalido => "El minuto ingresado es incorrecto.".to_owned(),
                ErrorFecha::SegInvalido => "El segundo ingresado es incorrecto.".to_owned()
            }
        }
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
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::AdminYaRegistrado)), sistema.registrarse_en_sistema_priv("julian".to_string(), "12345678".to_string()));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(()),sistema.registrarse_en_sistema("julian".to_string(), "12345678".to_string()));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::UsuarioYaRegistradoEnPeticiones)),sistema.registrarse_en_sistema_priv("julian".to_string(), "12345678".to_string()));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::NoSePoseenPermisos)),sistema.aprobar_usuario_sistema_priv(accounts.bob));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            assert_eq!(Ok(()),sistema.aprobar_usuario_sistema_priv(accounts.bob));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::UsuarioYaRegistrado)),sistema.registrarse_en_sistema_priv("julian".to_string(), "12345678".to_string()));
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
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::NoSePoseenPermisos)),sistema.get_peticiones_de_registro_sistema_priv());
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
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::NoSePoseenPermisos)),sistema.delegar_admin_priv(accounts.bob, "bob".to_string(), "12345".to_string()));
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
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:1,mes:1,año:2000,hora:00,min:00,seg:00}.to_timestamp());
            let mut sistema=SistemaVotacion::new("tobias".to_string(), "43107333".to_string());
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::FechaInicioInvalida(ErrorFecha::DiaInvalido))),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 0, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::FechaInicioInvalida(ErrorFecha::MesInvalido))),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 1, mes: 14, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::FechaInicioInvalida(ErrorFecha::HoraInvalida))),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 1, mes: 10, año: 2001, hora: 60, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::FechaInicioInvalida(ErrorFecha::MinInvalido))),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 1, mes: 10, año: 2001, hora: 20, min: 70, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::FechaInicioInvalida(ErrorFecha::SegInvalido))),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 1, mes: 10, año: 2001, hora: 20, min: 30, seg: 99 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::FechaCierreInvalida(ErrorFecha::DiaInvalido))),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 1, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 32, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::FechaCierreInvalida(ErrorFecha::MesInvalido))),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 1, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 14, año: 2001, hora: 20, min: 30, seg: 00 }));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::FechaCierreInvalida(ErrorFecha::HoraInvalida))),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 1, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 60, min: 30, seg: 00 }));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::FechaCierreInvalida(ErrorFecha::MinInvalido))),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 1, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 70, seg: 00 }));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::FechaCierreInvalida(ErrorFecha::SegInvalido))),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 1, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 99 }));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::FechaInicioPasada)),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 1, mes: 1, año: 1600, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::FechaCierreAntesInicio)),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 1, mes: 1, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2000, hora: 20, min: 30, seg: 00 }));
            assert_eq!(Ok(()),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 12, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::NoSePoseenPermisos)),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 12, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
            sistema.elecciones_conteo_id= 18446744073709551615;
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::RepresentacionLimiteAlcanzada)),sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 12, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }));
        }

        #[allow(unused)]
        #[ink::test]
        fn test_get_elecciones()
        {
            let accounts=ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:1,mes:1,año:2000,hora:00,min:00,seg:00}.to_timestamp());
            let mut sistema=SistemaVotacion::new("tobias".to_string(), "43107333".to_string());
            sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 12, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 });
            assert_eq!(Ok(sistema.clonar_elecciones_actuales_a_interfaz(0)),sistema.get_elecciones_actuales_priv());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            sistema.registrarse_en_sistema_priv("bob".to_string(), "12345".to_string());
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::UsuarioNoAprobado)),sistema.get_elecciones_actuales_priv());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            sistema.aprobar_usuario_sistema_priv(accounts.bob);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(sistema.clonar_elecciones_actuales_a_interfaz(0)),sistema.get_elecciones_actuales_priv());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::NoExisteUsuario)),sistema.get_elecciones_actuales_priv());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:13,mes:10,año:2001,hora:21,min:00,seg:00}.to_timestamp());
            sistema.finalizar_y_contar_eleccion_priv(0);
            assert_eq!(Ok(sistema.clonar_elecciones_historicas_a_interfaz()),sistema.get_elecciones_historial_priv());
            assert_eq!(sistema.elecciones_finiquitadas,sistema.get_elecciones_finiquitadas_priv());
            
        }

        #[allow(unused)]
        #[ink::test]
        fn test_registrarse_eleccion()
        {
            let accounts=ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:1,mes:1,año:2000,hora:00,min:00,seg:00}.to_timestamp());
            let mut sistema=SistemaVotacion::new("tobias".to_string(), "43107333".to_string());
            sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 12, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 });
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::AccionUnicaDeUsuarios)),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Candidato));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:1,mes:1,año:2000,hora:00,min:00,seg:00}.to_timestamp());
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::NoExisteUsuario)),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Candidato));
            sistema.registrarse_en_sistema_priv("bob".to_string(), "12345".to_string());
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::UsuarioNoAprobado)),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Candidato));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            sistema.aprobar_usuario_sistema(accounts.bob);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Ok(()),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Candidato));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::NoSePoseenPermisos)),sistema.get_candidatos_pendientes_priv(0));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:1,mes:1,año:2000,hora:00,min:00,seg:00}.to_timestamp());
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::NoExisteUsuario)),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Votante));
            sistema.registrarse_en_sistema_priv("alice".to_string(), "11111".to_string());
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::UsuarioNoAprobado)),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Candidato));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            assert_eq!(Ok(vec![Usuario::new(accounts.bob,"bob".to_string(),"12345".to_string())]),sistema.get_candidatos_pendientes_priv(0));
            sistema.aprobar_usuario_sistema(accounts.alice);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::ErrorDeEleccion(ErrorEleccion::NoExisteEleccion))),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id, Rol::Votante));
            assert_eq!(Ok(()),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Votante));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::NoSePoseenPermisos)),sistema.get_votantes_pendientes_priv(0));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::ErrorDeEleccion(ErrorEleccion::VotanteEnPendiente))),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Votante));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            assert_eq!(Ok(vec![Usuario::new(accounts.alice,"alice".to_string(),"11111".to_string())]),sistema.get_votantes_pendientes_priv(0));
        }

        #[allow(unused)]
        #[ink::test]
        fn test_aprobar_votante_candidato_en_eleccion()
        {
            let accounts=ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:1,mes:1,año:2000,hora:00,min:00,seg:00}.to_timestamp());
            let mut sistema=SistemaVotacion::new("tobias".to_string(), "43107333".to_string());
            sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 12, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 });
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:1,mes:1,año:2000,hora:00,min:00,seg:00}.to_timestamp());
            sistema.registrarse_en_sistema_priv("bob".to_string(), "12345".to_string());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            sistema.aprobar_usuario_sistema(accounts.bob);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Candidato);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            assert_eq!(Ok(()),sistema.aprobar_candidato_eleccion(sistema.elecciones_conteo_id-1, "12345".to_owned()));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::ErrorDeEleccion(ErrorEleccion::CandidatoActualmenteAprobado))),sistema.aprobar_candidato_eleccion(sistema.elecciones_conteo_id-1, "12345".to_owned()));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::ErrorDeEleccion(ErrorEleccion::CandidatoActualmenteAprobado))),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Votante));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:1,mes:1,año:2000,hora:00,min:00,seg:00}.to_timestamp());
            sistema.registrarse_en_sistema_priv("alice".to_string(), "11111".to_string());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            sistema.aprobar_usuario_sistema(accounts.alice);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Votante);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            assert_eq!(Ok(()),sistema.aprobar_votante_eleccion(sistema.elecciones_conteo_id-1, "11111".to_owned()));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::ErrorDeEleccion(ErrorEleccion::VotanteActualmenteAprobado))),sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Votante));
            
        }

        #[allow(unused)]
        #[ink::test]
        fn test_votar_finalizar_eleccion()
        {
            let accounts=ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:1,mes:1,año:2000,hora:00,min:00,seg:00}.to_timestamp());
            let mut sistema=SistemaVotacion::new("tobias".to_string(), "43107333".to_string());
            sistema.crear_nueva_eleccion_priv("Emperador".to_string(), Fecha { dia: 12, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 }, Fecha { dia: 13, mes: 10, año: 2001, hora: 20, min: 30, seg: 00 });
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:1,mes:1,año:2000,hora:00,min:00,seg:00}.to_timestamp());
            sistema.registrarse_en_sistema_priv("bob".to_string(), "12345".to_string());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            sistema.aprobar_usuario_sistema(accounts.bob);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Candidato);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            sistema.aprobar_candidato_eleccion(sistema.elecciones_conteo_id-1, "12345".to_owned());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:1,mes:1,año:2000,hora:00,min:00,seg:00}.to_timestamp());
            sistema.registrarse_en_sistema_priv("alice".to_string(), "11111".to_string());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            sistema.aprobar_usuario_sistema(accounts.alice);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            sistema.registrarse_a_eleccion_priv(sistema.elecciones_conteo_id-1, Rol::Votante);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            sistema.aprobar_votante_eleccion(sistema.elecciones_conteo_id-1, "11111".to_owned());
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::ErrorDeEleccion(ErrorEleccion::EleccionEnProcesoInscripcion))),sistema.votar_eleccion_priv(sistema.elecciones_conteo_id-1, "12345".to_owned()));
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:12,mes:10,año:2001,hora:21,min:00,seg:00}.to_timestamp());
            assert_eq!(Ok(()),sistema.votar_eleccion_priv(sistema.elecciones_conteo_id-1, "12345".to_owned()));
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::VotanteYaVoto)),sistema.votar_eleccion_priv(sistema.elecciones_conteo_id-1, "12345".to_owned()));
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:13,mes:10,año:2001,hora:21,min:00,seg:00}.to_timestamp());
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::ErrorDeEleccion(ErrorEleccion::EleccionCerrada))),sistema.votar_eleccion_priv(sistema.elecciones_conteo_id-1, "12345".to_owned()));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:12,mes:10,año:2001,hora:21,min:00,seg:00}.to_timestamp());
            assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::ErrorDeEleccion(ErrorEleccion::VotanteNoExiste))),sistema.votar_eleccion_priv(sistema.elecciones_conteo_id-1, "12345".to_owned()));
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(Fecha{dia:13,mes:10,año:2001,hora:21,min:00,seg:00}.to_timestamp());
            assert_eq!(Ok(CandidatoVotos{candidato_nombre:"bob".to_string(), candidato_dni:"12345".to_string(), votos_recaudados:1}),sistema.finalizar_y_contar_eleccion_priv(0));
            // assert_eq!(Ok(vec![Usuario::new(accounts.alice,"alice".to_string(),"11111".to_string())]),sistema.get_elecciones_terminadas_x(0));
            // assert_eq!(Err(ErrorInterfaz::new(ErrorSistema::EleccionInvalida)),sistema.get_elecciones_terminadas_x(4));
        }
    }

}