package flash.ui {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;

    import flash.geom.Point;
    import flash.display.BitmapData;

    public final class MouseCursorData {
        private var _data:Vector.<BitmapData>;
        private var _frameRate:Number;
        private var _hotSpot:Point = new Point(0, 0);

        public function get data():Vector.<BitmapData> {
            stub_getter("flash.ui.MouseCursorData", "data");
            return this._data;
        }

        public function set data(value:Vector.<BitmapData>):void {
            stub_setter("flash.ui.MouseCursorData", "data");
            this._data = value;
        }

        public function get frameRate():Number {
            stub_getter("flash.ui.MouseCursorData", "frameRate");
            return this._frameRate;
        }

        public function set frameRate(value:Number):void {
            stub_setter("flash.ui.MouseCursorData", "frameRate");
            this._frameRate = value;
        }

        public function get hotSpot():Point {
            stub_getter("flash.ui.MouseCursorData", "hotSpot");
            return this._hotSpot;
        }

        public function set hotSpot(value:Point):void {
            stub_setter("flash.ui.MouseCursorData", "hotSpot");
            this._hotSpot = value;
        }
    }
}
