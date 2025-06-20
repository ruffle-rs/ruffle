// Compile with:
//  mtasc -main Test.as -swf assets.swf -out test.swf
class Test {
    static function main(self) {
        trace("// hitTest inside (even-odd)")
        trace(self.hitTest(75, 75, true));

        trace("// hitTest in hole (even-odd)")
        trace(self.hitTest(125, 125, true));

        trace("// hitTest outside (even-odd)")
        trace(self.hitTest(75, 175, true));


        trace("// hitTest inside (non-zero)")
        trace(self.hitTest(275, 275, true));

        trace("// hitTest in hole (non-zero)")
        trace(self.hitTest(325, 325, true));

        trace("// hitTest outside (non-zero)")
        trace(self.hitTest(275, 375, true));
    }
}
