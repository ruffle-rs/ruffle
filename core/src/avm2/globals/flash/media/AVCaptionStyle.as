package flash.media {
    [API("688")]
    public class AVCaptionStyle {
        public static const DEFAULT:String = "default";
        public static const NONE:String = "none";

        public static const MONOSPACE_WITH_SERIFS:String = "monospaced_with_serifs";
        public static const MONOSPACED_WITHOUT_SERIFS:String = "monospaced_without_serifs";
        public static const PROPORTIONAL_WITH_SERIFS:String = "proportional_with_serifs";
        public static const PROPORTIONAL_WITHOUT_SERIFS:String = "proportional_without_serifs";

        public static const CASUAL:String = "casual";
        public static const CURSIVE:String = "cursive";
        public static const DEPRESSED:String = "depressed";
        public static const RAISED:String = "raised";
        public static const SMALL_CAPITALS:String = "small_capitals";
        public static const UNIFORM:String = "uniform";

        public static const SMALL:String = "small";
        public static const MEDIUM:String = "medium";
        public static const LARGE:String = "large";

        public static const BRIGHT_MAGENTA:String = "bright_magenta";
        public static const MAGENTA:String = "magenta";
        public static const DARK_MAGENTA:String = "dark_magenta";

        public static const BRIGHT_RED:String = "bright_red";
        public static const RED:String = "red";
        public static const DARK_RED:String = "dark_red";

        public static const BRIGHT_YELLOW:String = "bright_yellow";
        public static const YELLOW:String = "yellow";
        public static const DARK_YELLOW:String = "dark_yellow";

        public static const BRIGHT_GREEN:String = "bright_green";
        public static const GREEN:String = "green";
        public static const DARK_GREEN:String = "dark_green";

        public static const BRIGHT_CYAN:String = "bright_cyan";
        public static const CYAN:String = "cyan";
        public static const DARK_CYAN:String = "dark_cyan";

        public static const BRIGHT_BLUE:String = "bright_blue";
        public static const BLUE:String = "blue";
        public static const DARK_BLUE:String = "dark_blue";

        public static const BRIGHT_WHITE:String = "bright_white";
        public static const WHITE:String = "white";
        public static const GRAY:String = "gray";
        public static const BLACK:String = "black";

        public static const LEFT_DROP_SHADOW:String = "drop_shadow_left";
        public static const RIGHT_DROP_SHADOW:String = "drop_shadow_right";

        private var _backgroundColor:String;
        private var _backgroundOpacity:String;
        private var _bottomInset:String;
        private var _edgeColor:String;
        private var _fillColor:String;
        private var _fillOpacity:String;
        private var _font:String;
        private var _fontColor:String;
        private var _fontEdge:String;
        private var _fontOpacity:String;
        private var _size:String;

        public function AVCaptionStyle() {
            this._backgroundColor = "";
            this._backgroundOpacity = "";
            this._bottomInset = "0";
            this._edgeColor = "";
            this._fillColor = "";
            this._fillOpacity = "";
            this._font = "";
            this._fontColor = "";
            this._fontEdge = "";
            this._fontOpacity = "";
            this._size = "";
        }

        public function get backgroundColor():String {
            return this._backgroundColor;
        }

        public function set backgroundColor(value:String):void {
            this._backgroundColor = value;
        }

        public function get backgroundOpacity():String {
            return this._backgroundOpacity;
        }

        public function set backgroundOpacity(value:String):void {
            this._backgroundOpacity = value;
        }

        public function get bottomInset():String {
            return this._bottomInset;
        }

        public function set bottomInset(value:String):void {
            this._bottomInset = value;
        }

        public function get edgeColor():String {
            return this._edgeColor;
        }

        public function set edgeColor(value:String):void {
            this._edgeColor = value;
        }

        public function get fillColor():String {
            return this._fillColor;
        }

        public function set fillColor(value:String):void {
            this._fillColor = value;
        }

        public function get fillOpacity():String {
            return this._fillOpacity;
        }

        public function set fillOpacity(value:String):void {
            this._fillOpacity = value;
        }

        public function get font():String {
            return this._font;
        }

        public function set font(value:String):void {
            this._font = value;
        }

        public function get fontColor():String {
            return this._fontColor;
        }

        public function set fontColor(value:String):void {
            this._fontColor = value;
        }

        public function get fontEdge():String {
            return this._fontEdge;
        }

        public function set fontEdge(value:String):void {
            this._fontEdge = value;
        }

        public function get fontOpacity():String {
            return this._fontOpacity;
        }

        public function set fontOpacity(value:String):void {
            this._fontOpacity = value;
        }

        public function get size():String {
            return this._size;
        }

        public function set size(value:String):void {
            this._size = value;
        }
    }
}
