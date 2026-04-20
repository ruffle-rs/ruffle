package {
    import flash.display.MovieClip;
    import flash.utils.ByteArray;

    public class Test extends MovieClip {
        public function Test() {
            super();
            var theArray:ByteArray = new ByteArray();
            doTest(function() {
                theArray.compress(null);
            });
            doTest(function() {
                theArray.compress("abcdef");
            });
            doTest(function() {
                theArray.uncompress(null);
            });
            doTest(function() {
                theArray.uncompress("abcdef");
            });
            doTest(function() {
                theArray.uncompress("zlib");
            });
            doTest(function() {
                theArray.endian = null;
            });
            doTest(function() {
                theArray.endian = "abcdef";
            });
            doTest(function() {
                theArray.writeUTF(null);
            });
            doTest(function() {
                theArray.writeUTFBytes(null);
            });
            doTest(function() {
                theArray.writeUTF(null);
            });
            doTest(function() {
                theArray.writeUTF(null);
            });
            doTest(function() {
                theArray.writeMultiByte("abcd", "utf-8");
            });
            doTest(function() {
                theArray.writeMultiByte("abcd", "aisjdasd");
            });
            doTest(function() {
                theArray.writeMultiByte(null, "utf-8");
            });
            doTest(function() {
                theArray.writeMultiByte(null, "aisjdasd");
            });
            doTest(function() {
                theArray.writeMultiByte(null, null);
            });
            doTest(function() {
                theArray.readMultiByte(0, "");
            });
            doTest(function() {
                theArray.readMultiByte(20, "");
            });
            doTest(function() {
                theArray.readMultiByte(0, null);
            });
            doTest(function() {
                theArray.readMultiByte(20, null);
            });
            doTest(function() {
                theArray.readMultiByte(0, "aisjdasd");
            });
            doTest(function() {
                theArray.readMultiByte(20, "aisjdasd");
            });
            doTest(function() {
                theArray.readMultiByte(0, "utf-8");
            });
            doTest(function() {
                theArray.readMultiByte(20, "utf-8");
            });
        }
        
        function doTest(test:Function) {
            try {
                test();
                trace("No error");
            } catch(e:Error) {
                trace("Got " + Object.prototype.toString.call(e) + " id " + e.errorID);
            }
        }
    }
}
