package flash.printing {
    import flash.display.Sprite;
    import flash.events.EventDispatcher;
    import flash.geom.Rectangle;

    public class PrintJob extends EventDispatcher {
        private var _active:Boolean = false;
        private var _copies:int = 1;
        private var _firstPage:int = 0;
        private var _lastPage:int = 0;
        private var _jobName:String = "";
        private var _orientation:String = PrintJobOrientation.PORTRAIT;
        private var _pageHeight:int = 0;
        private var _pageWidth:int = 0;
        private var _paperHeight:int = 0;
        private var _paperWidth:int = 0;
        private var _printer:String = null;

        public function PrintJob() {
            super();
        }

        [API("667")]
        public static function get isSupported():Boolean {
            return false;
        }

        [API("674")]
        public static function get printers():Vector.<String> {
            return new Vector.<String>();
        }

        [API("674")]
        public static function get supportsPageSetupDialog():Boolean {
            return false;
        }

        [API("674")]
        public function get active():Boolean {
            return this._active;
        }

        [API("674")]
        public function get copies():int {
            return this._copies;
        }

        [API("674")]
        public function set copies(value:int):void {
            this._copies = value;
        }

        [API("674")]
        public function get firstPage():int {
            return this._firstPage;
        }

        [API("674")]
        public function get jobName():String {
            return this._jobName;
        }

        [API("674")]
        public function set jobName(value:String):void {
            this._jobName = value;
        }

        [API("674")]
        public function get lastPage():int {
            return this._lastPage;
        }

        [API("674")]
        public function get maxPixelsPerInch():Number {
            return 72;
        }

        public function get orientation():String {
            return this._orientation;
        }

        [API("674")]
        public function set orientation(value:String):void {
            if (value != PrintJobOrientation.LANDSCAPE && value != PrintJobOrientation.PORTRAIT) {
                Error.throwError(ArgumentError, 2008, "orientation");
            }
            this._orientation = value;
        }

        public function get pageHeight():int {
            return this._pageHeight;
        }

        public function get pageWidth():int {
            return this._pageWidth;
        }

        [API("674")]
        public function get paperArea():Rectangle {
            return new Rectangle(0, 0, this._paperWidth, this._paperHeight);
        }

        public function get paperHeight():int {
            return this._paperHeight;
        }

        public function get paperWidth():int {
            return this._paperWidth;
        }

        [API("674")]
        public function get printableArea():Rectangle {
            return new Rectangle(0, 0, this._pageWidth, this._pageHeight);
        }

        [API("674")]
        public function get printer():String {
            return this._printer;
        }

        [API("674")]
        public function set printer(value:String):void {
            this._printer = value;
        }

        public function start():Boolean {
            this._active = false;
            return false;
        }

        [API("674")]
        public function start2(uiOptions:Object = null, showPrintDialog:Boolean = true):Boolean {
            this._active = false;
            return false;
        }

        public function addPage(
            sprite:Sprite,
            printArea:Rectangle = null,
            options:PrintJobOptions = null,
            frameNum:int = 0
        ):void {
            if (sprite == null) {
                Error.throwError(TypeError, 2007, "sprite");
            }
            if (!this._active) {
                Error.throwError(Error, 2057);
            }
        }

        public function send():void {
            if (!this._active) {
                Error.throwError(Error, 2057);
            }
            this._active = false;
        }

        [API("674")]
        public function selectPaperSize(paperSize:String):void {
        }

        [API("674")]
        public function showPageSetupDialog():Boolean {
            return false;
        }

        [API("674")]
        public function terminate():void {
            this._active = false;
        }
    }
}
