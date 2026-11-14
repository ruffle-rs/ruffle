package flash.ui {
    import flash.geom.Point;
    import flash.display.BitmapData;

    public final class MouseCursorData {
        private var _data:Vector.<BitmapData>;
        private var _frameRate:Number;
        private var _hotSpot:Point = new Point(0, 0);

        public function get data():Vector.<BitmapData> {
            return this._data;
        }

        public function set data(value:Vector.<BitmapData>):void {
            this._data = value;
        }

        public function get frameRate():Number {
            return this._frameRate;
        }

        public function set frameRate(value:Number):void {
            this._frameRate = value;
        }

        public function get hotSpot():Point {
            return this._hotSpot;
        }

        public function set hotSpot(value:Point):void {
            this._hotSpot = value;
        }
    }
}
