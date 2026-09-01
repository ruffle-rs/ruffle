message-cant-embed =
    Ruffle kunde inte köra Flash-innehållet som är inbäddat på den här sidan.
    Du kan försöka kringgå problemet genom att öppna filen på en separat flik.
message-restored-from-bfcache =
    Din webbläsare återställde detta Flash-innehåll från en tidigare session.
    För att börja på nytt, ladda om sidan.
panic-title = Något gick fel :(
more-info = Mer information
run-anyway = Kör ändå
continue = Fortsätt
report-bug = Rapportera fel
update-ruffle = Uppdatera Ruffle
ruffle-demo = Webbdemo
ruffle-desktop = Skrivbordsprogram
ruffle-wiki = Visa Ruffles wiki
enable-hardware-acceleration = Hårdvaruaccelerationen verkar vara avstängd. Ruffle kan fortfarande fungera, men det kan gå mycket långsamt. Följ länken nedan för information om hur du aktiverar hårdvaruacceleration:
enable-hardware-acceleration-link = FAQ – hårdvaruacceleration i Chrome
view-error-details = Visa felinformation
open-in-new-tab = Öppna i en ny flik
click-to-unmute = Klicka för att slå på ljudet
clipboard-message-title = Kopiera och klistra in i Ruffle
clipboard-message-description =
    { $variant ->
       *[unsupported] Din webbläsare har inte fullständig åtkomst till urklippet,
        [access-denied] Urklippsåtkomst har nekats,
    } men du kan använda dessa kortkommandon istället:
clipboard-message-copy = { " " } för att kopiera
clipboard-message-cut = { " " } för att klippa ut
clipboard-message-paste = { " " } för att klistra in
error-canvas-reload = Kan inte ladda om med canvas-renderaren när den redan används.
error-file-protocol =
    Det verkar som att du kör Ruffle via protokollet ”file:”.
    Det fungerar inte eftersom webbläsare av säkerhetsskäl blockerar många nödvändiga funktioner.
    Konfigurera i stället en lokal server eller använd webbdemon eller skrivbordsprogrammet.
error-javascript-config =
    Ruffle har stött på ett allvarligt problem på grund av en felaktig JavaScript-konfiguration.
    Om du är serveradministratör kan du kontrollera felinformationen för att se vilken parameter som orsakar felet.
    Du kan även få hjälp i Ruffles wiki.
error-wasm-not-found =
    Ruffle kunde inte läsa in den nödvändiga ”.wasm”-filen.
    Om du är serveradministratör bör du kontrollera att filen har laddats upp korrekt.
    Om problemet kvarstår kan du behöva använda inställningen ”publicPath”. Mer information finns i Ruffles wiki.
error-wasm-mime-type =
    Ruffle har stött på ett allvarligt problem under initieringen.
    Webbservern levererar inte ”.wasm”-filer med rätt MIME-typ.
    Om du är serveradministratör finns mer information i Ruffles wiki.
error-invalid-swf =
    Ruffle kan inte tolka den begärda filen.
    Den troligaste orsaken är att filen inte är en giltig SWF-fil.
error-swf-fetch =
    Ruffle kunde inte läsa in Flash-SWF-filen.
    Den troligaste orsaken är att filen inte längre finns och därför inte kan läsas in.
    Kontakta webbplatsens administratör för hjälp.
error-swf-cors =
    Ruffle kunde inte läsa in Flash-SWF-filen.
    Hämtningen har troligen blockerats av CORS-policyn.
    Om du är serveradministratör finns mer information i Ruffles wiki.
error-wasm-cors =
    Ruffle kunde inte läsa in den nödvändiga ”.wasm”-filen.
    Hämtningen har troligen blockerats av CORS-policyn.
    Om du är serveradministratör finns mer information i Ruffles wiki.
error-wasm-invalid =
    Ruffle har stött på ett allvarligt problem under initieringen.
    Sidan verkar sakna giltiga filer som krävs för att köra Ruffle.
    Om du är serveradministratör finns mer information i Ruffles wiki.
error-wasm-download =
    Ruffle har stött på ett stort fel under initieringen.
    Detta kan ofta lösas av sig själv så du kan prova att ladda om sidan.
    Kontakta annars vänligen webbplatsens administratör.
error-wasm-disabled-on-edge =
    Ruffle kunde inte läsa in den nödvändiga ”.wasm”-filen.
    Försök åtgärda problemet genom att öppna webbläsarens inställningar, klicka på ”Sekretess, sökning och tjänster”, rulla ned och stänga av ”Förbättra säkerheten på webben”.
    Då kan webbläsaren läsa in de nödvändiga ”.wasm”-filerna.
    Om problemet kvarstår kan du behöva använda en annan webbläsare.
error-wasm-unsupported-browser =
    Webbläsaren stöder inte de WebAssembly-tillägg som krävs för att köra Ruffle.
    Byt till en webbläsare som stöds.
    En lista över kompatibla webbläsare finns i wikin.
error-javascript-conflict =
    Ruffle har stött på ett allvarligt problem under initieringen.
    Sidan verkar använda JavaScript-kod som står i konflikt med Ruffle.
    Om du är serveradministratör kan du försöka läsa in filen på en tom sida.
error-javascript-conflict-outdated = Du kan också försöka ladda upp en nyare version av Ruffle, vilket kan kringgå problemet (nuvarande version är utdaterad: { $buildDate }).
error-csp-conflict =
    Ruffle har stött på ett allvarligt problem under initieringen.
    Webbserverns innehållssäkerhetspolicy tillåter inte att den nödvändiga ”.wasm”-komponenten körs.
    Om du är serveradministratör finns mer information i Ruffles wiki.
error-url-invalid =
    Ruffle kunde inte läsa in Flash-SWF-filen.
    Den troligaste orsaken är att Ruffle fick en ogiltig URL till SWF-filen.
error-unknown =
    Ruffle har stött på ett stort fel medan den försökte visa Flash-innehållet.
    { $outdated ->
        [true] Om du är serveradministratören försök att ladda upp en nyare version av Ruffle (nuvarande version är utdaterad: { $buildDate }).
       *[false] Detta är inte tänkt att hända så vi skulle verkligen uppskatta om du kunde rapportera in en bugg!
    }
