package flash.text.engine {
    public final class SpaceJustifier extends TextJustifier {
        private var _letterSpacing:Boolean;

        public function SpaceJustifier(locale:String = "en", lineJustification:String = "unjustified", letterSpacing:Boolean = false) {
            super(locale, lineJustification);
            this._letterSpacing = letterSpacing;
        }
        
        public function get letterSpacing():Boolean {
            return this._letterSpacing;
        }
        
        public function set letterSpacing(value:Boolean):void {
            this._letterSpacing = value;
        }
    }
}
