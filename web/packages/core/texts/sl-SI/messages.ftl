message-cant-embed =
    Ruffle ni mogel zagnati Flash vsebine, vgrajene v to stran.
    Lahko poskusite odpreti datoteko v ločenem zavihku, da se izognete tej težavi.
message-restored-from-bfcache =
    Vaš brskalnik je obnovil to Flash vsebino iz prejšnje seje.
    Da bi začeli na novo, ponovno naložite stran.
panic-title = Nekaj je šlo narobe :(
more-info = Več informacij
run-anyway = Vseeno zaženi
continue = Nadaljuj
report-bug = Prijavi napako
update-ruffle = Posodobite Ruffle
ruffle-demo = Spletni demo
ruffle-desktop = Namizna aplikacija
ruffle-wiki = Oglejte si Ruffle Wiki
enable-hardware-acceleration = Zdi se, da je strojna pospešitev onemogočena. Ruffle bo sicer deloval, vendar bo lahko zelo počasen. Kako omogočiti strojno pospešitev, lahko izveste na spodnji povezavi:
enable-hardware-acceleration-link = Pogosta vprašanja – Pospeševanje strojne opreme v brskalniku Chrome
view-error-details = Poglej podrobnosti napake
open-in-new-tab = Odpri v novem zavihku
click-to-unmute = Kliknite za vklop zvoka
clipboard-message-title = Kopiranje in lepljenje v Ruffle
clipboard-message-description =
    { $variant ->
       *[unsupported] Vaš brskalnik ne podpira polnega dostopa do odložišča,
        [access-denied] Dostop do odložišča je bil zavrnjen,
    } vendar lahko namesto tega vedno uporabite te bližnjice:
clipboard-message-copy = { " " } za kopiranje
clipboard-message-cut = { " " } za izrez
clipboard-message-paste = { " " } za lepljenje
error-canvas-reload = Ne morem ponovno naložiti z upodabljalnikom platna, če je upodabljalnik platna že v uporabi.
error-file-protocol =
    Zdi se, da uporabljate Ruffle na protokolu "file:".
    To ne deluje, ker brskalniki iz varnostnih razlogov blokirajo delovanje mnogih funkcij.
    Namesto tega vam priporočamo, da nastavite lokalni strežnik ali uporabite spletno demo ali namizno aplikacijo.
error-javascript-config =
    Ruffle je naletel na večjo težavo zaradi nepravilne konfiguracije JavaScript.
    Če ste skrbnik strežnika, vas prosimo, da preverite podrobnosti napake in ugotovite, kateri parameter je kriv.
    Za pomoč lahko poiščete tudi wiki Ruffle.
error-invalid-swf =
    Ruffle ne more razčleniti zahtevane datoteke.
    Najverjetnejši razlog je, da zahtevana datoteka ni veljavna datoteka SWF.
error-swf-fetch =
    Ruffle ni uspel naložiti datoteke Flash SWF.
    Najverjetnejši razlog je, da datoteka ne obstaja več, zato Ruffle nima kaj naložiti.
    Za pomoč se obrnite na skrbnika spletnega mesta.
error-swf-cors =
    Ruffle ni uspel naložiti datoteke Flash SWF.
    Dostop do prenosa je verjetno blokiran s politiko CORS.
    Če ste skrbnik strežnika, poiščite pomoč v Ruffle wiki.
error-wasm-invalid =
    Ruffle je med poskusom inicializacije naletel na večjo težavo.
    Zdi se, da na tej strani manjkajo datoteke ali so datoteke za zagon Ruffle neveljavne.
    Če ste skrbnik strežnika, poiščite pomoč v Ruffle wiki.
error-wasm-download =
    Ruffle je med poskusom inicializacije naletel na večjo težavo.
    Ta se pogosto reši sama, zato lahko poskusite ponovno naložiti stran.
    V nasprotnem primeru se obrnite na skrbnika spletnega mesta.
error-wasm-unsupported-browser =
    Brskalnik, ki ga uporabljate, ne podpira razširitev WebAssembly, ki jih Ruffle potrebuje za delovanje.
    Preklopite na podprt brskalnik.
    Seznam podprtih brskalnikov najdete na Wiki.
error-javascript-conflict =
    Ruffle je med poskusom inicializacije naletel na večjo težavo.
    Zdi se, da ta stran uporablja JavaScript kodo, ki je v nasprotju z Ruffle.
    Če ste skrbnik strežnika, vas prosimo, da poskusite naložiti datoteko na prazno stran.
error-javascript-conflict-outdated = Lahko poskusite naložiti novejšo različico Ruffle, ki bo morda odpravila težavo (trenutna različica je zastarela: { $buildDate }).
error-unknown =
    Ruffle je naletel na večjo težavo pri prikazovanju te vsebine Flash.
    { $outdated ->
        [true] Če ste skrbnik strežnika, poskusite naložiti novejšo različico Ruffle (trenutna različica je zastarela: { $buildDate }).
       *[false] To se ne bi smelo zgoditi, zato bi bili zelo hvaležni, če bi prijavili napako!
    }
