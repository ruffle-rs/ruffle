package flash.text.engine {
    public final class EastAsianJustifier extends TextJustifier {
        public function EastAsianJustifier(locale:String = "ja", lineJustification:String = "allButLast", justificationStyle:String = "pushInKinsoku") {
            super(locale, lineJustification);
        }
    }
}
