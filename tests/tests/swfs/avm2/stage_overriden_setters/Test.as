package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            super();
            testSetter("accessibilityImplementation", null);
            testSetter("accessibilityProperties", null);
            testSetter("alpha", 0.5);
            testSetter("blendMode", null);
            testSetter("cacheAsBitmap", true);
            testSetter("contextMenu", null);
            testSetter("filters", []);
            testSetter("focusRect", false);
            testSetter("height", 100.0);
            testSetter("mask", null);
            testSetter("mouseChildren", false);
            testSetter("name", "theStage");
            testSetter("opaqueBackground", null);
            testSetter("rotation", 1.0);
            testSetter("rotationX", 1.5);
            testSetter("rotationY", 2.0);
            testSetter("rotationZ", 2.5);
            testSetter("scale9Grid", null);
            testSetter("scaleX", 3.0);
            testSetter("scaleY", 2.5);
            testSetter("scaleZ", 2.0);
            testSetter("scrollRect", null);
            testSetter("tabEnabled", false);
            testSetter("tabIndex", -1);
            testSetter("transform", null);
            testSetter("visible", true);
            testSetter("width", 50.75);
            testSetter("x", 0.25);
            testSetter("y", 0.75);
            testSetter("z", 1.25);

            try {
                trace(this.stage.textSnapshot);
            } catch(e:Error) {
                trace("Got " + e.errorID + " trying to get textSnapshot");
            }
        }
        
        function testSetter(setterName:String, value:*):void {
            try {
                this.stage[setterName] = value;
                trace("No error trying to set " + setterName);
            } catch(e:Error) {
                trace("Got " + e.errorID + " trying to set " + setterName);
            }
        }
    }
}
