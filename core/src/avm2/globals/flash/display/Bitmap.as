package flash.display {

    [Ruffle(InstanceAllocator)]
    public class Bitmap extends DisplayObject {
        public native function get bitmapData():BitmapData;
        public native function set bitmapData(value:BitmapData):void;
        public native function get pixelSnapping():String;
        public native function set pixelSnapping(value:String):void;
        public native function get smoothing():Boolean;
        public native function set smoothing(value:Boolean):void;

        public function Bitmap(bitmapData:BitmapData = null, pixelSnapping:String = "auto", smoothing:Boolean = false) {
            this.init(bitmapData, pixelSnapping, smoothing);
        }

        private native function init(bitmapData:BitmapData, pixelSnapping:String, smoothing:Boolean):void;
    }
}