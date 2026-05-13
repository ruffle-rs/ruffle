// Probe: flash.media.Video constructor quirks.

package {
    import flash.display.MovieClip;
    import flash.media.Video;
    import flash.system.fscommand;

    public class Test extends MovieClip {
        public function Test() {
            run();
            fscommand("quit");
        }

        private function run():void {
            section("--- no args / undefined / null ---");
            tryCtor("new Video()",                function():Video { return new Video(); });
            tryCtor("new Video(undefined)",       function():Video { return new Video(undefined); });
            tryCtor("new Video(null)",            function():Video { return new Video(null); });
            tryCtor("new Video(undefined, 100)",  function():Video { return new Video(undefined, 100); });
            tryCtor("new Video(null, 100)",       function():Video { return new Video(null, 100); });
            tryCtor("new Video(100, undefined)",  function():Video { return new Video(100, undefined); });
            tryCtor("new Video(100, null)",       function():Video { return new Video(100, null); });

            section("--- single positive arg ---");
            tryCtor("new Video(1)",               function():Video { return new Video(1); });
            tryCtor("new Video(100)",             function():Video { return new Video(100); });
            tryCtor("new Video(320)",             function():Video { return new Video(320); });

            section("--- zero anywhere => default both ---");
            tryCtor("new Video(0)",               function():Video { return new Video(0); });
            tryCtor("new Video(-0)",              function():Video { return new Video(-0); });
            tryCtor("new Video(0, 0)",            function():Video { return new Video(0, 0); });
            tryCtor("new Video(0, 100)",          function():Video { return new Video(0, 100); });
            tryCtor("new Video(100, 0)",          function():Video { return new Video(100, 0); });
            tryCtor("new Video(0, 240)",          function():Video { return new Video(0, 240); });
            tryCtor("new Video(320, 0)",          function():Video { return new Video(320, 0); });

            section("--- two positive args ---");
            tryCtor("new Video(1, 1)",            function():Video { return new Video(1, 1); });
            tryCtor("new Video(100, 100)",        function():Video { return new Video(100, 100); });
            tryCtor("new Video(320, 240)",        function():Video { return new Video(320, 240); });
            tryCtor("new Video(800, 600)",        function():Video { return new Video(800, 600); });

            section("--- negative => RangeError ---");
            tryCtor("new Video(-1)",              function():Video { return new Video(-1); });
            tryCtor("new Video(-1, 100)",         function():Video { return new Video(-1, 100); });
            tryCtor("new Video(100, -1)",         function():Video { return new Video(100, -1); });
            tryCtor("new Video(-1, -1)",          function():Video { return new Video(-1, -1); });
            tryCtor("new Video(0, -1)",           function():Video { return new Video(0, -1); });
            tryCtor("new Video(-1, 0)",           function():Video { return new Video(-1, 0); });
            tryCtor("new Video(-2147483648)",     function():Video { return new Video(-2147483648); });

            section("--- sub-integer floats (truncate toward 0) ---");
            tryCtor("new Video(0.5)",             function():Video { return new Video(0.5); });
            tryCtor("new Video(0.999)",           function():Video { return new Video(0.999); });
            tryCtor("new Video(1.5)",             function():Video { return new Video(1.5); });
            tryCtor("new Video(1.9)",             function():Video { return new Video(1.9); });
            tryCtor("new Video(-0.001)",          function():Video { return new Video(-0.001); });
            tryCtor("new Video(-0.5)",            function():Video { return new Video(-0.5); });
            tryCtor("new Video(-0.999)",          function():Video { return new Video(-0.999); });
            tryCtor("new Video(-1.5)",            function():Video { return new Video(-1.5); });
            tryCtor("new Video(-1.001)",          function():Video { return new Video(-1.001); });

            section("--- NaN / +-Infinity (int coerces to 0) ---");
            tryCtor("new Video(NaN)",             function():Video { return new Video(NaN); });
            tryCtor("new Video(Infinity)",        function():Video { return new Video(Infinity); });
            tryCtor("new Video(-Infinity)",       function():Video { return new Video(-Infinity); });
            tryCtor("new Video(NaN, 100)",        function():Video { return new Video(NaN, 100); });
            tryCtor("new Video(100, Infinity)",   function():Video { return new Video(100, Infinity); });

            section("--- threshold-finding for large width (binary search) ---");
            tryCtor("new Video(0x100000)",        function():Video { return new Video(0x100000); });
            tryCtor("new Video(0x200000)",        function():Video { return new Video(0x200000); });
            tryCtor("new Video(0x300000)",        function():Video { return new Video(0x300000); });
            tryCtor("new Video(0x400000)",        function():Video { return new Video(0x400000); });
            tryCtor("new Video(0x500000)",        function():Video { return new Video(0x500000); });
            tryCtor("new Video(0x600000)",        function():Video { return new Video(0x600000); });
            tryCtor("new Video(0x700000)",        function():Video { return new Video(0x700000); });
            tryCtor("new Video(0x780000)",        function():Video { return new Video(0x780000); });
            tryCtor("new Video(0x7C0000)",        function():Video { return new Video(0x7C0000); });
            tryCtor("new Video(0x7E0000)",        function():Video { return new Video(0x7E0000); });
            tryCtor("new Video(0x7F0000)",        function():Video { return new Video(0x7F0000); });
            tryCtor("new Video(0x7F8000)",        function():Video { return new Video(0x7F8000); });
            tryCtor("new Video(0x7FC000)",        function():Video { return new Video(0x7FC000); });
            tryCtor("new Video(0x7FE000)",        function():Video { return new Video(0x7FE000); });
            tryCtor("new Video(0x7FF000)",        function():Video { return new Video(0x7FF000); });
            tryCtor("new Video(0x7FF800)",        function():Video { return new Video(0x7FF800); });
            tryCtor("new Video(0x7FFC00)",        function():Video { return new Video(0x7FFC00); });
            tryCtor("new Video(0x7FFE00)",        function():Video { return new Video(0x7FFE00); });
            tryCtor("new Video(0x7FFF00)",        function():Video { return new Video(0x7FFF00); });
            tryCtor("new Video(0x7FFFFF)",        function():Video { return new Video(0x7FFFFF); });
            tryCtor("new Video(0x800000)",        function():Video { return new Video(0x800000); });
            tryCtor("new Video(2880, 2880)",      function():Video { return new Video(2880, 2880); });
            tryCtor("new Video(8192, 8192)",      function():Video { return new Video(8192, 8192); });

            section("--- height has same threshold? ---");
            tryCtor("new Video(100, 0x100000)",   function():Video { return new Video(100, 0x100000); });
            tryCtor("new Video(100, 0x400000)",   function():Video { return new Video(100, 0x400000); });
            tryCtor("new Video(100, 0x7FFFFF)",   function():Video { return new Video(100, 0x7FFFFF); });
            tryCtor("new Video(100, 0x800000)",   function():Video { return new Video(100, 0x800000); });
            tryCtor("new Video(0x7FFFFF, 0x7FFFFF)", function():Video { return new Video(0x7FFFFFF, 0x7FFFFFF); });

            section("--- forced numeric coercion via int()/Number() ---");
            tryCtor("new Video(int('100'))",       function():Video { return new Video(int("100")); });
            tryCtor("new Video(int('abc'))",       function():Video { return new Video(int("abc")); });
            tryCtor("new Video(int(true))",        function():Video { return new Video(int(true)); });
            tryCtor("new Video(int(false))",       function():Video { return new Video(int(false)); });
            tryCtor("new Video(int(true), int(false))", function():Video { return new Video(int(true), int(false)); });
            tryCtor("new Video(Number('100.7'))",  function():Video { return new Video(Number("100.7")); });
            tryCtor("new Video(Number('-0.5'))",   function():Video { return new Video(Number("-0.5")); });

            section("--- pre-coerced via Number variable ---");
            tryCtor("var n:Number=-0.5; new Video(n)", function():Video { var n:Number = -0.5; return new Video(n); });
            tryCtor("var n:Number=-1.5; new Video(n)", function():Video { var n:Number = -1.5; return new Video(n); });
            tryCtor("var n:Number=0.999; new Video(n)", function():Video { var n:Number = 0.999; return new Video(n); });

            section("--- subclasses ---");
            tryCtor("new SubVid_NoSuper()",       function():Video { return new SubVid_NoSuper(); });
            tryCtor("new SubVid_DefaultSuper()",  function():Video { return new SubVid_DefaultSuper(); });
            tryCtor("new SubVid_Zero()",          function():Video { return new SubVid_Zero(); });
            tryCtor("new SubVid_Pass(0, 100)",    function():Video { return new SubVid_Pass(0, 100); });
            tryCtor("new SubVid_Pass(50, 60)",    function():Video { return new SubVid_Pass(50, 60); });

            section("--- bounds vs width readback ---");
            boundsRow("new Video()",      new Video());
            boundsRow("new Video(160,120)", new Video(160, 120));
            boundsRow("new Video(0,100)", new Video(0, 100));

            section("--- toString ---");
            trace("new Video().toString() -> " + new Video().toString());
            trace("Video as Class -> " + Video);

            section("--- narrow product threshold (target 2^29) ---");
            tryCtor("new Video(0x200000, 255)",   function():Video { return new Video(0x200000, 255); });
            tryCtor("new Video(0x200000, 256)",   function():Video { return new Video(0x200000, 256); });
            tryCtor("new Video(0x200000, 257)",   function():Video { return new Video(0x200000, 257); });
            tryCtor("new Video(0x100000, 511)",   function():Video { return new Video(0x100000, 511); });
            tryCtor("new Video(0x100000, 512)",   function():Video { return new Video(0x100000, 512); });
            tryCtor("new Video(0x100000, 513)",   function():Video { return new Video(0x100000, 513); });
            tryCtor("new Video(0x80000, 1024)",   function():Video { return new Video(0x80000, 1024); });
            tryCtor("new Video(0x80000, 1025)",   function():Video { return new Video(0x80000, 1025); });
            tryCtor("new Video(1, 0x20000000)",   function():Video { return new Video(1, 0x20000000); });
            tryCtor("new Video(0x20000000, 1)",   function():Video { return new Video(0x20000000, 1); });
            tryCtor("new Video(0x1FFFFFFF, 1)",   function():Video { return new Video(0x1FFFFFFF, 1); });
            tryCtor("new Video(0x20000001, 1)",   function():Video { return new Video(0x20000001, 1); });

            section("--- ctor single-dim cap (h=1, narrow) ---");
            tryCtor("new Video(0x800000, 1)",   function():Video { return new Video(0x800000, 1); });
            tryCtor("new Video(0x1000000, 1)",  function():Video { return new Video(0x1000000, 1); });
            tryCtor("new Video(0x2000000, 1)",  function():Video { return new Video(0x2000000, 1); });
            tryCtor("new Video(0x4000000, 1)",  function():Video { return new Video(0x4000000, 1); });
            tryCtor("new Video(0x8000000, 1)",  function():Video { return new Video(0x8000000, 1); });
            tryCtor("new Video(0xFFFFFFF, 1)",  function():Video { return new Video(0xFFFFFFF, 1); });
            tryCtor("new Video(0x10000000, 1)", function():Video { return new Video(0x10000000, 1); });

            section("--- ctor single-dim cap (w=1, narrow) ---");
            tryCtor("new Video(1, 0x800000)",   function():Video { return new Video(1, 0x800000); });
            tryCtor("new Video(1, 0x1000000)",  function():Video { return new Video(1, 0x1000000); });
            tryCtor("new Video(1, 0x4000000)",  function():Video { return new Video(1, 0x4000000); });
            tryCtor("new Video(1, 0x10000000)", function():Video { return new Video(1, 0x10000000); });

            section("--- attempt to construct with NaN h after huge w ---");
            tryCtor("new Video(0x800000, NaN)",   function():Video { return new Video(0x800000, NaN); });

            section("--- narrow single-dim cap: twips-overflow hypothesis (dim*20<=2^31) ---");
            tryCtor("new Video(0x4000000, 1)",    function():Video { return new Video(0x4000000, 1); });
            tryCtor("new Video(0x6666667, 1)",    function():Video { return new Video(0x6666667, 1); });
            tryCtor("new Video(0x7FFFFFF, 1)",    function():Video { return new Video(0x7FFFFFF, 1); });
            tryCtor("new Video(107374183, 1)",    function():Video { return new Video(107374183, 1); });
            tryCtor("new Video(1, 0x6666667)",    function():Video { return new Video(1, 0x6666667); });
        }

        private function section(s:String):void {
            trace("");
            trace(s);
        }

        private function tryCtor(label:String, f:Function):void {
            try {
                var v:Video = f();
                trace(label
                    + " -> w=" + v.width.toFixed(2)
                    + " h=" + v.height.toFixed(2)
                    + " sx=" + v.scaleX.toFixed(2)
                    + " sy=" + v.scaleY.toFixed(2)
                    + " vW=" + Number(v.videoWidth).toFixed(2)
                    + " vH=" + Number(v.videoHeight).toFixed(2));
            } catch (e:Error) {
                trace(label + " -> THROWS #" + e.errorID);
            }
        }

        private function boundsRow(label:String, v:Video):void {
            var b:* = v.getBounds(v);
            trace(label
                + " -> w=" + v.width.toFixed(2)
                + " h=" + v.height.toFixed(2)
                + " bounds=(" + Number(b.x).toFixed(2) + "," + Number(b.y).toFixed(2) + "," + Number(b.width).toFixed(2) + "," + Number(b.height).toFixed(2) + ")");
        }
    }
}

import flash.media.Video;

class SubVid_NoSuper extends Video {
    public function SubVid_NoSuper() {}
}
class SubVid_DefaultSuper extends Video {
    public function SubVid_DefaultSuper() { super(); }
}
class SubVid_Zero extends Video {
    public function SubVid_Zero() { super(0, 0); }
}
class SubVid_Pass extends Video {
    public function SubVid_Pass(w:int, h:int) { super(w, h); }
}
