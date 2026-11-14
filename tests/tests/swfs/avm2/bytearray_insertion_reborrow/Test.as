package {
    import flash.display.MovieClip;
    import flash.utils.ByteArray;

    public class Test extends MovieClip {
        public function Test() {
            var arr:ByteArray = new ByteArray();
            
            var dumpArray:Function = function():void {
                var result:String = "";
                for (var i:int = 0; i < arr.length; i ++) {
                    result += arr[i];
                    if (i !== arr.length - 1) {
                        result += ",";
                    }
                }
                trace("Bytearray length is " + arr.length + ", contents are " + result);
            };

            arr.length = 4;
            dumpArray();
            arr[0] = 2;
            dumpArray();
            arr[3] = {
                "valueOf": function() {
                    arr.length = 2;
                    arr[1] = 4;
                    dumpArray();
                }
            };
            dumpArray();
            arr[0] = {
                "valueOf": function() {
                    dumpArray();
                }
            };
        }
    }
}
