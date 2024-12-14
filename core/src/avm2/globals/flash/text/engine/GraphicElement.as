package flash.text.engine {
    import flash.display.DisplayObject;
    import flash.events.EventDispatcher;

    import __ruffle__.stub_constructor;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;

    public final class GraphicElement extends ContentElement {
        public function GraphicElement(graphic:DisplayObject = null, elementWidth:Number = 15.0, elementHeight:Number = 15.0, elementFormat:ElementFormat = null, eventMirror:EventDispatcher = null, textRotation:String = "rotate0") {
            stub_constructor("flash.text.engine.GraphicElement");
        }

        public function get elementHeight():Number {
            stub_getter("flash.text.engine.GraphicElement", "elementHeight");
            return 15.0;
        }
        public function set elementHeight(value:Number):void {
            stub_setter("flash.text.engine.GraphicElement", "elementHeight");
        }

        public function get elementWidth():Number {
            stub_getter("flash.text.engine.GraphicElement", "elementWidth");
            return 15.0;
        }

        public function set elementWidth(value:Number):void {
            stub_setter("flash.text.engine.GraphicElement", "elementWidth");
        }

        public function get graphic():DisplayObject {
            stub_getter("flash.text.engine.GraphicElement", "graphic");
            return null;
        }

        public function set graphic(value:DisplayObject):void {
            stub_setter("flash.text.engine.GraphicElement", "graphic");
        }
    }
}