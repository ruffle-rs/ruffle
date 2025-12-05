package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            super();
            var obj:Object = {};

            testObj(obj, "before setting property");
            obj.prop = 99;
            testObj(obj, "after setting property");
            obj.setPropertyIsEnumerable("prop", false);
            testObj(obj, "after making property not-enumerable");
            obj.prop = 99;
            testObj(obj, "after re-setting property");
            obj.prop = 89;
            testObj(obj, "after re-setting property (2)");
            delete obj.prop;
            testObj(obj, "after deleting property");
            obj.prop = 12;
            testObj(obj, "after re-setting property (3)");
        }

        static function testObj(obj:Object, info:String) {
            trace(info);
            trace("    value of prop: " + obj.prop);
            trace("    is prop enumerable: " + obj.propertyIsEnumerable("prop"));
            for (var key in obj) {
                if (key === "prop") {
                    trace("    prop iterated over");
                }
            }
        }
    }
}

