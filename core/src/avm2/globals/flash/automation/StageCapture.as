package flash.automation {
    import flash.events.EventDispatcher;
    import flash.geom.Rectangle;

    public class StageCapture extends EventDispatcher {
        public static const CURRENT:String = "current";
        public static const MULTIPLE:String = "multiple";
        public static const NEXT:String = "next";
        public static const RASTER:String = "raster";
        public static const SCREEN:String = "screen";
        public static const STAGE:String = "stage";

        private var _fileNameBase:String = "";
        private var _clipRect:Rectangle = null;
        private var _captureSource:String = STAGE;
        private var _capturePTS:Number = 0;

        public function StageCapture() {
            super();
        }

        public function capture(type:String):void {
        }

        public function cancel():void {
        }

        public function get fileNameBase():String {
            return this._fileNameBase;
        }

        public function set fileNameBase(value:String):void {
            this._fileNameBase = value;
        }

        public function get clipRect():Rectangle {
            return this._clipRect;
        }

        public function set clipRect(value:Rectangle):void {
            this._clipRect = value;
        }

        public function get captureSource():String {
            return this._captureSource;
        }

        public function set captureSource(value:String):void {
            this._captureSource = value;
        }

        public function get capturePTS():Number {
            return this._capturePTS;
        }

        public function set capturePTS(value:Number):void {
            this._capturePTS = value;
        }
    }
}
