message-cant-embed =
    Ruffle não conseguiu executar o Flash incorporado nesta página.
    Você pode tentar abrir o arquivo em uma guia separada para evitar esse problema.
panic-title = Algo deu errado :(
more-info = Mais informação
run-anyway = Executar mesmo assim
continue = Continuar
report-bug = Reportar Bug
update-ruffle = Atualizar Ruffle
ruffle-demo = Demo Web
ruffle-desktop = Aplicativo de Desktop
ruffle-wiki = Ver Wiki do Ruffle
enable-hardware-acceleration = Parece que a aceleração de hardware está desabilitada. Embora o Ruffle possa funcionar, ele pode ser muito lento. Você pode descobrir como habilitar a aceleração de hardware seguindo o link abaixo:
view-error-details = Ver detalhes do erro
open-in-new-tab = Abrir em uma nova guia
click-to-unmute = Clique para ativar o som
clipboard-message-title = Copiando e colando no Ruffle
clipboard-message-description =
    { $variant ->
       *[unsupported] Seu navegador não suporta acesso total à área de transferência,
        [access-denied] O acesso à área de transferência foi negado,
    } mas você sempre pode usar estes atalhos:
clipboard-message-copy = { " " } para copiar
clipboard-message-cut = { " " } para recortar
clipboard-message-paste = { " " } para colar
error-file-protocol =
    Parece que você está executando o Ruffle no protocolo "file:".
    Isto não funciona como navegadores bloqueiam muitos recursos de funcionar por razões de segurança.
    Ao invés disso, convidamos você a configurar um servidor local ou a usar a demonstração da web, ou o aplicativo de desktop.
error-javascript-config =
    O Ruffle encontrou um grande problema devido a uma configuração incorreta do JavaScript.
    Se você for o administrador do servidor, convidamos você a verificar os detalhes do erro para descobrir qual parâmetro está com falha.
    Você também pode consultar o wiki do Ruffle para obter ajuda.
error-wasm-not-found =
    Ruffle falhou ao carregar o componente de arquivo ".wasm" necessário.
    Se você é o administrador do servidor, por favor, certifique-se de que o arquivo foi carregado corretamente.
    Se o problema persistir, você pode precisar usar a configuração "publicPath": por favor consulte a wiki do Ruffle para obter ajuda.
error-wasm-mime-type =
    Ruffle encontrou um grande problema ao tentar inicializar.
    Este servidor de web não está servindo ".wasm" arquivos com o tipo MIME correto.
    Se você é o administrador do servidor, por favor consulte o wiki do Ruffle para obter ajuda.
error-invalid-swf =
    Ruffle não pode analisar o arquivo solicitado.
    O motivo provável é que o arquivo solicitado não seja um SWF válido.
error-swf-fetch =
    Ruffle falhou ao carregar o arquivo Flash SWF.
    A razão provável é que o arquivo não existe mais, então não há nada para o Ruffle carregar.
    Tente contatar o administrador do site para obter ajuda.
error-swf-cors =
    Ruffle falhou ao carregar o arquivo Flash SWF.
    O acesso para fetch provavelmente foi bloqueado pela política CORS.
    Se você for o administrador do servidor, consulte o wiki do Ruffle para obter ajuda.
error-wasm-cors =
    Ruffle falhou ao carregar o componente de arquivo ".wasm" necessário.
    O acesso para fetch foi provavelmente bloqueado pela política CORS.
    Se você é o administrador do servidor, por favor consulte a wiki do Ruffle para obter ajuda.
error-wasm-invalid =
    Ruffle encontrou um grande problema ao tentar inicializar.
    Parece que esta página tem arquivos ausentes ou inválidos para executar o Ruffle.
    Se você for o administrador do servidor, consulte o wiki do Ruffle para obter ajuda.
error-wasm-download =
    O Ruffle encontrou um grande problema ao tentar inicializar.
    Muitas vezes isso pode se resolver sozinho, então você pode tentar recarregar a página.
    Caso contrário, contate o administrador do site.
error-wasm-disabled-on-edge =
    O Ruffle falhou ao carregar o componente de arquivo ".wasm" necessário.
    Para corrigir isso, tente abrir configurações do seu navegador, clicando em "Privacidade, pesquisa e serviços", rolando para baixo e desativando "Melhore sua segurança na web".
    Isso permitirá que seu navegador carregue os arquivos ".wasm" necessários.
    Se o problema persistir, talvez seja necessário usar um navegador diferente.
error-javascript-conflict =
    Ruffle encontrou um grande problema ao tentar inicializar.
    Parece que esta página usa código JavaScript que entra em conflito com o Ruffle.
    Se você for o administrador do servidor, convidamos você a tentar carregar o arquivo em uma página em branco.
error-javascript-conflict-outdated = Você também pode tentar fazer o upload de uma versão mais recente do Ruffle que pode contornar o problema (a compilação atual está desatualizada: { $buildDate }).
error-csp-conflict =
    Ruffle encontrou um grande problema ao tentar inicializar.
    A política de segurança de conteúdo deste servidor da web não permite a execução do componente ".wasm" necessário.
    Se você for o administrador do servidor, consulte o wiki do Ruffle para obter ajuda.
error-unknown =
    O Ruffle encontrou um grande problema enquanto tentava exibir este conteúdo em Flash.
    { $outdated ->
        [true] Se você é o administrador do servidor, por favor tente fazer o upload de uma versão mais recente do Ruffle (a compilação atual está desatualizada: { $buildDate }).
       *[false] Isso não deveria acontecer, então apreciaríamos muito se você pudesse arquivar um bug!
    }
