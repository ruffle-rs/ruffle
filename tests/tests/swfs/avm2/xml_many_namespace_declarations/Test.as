package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            // One more than quick-xml's default limit.
            var decls:String = "";
            for (var i:int = 0; i < 257; i++) {
                decls += " xmlns:p" + i + "='urn:" + i + "'";
            }
            var x:XML = new XML("<root" + decls + "><p5:child/></root>");
            trace("namespaceDeclarations: " + x.namespaceDeclarations().length);
            trace("child namespace: " + x.children()[0].namespace().uri);
        }
    }
}
