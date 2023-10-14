message-cant-embed =
    Ruffle nu a putut rula Flash încorporat în această pagină.
    Puteți încerca să deschideți fișierul într-o filă separată, pentru a evita această problemă.
panic-title = Ceva a mers prost :(
more-info = Mai multe informații
run-anyway = Rulează oricum
continue = Continuă
report-bug = Raportează un bug
update-ruffle = Actualizează Ruffle
ruffle-demo = Demo web
ruffle-desktop = Aplicație desktop
ruffle-wiki = Vezi wikiul Ruffle
view-error-details = Vezi detaliile erorii
open-in-new-tab = Deschide într-o filă nouă
click-to-unmute = Dă click pentru a dezmuți
error-file-protocol =
    Se pare că rulezi Ruffle pe protocolul „file:”.
    Acesta nu funcționează, deoarece browserele blochează funcționarea multor funcții din motive de securitate.
    În schimb, te invităm să configurezi un server local sau să folosești fie demoul web, fie aplicația desktop.
error-javascript-config =
    Ruffle a întâmpinat o problemă majoră din cauza unei configurări incorecte a JavaScript.
    Dacă sunteți administratorul serverului, vă invităm să verificați detaliile de eroare pentru a afla care parametru este defect.
    Puteți consulta și Ruffle wiki pentru ajutor.
error-wasm-not-found =
    Ruffle a eșuat la încărcarea componentei de fișier ".wasm".
    Dacă sunteți administratorul serverului, vă rugăm să vă asigurați că fișierul a fost încărcat corect.
    Dacă problema persistă, poate fi necesar să utilizaţi setarea "publicPath": vă rugăm să consultaţi Ruffle wiki pentru ajutor.
error-wasm-mime-type =
    Ruffle a întâmpinat o problemă majoră în timp ce se încerca inițializarea.
    Acest server web nu servește ". asm" fișiere cu tipul corect MIME.
    Dacă sunteți administrator de server, vă rugăm să consultați Ruffle wiki pentru ajutor.
error-swf-fetch =
    Ruffle a eșuat la încărcarea fișierului Flash SWF.
    Motivul cel mai probabil este că fişierul nu mai există, deci nu există nimic pentru Ruffle să se încarce.
    Încercați să contactați administratorul site-ului web pentru ajutor.
error-swf-cors =
    Ruffle a eșuat la încărcarea fișierului Flash SWF.
    Accesul la preluare a fost probabil blocat de politica CORS.
    Dacă sunteţi administratorul serverului, vă rugăm să consultaţi Ruffle wiki pentru ajutor.
error-wasm-cors =
    Ruffle a eșuat în încărcarea componentei de fișier ".wasm".
    Accesul la preluare a fost probabil blocat de politica CORS.
    Dacă sunteţi administratorul serverului, vă rugăm să consultaţi Ruffle wiki pentru ajutor.
error-wasm-invalid =
    Ruffle a întâmpinat o problemă majoră în timp ce se încearcă inițializarea.
    Se pare că această pagină are fișiere lipsă sau invalide pentru rularea Ruffle.
    Dacă sunteţi administratorul serverului, vă rugăm să consultaţi Ruffle wiki pentru ajutor.
error-wasm-download =
    Ruffle a întâmpinat o problemă majoră în timp ce încerca să inițializeze.
    Acest lucru se poate rezolva adesea, astfel încât puteţi încerca să reîncărcaţi pagina.
    Altfel, vă rugăm să contactaţi administratorul site-ului.
error-wasm-disabled-on-edge =
    Ruffle nu a putut încărca componenta de fișier ".wasm".
    Pentru a remedia acest lucru, încercați să deschideți setările browser-ului dvs., apăsând pe "Confidențialitate, căutare și servicii", derulând în jos și închizând "Îmbunătățește-ți securitatea pe web".
    Acest lucru va permite browser-ului să încarce fișierele ".wasm" necesare.
    Dacă problema persistă, ar putea fi necesar să folosiți un browser diferit.
error-javascript-conflict =
    Ruffle a întâmpinat o problemă majoră în timp ce încerca să inițializeze.
    Se pare că această pagină folosește codul JavaScript care intră în conflict cu Ruffle.
    Dacă sunteţi administratorul serverului, vă invităm să încărcaţi fişierul pe o pagină goală.
error-javascript-conflict-outdated = De asemenea, poți încerca să încarci o versiune mai recentă de Ruffle care poate ocoli problema (versiunea curentă este expirată: { $buildDate }).
error-csp-conflict =
    Ruffle a întâmpinat o problemă majoră în timp ce se încerca inițializarea.
    Politica de securitate a conținutului acestui server web nu permite serviciul necesar". asm" componentă pentru a rula.
    Dacă sunteți administratorul de server, consultați Ruffle wiki pentru ajutor.
error-unknown =
    Ruffle a întâmpinat o problemă majoră în timp ce încerca să afișeze acest conținut Flash.
    { $outdated ->
        [true] Dacă ești administratorul serverului, te rugăm să încerci să încarci o versiune mai recentă de Ruffle (versiunea actuală este depăşită: { $buildDate }).
       *[false] Acest lucru nu ar trebui să se întâmple, așa că am aprecia foarte mult dacă ai putea trimite un bug!
    }
