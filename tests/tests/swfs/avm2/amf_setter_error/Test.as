package {
    import flash.display.MovieClip;
    import flash.net.registerClassAlias;
    import flash.utils.ByteArray;

    class Cls1 {
        public function get prop():int {
            return 0;
        }
        public function set prop(value:int):void {
            // Test setter throwing error
            throw new Error("Error setting property");
        }
    }

    class Cls2 {
        public function get prop():int {
            return 0;
        }
        public function set prop(value:int):void {
            // Test setter throwing primitive error
            throw "primitive";
        }
    }

    class Cls3 {
        public function get prop():int {
            return 0;
        }
        public function set prop(value:int):void {
            // Test setter throwing an error that will fail to coerce to string
            var error:Object = {
                "toString": function() {
                    trace("toString called");
                    return {};
                }
            };
            throw error;
        }
    }

    class Cls4 {
        public function get prop():int {
            return 0;
        }
        public function set prop(value:int):void {
            // Test setter throwing an error that will throw an error when attempted to coerce to string
            var error:Object = {
                "toString": function() {
                    trace("toString called");
                    throw new Error("error from in toString");
                }
            };
            throw error;
        }
    }

    public class Test extends MovieClip {
        public function Test() {
            registerClassAlias("Cls1", Cls1);
            registerClassAlias("Cls2", Cls2);
            registerClassAlias("Cls3", Cls3);
            registerClassAlias("Cls4", Cls4);

            var objects:Array = [new Cls1(), new Cls2(), new Cls3(), new Cls4()];

            for (var i:int = 0; i < objects.length; i ++) {
                var arr:ByteArray = new ByteArray();
                arr.writeObject(objects[i]);
                arr.position = 0;
                try {
                    trace("Read an object: " + arr.readObject());
                } catch(e:*) {
                    trace("Error reading object: " + e);
                }
            }
        }
    }
}
