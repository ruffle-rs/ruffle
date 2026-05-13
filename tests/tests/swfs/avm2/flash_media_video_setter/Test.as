// Probe: flash.media.Video width/height setter quirks.

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
            section("--- mutation: set width on a defaulted Video ---");
            var m:Video = new Video(0, 0);
            m.width = 320;
            trace("after new Video(0,0); width=320 -> w=" + m.width.toFixed(2) + " sx=" + m.scaleX.toFixed(2));
            var m2:Video = new Video();
            m2.width = 0;
            trace("after new Video(); width=0 -> w=" + m2.width.toFixed(2) + " sx=" + m2.scaleX.toFixed(2));
            var m3:Video = new Video();
            m3.height = 0;
            trace("after new Video(); height=0 -> h=" + m3.height.toFixed(2) + " sy=" + m3.scaleY.toFixed(2));

            // Probes that would write a `scaleX/Y` beyond i32 Twips range
            // (`0x10000000`, `0x7FFFFFFF`, `1e20`, etc.) are dropped pending
            // a Twips storage rework — they trip a subtract-overflow inside
            // `local_bounds()`. FP-truth for those is in the chat history.
            section("--- setter: huge / over-threshold values (no throw expected) ---");
            mut("v.width = 0x7FFFFF",  function(v:Video):void { v.width = 0x7FFFFF; });
            mut("v.width = 0x800000",  function(v:Video):void { v.width = 0x800000; });
            mut("v.width = -100",      function(v:Video):void { v.width = -100; });
            mut("v.width = NaN",       function(v:Video):void { v.width = NaN; });

            section("--- setter: scale then width interplay ---");
            var s1:Video = new Video();
            s1.scaleX = 0;
            s1.width = 100;
            trace("scaleX=0 then width=100 -> w=" + s1.width.toFixed(2) + " sx=" + s1.scaleX.toFixed(2));
            var s2:Video = new Video();
            s2.width = 0;
            s2.width = 320;
            trace("width=0 then width=320 -> w=" + s2.width.toFixed(2) + " sx=" + s2.scaleX.toFixed(2));

            section("--- writing to read-only videoWidth/videoHeight ---");
            var r:Video = new Video(100, 100);
            var ro:Object = r;
            try { ro["videoWidth"]  = 999; trace("v.videoWidth = 999 -> no throw, vW=" + Number(r.videoWidth).toFixed(2)); }
            catch (e:Error) { trace("v.videoWidth = 999 -> THROWS #" + e.errorID); }
            try { ro["videoHeight"] = 999; trace("v.videoHeight = 999 -> no throw, vH=" + Number(r.videoHeight).toFixed(2)); }
            catch (e:Error) { trace("v.videoHeight = 999 -> THROWS #" + e.errorID); }

            section("--- setter: ignore vs reset on invalid ---");
            ignoreOrReset("after width=100; width=-1",      function(v:Video):void { v.width = 100; v.width = -1; });
            ignoreOrReset("after width=100; width=NaN",     function(v:Video):void { v.width = 100; v.width = NaN; });
            ignoreOrReset("after width=100; width=0",       function(v:Video):void { v.width = 100; v.width = 0; });
            ignoreOrReset("after height=50;  height=-1",    function(v:Video):void { v.height = 50; v.height = -1; });

            section("--- setter: cap boundary ---");
            mut("v.width = 10485759",  function(v:Video):void { v.width = 10485759; });
            mut("v.width = 10485760",  function(v:Video):void { v.width = 10485760; });
            mut("v.width = 10485761",  function(v:Video):void { v.width = 10485761; });
            mut("v.width = 10485760.5", function(v:Video):void { v.width = 10485760.5; });
            mut("v.width = 16777215",  function(v:Video):void { v.width = 16777215; });
            mut("v.height = 10485760", function(v:Video):void { v.height = 10485760; });
            mut("v.height = 10485761", function(v:Video):void { v.height = 10485761; });

            section("--- setter: very small / fractional ---");
            mut("v.width = 0.5",        function(v:Video):void { v.width = 0.5; });
            mut("v.width = 1e-10",      function(v:Video):void { v.width = 1e-10; });
            mut("v.width = 0.05",       function(v:Video):void { v.width = 0.05; });
            mut("v.width = 0.0",        function(v:Video):void { v.width = 0.0; });
            mut("v.width = 1e-300",     function(v:Video):void { v.width = 1e-300; });

        }

        private function ignoreOrReset(label:String, f:Function):void {
            var v:Video = new Video();
            try {
                f(v);
                trace(label + " -> w=" + v.width.toFixed(2) + " h=" + v.height.toFixed(2) + " sx=" + v.scaleX.toFixed(2) + " sy=" + v.scaleY.toFixed(2));
            } catch (e:Error) {
                trace(label + " -> THROWS #" + e.errorID);
            }
        }

        private function mut(label:String, f:Function):void {
            var v:Video = new Video();
            try {
                f(v);
                trace(label + " -> w=" + v.width.toFixed(2) + " h=" + v.height.toFixed(2) + " sx=" + v.scaleX.toFixed(2) + " sy=" + v.scaleY.toFixed(2));
            } catch (e:Error) {
                trace(label + " -> THROWS #" + e.errorID);
            }
        }

        private function section(s:String):void {
            trace("");
            trace(s);
        }
    }
}
