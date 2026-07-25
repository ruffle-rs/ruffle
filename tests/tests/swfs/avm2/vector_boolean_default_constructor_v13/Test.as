// Reproduction from Ruffle issue #23317.
// https://github.com/ruffle-rs/ruffle/issues/23317
//
// Compile with Apache Flex SDK's mxmlc. The reporter used:
//   mxmlc -o test.swf -debug Test.as
// which defaults to a modern SWF version (>=14), where Flash and Ruffle
// disagree on the constructor case. To reproduce the SWF-version split,
// compile with:
//   mxmlc -o test_v13.swf -debug Test.as -swf-version 13   // Flash agrees with Ruffle
//   mxmlc -o test_v14.swf -debug Test.as -swf-version 14   // Flash diverges
//
// Flash output (SWF version >= 14):
//   null      <- top-level allocation
//   false     <- constructor allocation
//
// Ruffle output before the fix:
//   null
//   null

package {
    import flash.display.Sprite;
    public class Test extends Sprite {
        public function Test() {
            var vec:Vector.<Boolean> = new Vector.<Boolean>(1, true);
            trace(vec[0]); // false on Flash, null on Ruffle (before fix)
        }
    }
}

var vec:Vector.<Boolean> = new Vector.<Boolean>(1, true);
trace(vec[0]); // null on both
