<!DOCTYPE html>
<html lang="es">
  <head align="center" >
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  </head>
  <body >
    <h1 >Sistema de Votación Electrónica con Blockchain</h1>
    <h2>Descripción del Proyecto</h2>
    <p>
      Este proyecto desarrolla un sistema de votación electrónica seguro y
      transparente utilizando tecnología blockchain. El sistema permite la
      participación de diferentes roles de usuario y asegura que los resultados
      de la votación se mantengan confidenciales hasta el cierre oficial de la
      elección por parte del Administrador Electoral.
    </p>
    <h2>Detalles del Sistema</h2>
    <h3>Creación de Elección</h3>
    <p>
      El flujo del sistema comienza con la creación de una elección por parte
      del Administrador Electoral. La elección puede ser para una posición de
      cargo específica, como presidente de una organización. El Administrador
      Electoral configura los detalles de la elección, incluyendo la fecha de
      inicio y cierre de la votación.
    </p>
    <h3>Registro de Usuarios</h3>
    <p>
      Una vez creada la elección, los usuarios pueden registrarse en el sistema,
      eligiendo uno de los dos roles disponibles: candidato o votante. Un
      usuario no puede registrarse en ambos roles simultáneamente.
    </p>
    <h4>Registro de Candidatos</h4>
    <p>
      Los usuarios que desean postularse para la posición de presidente se
      registran como candidatos. Su registro incluye información relevante que
      será verificada y aprobada por el Administrador Electoral.
    </p>
    <h4>Registro de Votantes</h4>
    <p>
      Los usuarios que desean participar en la elección como votantes se
      registran proporcionando su información personal. Esta información será
      verificada y aprobada por el Administrador Electoral para asegurar que
      solo los votantes autorizados puedan participar en la elección.
    </p>
    <h3>Roles de Usuario</h3>
    <p>El sistema contempla tres roles principales:</p>
    <ul>
      <li>
        <strong>Administrador Electoral:</strong> Responsable de configurar las
        elecciones, verificar los registros de votantes y candidatos, generar
        reportes y gestionar el sistema en su totalidad. Tiene la autoridad para
        aprobar o rechazar registros y configurar los parámetros de cada
        elección.
      </li>
      <li>
        <strong>Candidato:</strong> Los individuos que se postulan para la
        posición de presidente. Pueden registrarse y presentar su candidatura
        para ser votados por los electores.
      </li>
      <li>
        <strong>Votante:</strong> Los usuarios que emitirán su voto en la
        elección. Cada votante registrado y aprobado tendrá la capacidad de
        votar de manera segura y anónima.
      </li>
    </ul>
    <h3>Proceso de Votación</h3>
    <ul>
      <li>
        <strong>Inicio de la Elección:</strong> La votación comienza una vez
        alcanzada la fecha de inicio configurada por el Administrador Electoral.
      </li>
      <li>
        <strong>Fin de la Elección:</strong> La votación se cierra al llegar a
        la fecha de fin configurada, pero los resultados no son visibles hasta
        que esto suceda.
      </li>
      <li>
        <strong>Conteo y Publicación de Resultados:</strong> Los votos serán
        contados y los resultados serán publicados de manera transparente,
        mostrando el número de votos recibidos por cada candidato y los
        resultados finales.
      </li>
    </ul>
    <h3>Ocultación de Resultados</h3>
    <p>
      Para garantizar que los resultados no sean visibles hasta el cierre de la
      elección y asegurar el anonimato de los votos, no se podrá conocer o
      calcular la votación hasta que la elección haya concluido.
    </p>
    <h3>Generación de Reportes</h3>
    <p>
      El sistema genera varios reportes desde otro contrato con los permisos
      pertinentes:
    </p>
    <ul>
      <li>
        <strong>Reporte de Registro de Votantes:</strong> Detalla los votantes
        registrados y aprobados para una determinada elección.
      </li>
      <li>
        <strong>Reporte de Participación:</strong> Indica la cantidad de votos
        emitidos y el porcentaje de participación, una vez cerrada la elección.
      </li>
      <li>
        <strong>Reporte de Resultado:</strong> Muestra el número de votos
        recibidos por cada candidato y los resultados finales, una vez cerrada
        la elección. Este reporte muestra de manera descendente los votos, donde
        el primer candidato es el ganador de la elección.
      </li>
    </ul>
    <h2>Requisitos</h2>
    <ul>
      <li>Cobertura del 85% en ambos contratos.</li>
      <li>Todo el código debe incluir documentación estándar.</li>
      <li>
        Las direcciones (address) de ambos contratos deben estar desplegadas y
        probadas, es decir, funcionales.
      </li>
      <li>Testnet: Rococo.</li>
    </ul>
    <h2>Instalación</h2>
    <ol>
      <li>
        Clonar el repositorio:
        <pre><code>git clone https://github.com/NahuelArn/Sistema-de-Votacion-Electronica-con-Blockchain.git
cd Sistema-de-Votacion-Electronica-con-Blockchain/</code></pre>
      </li>
      <li>
        Instalar las dependencias:
        <pre><code>cargo build</code></pre>
      </li>
    </ol>
    <h2>Uso</h2>
    <ol>
      <li>
        Desplegar los contratos:
        <pre><code>cargo contract build</code></pre>
      </li>
      <li>Configurar y crear una elección.</li>
      <li>Registrar candidatos y votantes.</li>
      <li>Iniciar y finalizar la votación según las fechas configuradas.</li>
      <li>Generar y revisar los reportes.</li>
    </ol>
    <h2>Recursos Adicionales</h2>
    <ul>
      <li>
        <a href="https://polkadot.js.org/apps/#/contracts"
          >Polkadot.js Apps (Contracts)</a
        >
      </li>
      <li>
        <a
          href="https://ui.use.ink/contract/5F9ubUzCQXBhemUECMZSuP9ZzhfcPfgnvMEZhzfwuXN7phpk"
          >Página de despliegue de contratos en Rococo</a
        >
      </li>
    </ul>
    <h2 align="center">Licencia</h2>
    <p align="center">
      Este proyecto está licenciado bajo los términos de la
      <a href="LICENSE">MIT License</a>.
    </p>
  </body>
</html>
