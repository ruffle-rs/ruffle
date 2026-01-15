package {
    import flash.display.MovieClip;
    import flash.utils.ByteArray;
    import flash.utils.Dictionary;
    
    public class Test extends MovieClip {

        public function Test() {
            super();

            trace("Dictionary with object key:");
            var b:ByteArray = new ByteArray();
            var dict:Dictionary = new Dictionary();
            var obj:Object = {"key":"value"};
            dict[obj] = "value1";
            b.writeObject(dict);
            b.position = 0;
            dict = b.readObject();

            for(var key in dict) {
                trace(key);
                trace(typeof key);
                for(var keyKey in key) {
                    trace("    " + keyKey + ": " + key[keyKey]);
                }
                trace("    " + dict[key]);
            }

            trace("Dictionary with string key:");
            b = new ByteArray();
            dict = new Dictionary();
            dict["obj"] = "value1";
            b.writeObject(dict);
            b.position = 0;
            dict = b.readObject();

            for(var key in dict) {
                trace(key);
                trace(typeof key);
                for(var keyKey in key) {
                    trace("    " + keyKey + ": " + key[keyKey]);
                }
                trace("    " + dict[key]);
            }
        }
    }
}
