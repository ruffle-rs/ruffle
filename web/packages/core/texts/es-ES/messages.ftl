message-unsupported-avm2 =
    Puede que el emulador Ruffle no soporte en su totalidad el ActionScript 3 usado por este contenido.
    Algunas partes del contenido podrían no funcionar de manera correcta.
message-cant-embed =
    Ruffle no pudo ejecutar el Flash incrustado en esta página.
    Puedes intentar abrir el archivo en una pestaña aparte, para evitar este problema.
panic-title = Algo salió mal :(
more-info = Más info
run-anyway = Ejecutar de todos modos
continue = Continuar
report-bug = Reportar un Error
update-ruffle = Actualizar Ruffle
error-csp-conflict =
    Ruffle encontró un problema al intentar inicializarse.
    La Política de Seguridad de Contenido de este servidor web no permite el componente requerido ".wasm". 
    Si eres el administrador del servidor, por favor consulta la wiki de Ruffle para obtener ayuda.
error-unknown =
    Ruffle ha encontrado un problema al tratar de mostrar el contenido Flash.
    { $outdated ->
        [true] Si eres el administrador del servidor, intenta cargar una version más reciente de Ruffle (la version actual esta desactualizada: { $buildDate }).
       *[false] Esto no deberia suceder! apreciariamos que reportes el error!
    }
