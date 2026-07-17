message-cant-embed =
    Ruffle konnte das in diese Seite eingebettete Flash-Element nicht ausführen.
    Sie können versuchen, die Datei in einem separaten Tab zu öffnen, um dieses Problem zu umgehen.
message-restored-from-bfcache =
    Ihr Browser hat diesen Flash-Inhalt aus einer vorherigen Sitzung wiederhergestellt.
    Laden Sie die Seite neu, um neu zu starten.
panic-title = Etwas ist schiefgelaufen :(
more-info = Weitere Informationen
run-anyway = Trotzdem ausführen
continue = Fortfahren
report-bug = Fehler melden
update-ruffle = Ruffle aktualisieren
ruffle-demo = Web-Demo
ruffle-desktop = Desktop-Anwendung
ruffle-wiki = Ruffle-Wiki anzeigen
enable-hardware-acceleration = Es sieht so aus, als sei die Hardwarebeschleunigung deaktiviert. Ruffle funktioniert zwar möglicherweise, könnte aber sehr langsam sein. Unter dem folgenden Link erfahren Sie, wie Sie die Hardwarebeschleunigung aktivieren können:
enable-hardware-acceleration-link = FAQ - Chrome Hardwarebeschleunigung
view-error-details = Fehlerdetails anzeigen
open-in-new-tab = In einem neuen Tab öffnen
click-to-unmute = Zum Aktivieren des Tons klicken
clipboard-message-title = Kopieren und Einfügen in Ruffle
clipboard-message-description =
    { $variant ->
       *[unsupported] Ihr Browser unterstützt keinen vollständigen Zugriff auf die Zwischenablage,
        [access-denied] Der Zugriff auf die Zwischenablage wurde verweigert,
    } Sie können jedoch stattdessen jederzeit diese Tastenkombinationen verwenden:
clipboard-message-copy = { " " } zum Kopieren
clipboard-message-cut = { " " } zum Ausschneiden
clipboard-message-paste = { " " } zum Einfügen
error-canvas-reload = Das Neuladen mit dem Canvas-Renderer ist nicht möglich, wenn dieser bereits verwendet wird.
error-file-protocol =
    Es scheint, als würden Sie Ruffle über das "file:"-Protokoll ausführen.
    Dies funktioniert nicht, da Browser aus Sicherheitsgründen viele Funktionen blockieren.
    Wir empfehlen Ihnen stattdessen, einen lokalen Server einzurichten oder entweder die Web-Demo oder die Desktop-Anwendung zu nutzen.
error-javascript-config =
    Bei Ruffle ist aufgrund einer fehlerhaften JavaScript-Konfiguration ein schwerwiegendes Problem aufgetreten.
    Wenn Sie der Serveradministrator sind, bitten wir Sie, die Fehlerdetails zu überprüfen, um festzustellen, welcher Parameter die Ursache ist.
    Sie können auch im Ruffle-Wiki nach Hilfe suchen.
error-wasm-not-found =
    Ruffle konnte die erforderliche ".wasm"-Datei-Komponente nicht laden.
    Wenn Sie der Server-Administrator sind, stellen Sie bitte sicher, dass die Datei korrekt hochgeladen wurde.
    Wenn das Problem weiterhin besteht, müssen Sie unter Umständen die "publicPath"-Einstellung verwenden: Bitte konsultieren Sie das Ruffle-Wiki für Hilfe.
error-wasm-mime-type =
    Bei der Initialisierung von Ruffle ist ein schwerwiegendes Problem aufgetreten.
    Dieser Webserver stellt ".wasm"-Dateien nicht mit dem richtigen MIME-Typ bereit.
    Wenn Sie der Serveradministrator sind, finden Sie Hilfe im Ruffle-Wiki.
error-invalid-swf =
    Ruffle kann die angeforderte Datei nicht verarbeiten.
    Der wahrscheinlichste Grund dafür ist, dass die angeforderte Datei keine gültige SWF-Datei ist.
error-swf-fetch =
    Ruffle konnte die Flash-SWF-Datei nicht laden.
    Der wahrscheinlichste Grund ist, dass die Datei nicht mehr vorhanden ist und Ruffle daher nichts laden kann.
    Wenden Sie sich bitte an den Administrator der Website, um Hilfe zu erhalten.
error-swf-cors =
    Ruffle konnte die Flash-SWF-Datei nicht laden.
    Der Zugriff auf die Datei wurde wahrscheinlich durch die CORS-Richtlinie blockiert.
    Wenn Sie der Serveradministrator sind, finden Sie Hilfe im Ruffle-Wiki.
error-wasm-cors =
    Ruffle konnte die Flash-SWF-Datei nicht laden.
    Der Zugriff auf den Abruf wurde wahrscheinlich durch die CORS-Richtlinie blockiert.
    Wenn Sie der Serveradministrator sind, finden Sie Hilfe im Ruffle-Wiki.
error-wasm-invalid =
    Bei der Initialisierung von Ruffle ist ein schwerwiegendes Problem aufgetreten.
    Es scheint, als fehlten auf dieser Seite Dateien, die für die Ausführung von Ruffle erforderlich sind, oder als seien diese ungültig.
    Wenn Sie der Serveradministrator sind, finden Sie Hilfe im Ruffle-Wiki.
error-wasm-download =
    Bei der Initialisierung von Ruffle ist ein schwerwiegendes Problem aufgetreten.
    Oftmals behebt sich dieses Problem von selbst, sodass Sie versuchen können, die Seite neu zu laden.
    Andernfalls wenden Sie sich an den Website-Administrator.
error-wasm-disabled-on-edge =
    Ruffle konnte die erforderliche ".wasm"-Datei nicht laden.
    Um das Problem zu beheben, öffnen Sie die Einstellungen Ihres Browsers, klicken Sie auf "Datenschutz, Suche und Dienste", scrollen Sie nach unten und deaktivieren Sie die Option "Sicherheit im Internet verbessern".
    Dadurch kann Ihr Browser die erforderlichen ".wasm"-Dateien laden.
    Sollte das Problem weiterhin bestehen, müssen Sie möglicherweise einen anderen Browser verwenden.
error-wasm-unsupported-browser =
    Der von Ihnen verwendete Browser unterstützt die WebAssembly-Erweiterungen nicht, die Ruffle zum Ausführen benötigt.
    Bitte wechseln Sie zu einem unterstützten Browser.
    Eine Liste der unterstützten Browser finden Sie im Wiki.
error-javascript-conflict =
    Bei der Initialisierung von Ruffle ist ein schwerwiegendes Problem aufgetreten.
    Es scheint, als würde diese Seite JavaScript-Code verwenden, der mit Ruffle in Konflikt steht.
    Falls Sie der Serveradministrator sind, bitten wir Sie, die Datei auf einer leeren Seite zu laden.
error-javascript-conflict-outdated = Sie können auch versuchen, eine neuere Version von Ruffle hochzuladen, die das Problem möglicherweise behebt (der aktuelle Build ist veraltet: { $buildDate }).
error-csp-conflict =
    Bei der Initialisierung von Ruffle ist ein schwerwiegendes Problem aufgetreten.
    Die Content Security Policy dieses Webservers lässt die Ausführung der erforderlichen ".wasm"-Komponente nicht zu.
    Wenn Sie der Serveradministrator sind, finden Sie Hilfe im Ruffle-Wiki.
error-unknown =
    Bei der Anzeige dieses Flash-Inhalts ist bei Ruffle ein schwerwiegendes Problem aufgetreten.
    { $outdated ->
        [true] Wenn Sie der Serveradministrator sind, versuchen Sie bitte, eine aktuellere Version von Ruffle hochzuladen (der aktuelle Build ist veraltet: { $buildDate }).
       *[false] Das sollte eigentlich nicht passieren, daher wären wir Ihnen sehr dankbar, wenn Sie den Fehler melden könnten!
    }
