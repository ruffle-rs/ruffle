package flash.text.engine {
    import flash.display.DisplayObject;
    import flash.events.EventDispatcher;

    [API("662")]
    public final class GraphicElement extends ContentElement {
        private var _graphic:DisplayObject;
        private var _elementWidth:Number;
        private var _elementHeight:Number;

        public function GraphicElement(
            graphic:DisplayObject = null,
            elementWidth:Number = 15.0,
            elementHeight:Number = 15.0,
            elementFormat:ElementFormat = null,
            eventMirror:EventDispatcher = null,
            textRotation:String = "rotate0"
        ) {
            super(elementFormat, eventMirror, textRotation);

            this._graphic = graphic;
            this._elementWidth = elementWidth;
            this._elementHeight = elementHeight;

            this.init();
        }

        private native function init():void;

        public function get elementHeight():Number {
            return this._elementHeight;
        }
        public function set elementHeight(value:Number):void {
            this._elementHeight = value;
        }

        public function get elementWidth():Number {
            return this._elementWidth;
        }
        public function set elementWidth(value:Number):void {
            this._elementWidth = value;
        }

        public function get graphic():DisplayObject {
            return this._graphic;
        }
        public function set graphic(value:DisplayObject):void {
            this._graphic = value;
        }
    }
}
