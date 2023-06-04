// Compile with:
//  mtasc -main Test.as -swf assets.swf -out test.swf
class Test {
    static function main(self) {
        var mc = self.not_empty;

        // Valid values
        testBlendMode(mc, "layer");
        testBlendMode(mc, null);
        testBlendMode(mc, undefined);
        testBlendMode(mc, 3);
        testBlendMode(mc, 256 + 3);
        testBlendMode(mc, 3.5);
        
        // Invalid values
        testBlendMode(mc, true);
        testBlendMode(mc, false);
        testBlendMode(mc, -1);
        testBlendMode(mc, "lAyEr");
        testBlendMode(mc, {});
        testBlendMode(mc, self);
    }

    static function testBlendMode(mc, blendMode) {
        // Set blend mode to a known, non-default value.
        mc.blendMode = "invert";

        trace("// mc.blendMode = " + blendMode);
        mc.blendMode = blendMode;
        trace(mc.blendMode + "\n");
    }

}
