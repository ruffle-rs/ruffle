// Verifies that `getURL` normalizes the `window` (target) argument the
// same way Flash Player does for `_blank`:
//   - Matched case-insensitively
//   - The leading underscore is optional, so "blank" also means "_blank"
//   - Any other value (including the other reserved targets such as "_self")
//     is passed through unchanged
//
// Each call uses a unique URL so the order of normalized targets in the
// trace output is unambiguous.
class Main extends MovieClip {
    function onLoad() {
        var targets:Array = [
            "_blank", "blank", "BLANK", "_Blank", "_BLANK", "bLaNk", "_BlAnK",
            "_self", "self",
            "_parent", "parent",
            "_top", "top",
            "myWindow", "", "_custom", "blanket", "_blanker"
        ];
        for (var i:Number = 0; i < targets.length; i++) {
            var t:String = targets[i];
            trace("// target: [" + t + "]");
            getURL("https://example.com/" + i, t);
            trace("");
        }
    }
}
