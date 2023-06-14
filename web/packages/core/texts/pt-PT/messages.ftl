message-unsupported-avm2 =
    O emulador Ruffle pode não suportar completamente o ActionScript 3 usado neste conteúdo.
    Algumas partes do conteúdo podem não funcionar como o esperado.
message-cant-embed =
    O Ruffle não conseguiu abrir o Flash integrado nesta página.
    Pode tentar abrir o arquivo numa página separada para corrigir o problema.
panic-title = Algo de errado aconteceu :(
more-info = Mais informações
run-anyway = Executar mesmo assim
continue = Continuar
report-bug = Reportar problema
update-ruffle = Atualizar o Ruffle
ruffle-demo = Demonstração na Web
ruffle-desktop = Aplicativo para Área de Trabalho
ruffle-wiki = Ver a Wiki do Ruffle
view-error-details = Ver detalhes do erro
open-in-new-tab = Abrir numa nova página
click-to-unmute = Clique para ativar o som
error-file-protocol =
    Parece que está a executar o Ruffle em "file:" protocolo.
    Isto não funciona, pois, os navegadores bloqueiam de Internet bloqueiam muitas funcionalidades por razões de segurança.
    Em vez disto, nós recomendados configurar um servidor local ou usar a demonstração na web, ou o aplicativo para desktop.
error-javascript-config =
    Ruffle encontrou um problema maior devido a uma configuração de JavaScript incorreta.
    Se é o administrador do servidor, convidamos você a verificar os detalhes do erro para descobrir de qual parâmetro é a culpa.
    Também pode consultar a wiki do Ruffle para obter ajuda.
error-wasm-not-found =
    Ruffle falhou ao carregar o componente de arquivo ".wasm" necessário.
    Se você é o administrador do servidor, por favor, certifique-se de que o arquivo foi carregado corretamente.
    Se o problema persistir, você pode precisar usar a configuração "publicPath": por favor consulte a wiki do Ruffle para obter ajuda.
error-wasm-mime-type =
    Ruffle encontrou um grande problema ao tentar inicializar.
    Este servidor de web não suporta arquivos ".wasm" com o tipo MIME correto.
    Se você é o administrador do servidor, por favor consulte o wiki do Ruffle para obter ajuda.
error-swf-fetch =
    Ruffle falhou ao carregar o arquivo SWF do Flash
    A razão mais provável é que o arquivo não existe mais, então não há nada para o Ruffle carregar.
    Tente contactar o administrador do site para obter ajuda.
error-swf-cors =
    Ruffle falhou ao carregar o arquivo Flash SWF.
    O acesso para buscar foi provavelmente bloqueado pela política de CORS.
    Se você é o administrador do servidor, por favor consulte a wiki do Ruffle para obter ajuda.
error-wasm-cors =
    Ruffle falhou ao carregar o componente de arquivo ".wasm" necessário.
    O acesso para buscar foi provavelmente bloqueado pela política CORS.
    Se você é o administrador do servidor, por favor consulte a wiki do Ruffle para obter ajuda.
error-wasm-invalid =
    Ruffle encontrou um grande problema ao tentar inicializar.
    Parece que esta página está ausente ou arquivos inválidos para executar o Ruffle.
    Se você é o administrador do servidor, por favor consulte a wiki do Ruffle para obter ajuda.
error-wasm-download =
    O Ruffle encontrou um grande problema ao tentar inicializar.
    Muitas vezes isso pode se resolver sozinho e você pode tentar recarregar a página.
    Caso contrário, contacte o administrador do site.
error-wasm-disabled-on-edge =
    O Ruffle falhou ao carregar o componente de arquivo ".wasm" necessário.
    Para corrigir isso, tente abrir configurações do seu navegador, clicando em "Privacidade, pesquisa e serviços", rolando para baixo e desativando "Melhore sua segurança na web".
    Isso permitirá que seu navegador carregue os arquivos ".wasm" necessários.
    Se o problema persistir, talvez seja necessário usar um navegador diferente.
error-javascript-conflict =
    Ruffle encontrou um grande problema ao tentar inicializar.
    Parece que esta página usa código JavaScript que entra em conflito com Ruffle.
    Se você é o administrador do servidor, nós convidamos você a tentar carregar o arquivo em branco.
error-javascript-conflict-outdated = Você também pode tentar carregar uma versão mais recente do Ruffle que pode contornar o problema (compilação atual está desatualizada: { $buildDate }).
error-csp-conflict =
    Ruffle encontrou um grande problema ao tentar inicializar.
    A Política de Segurança de Conteúdo deste servidor não permite o componente ".wasm" necessário para ser executado.
    Se você é o administrador do servidor, por favor consulte o wiki do Ruffle para obter ajuda.
error-unknown =
    O Ruffle encontrou um grande problema enquanto tentava exibir este conteúdo em Flash.
    { $outdated ->
        [true] Se você é o administrador do servidor, por favor tente carregar uma versão mais recente do Ruffle (a compilação atual está desatualizada: { $buildDate }).
       *[false] Isso não deveria acontecer, então apreciaríamos muito se você pudesse reportar o bug!
    }
