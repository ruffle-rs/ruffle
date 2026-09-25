package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {

        public function Test() {
            test("<xml>");
            test("<xm");
            test("><");
            test("<");
            test("<a/><![CDATA[");
            test("<!DOCTYPE ");
            test({"toString": function() { return this; }});
        }

        function test(info:*):void {
            trace("Testing case...");
            try {
                new XMLList(info);
                trace("    None of the test cases should construct successfully!");
            } catch(e:Error) {
                trace("    Error (expected): " + e.getStackTrace());
            }
        }
    }
}
