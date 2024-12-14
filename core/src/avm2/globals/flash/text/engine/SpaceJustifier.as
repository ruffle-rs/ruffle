package flash.text.engine {
    public final class SpaceJustifier extends TextJustifier {
        private var _letterSpacing:Boolean;
        private var _minimumSpacing:Number = 0.5;
        private var _optimumSpacing:Number = 1.0;
        private var _maximumSpacing:Number = 1.5;

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

        public function get minimumSpacing():Number {
            return this._minimumSpacing;
        }

        public function set minimumSpacing(value:Number):void {
            this._minimumSpacing = value;
        }

        public function get maximumSpacing():Number {
            return this._maximumSpacing;
        }

        public function set maximumSpacing(value:Number):void {
            this._maximumSpacing = value;
        }

        public function get optimumSpacing():Number {
            return this._optimumSpacing;
        }

        public function set optimumSpacing(value:Number):void {
            this._optimumSpacing = value;
        }
    }
}
