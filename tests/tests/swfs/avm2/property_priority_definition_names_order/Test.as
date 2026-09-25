// Build from this directory:
// compc -source-path=. -include-sources p1/Same.as p2/Same.as -output definitions.swc
// mxmlc -source-path=. -include-libraries=definitions.swc -o test.swf -debug Test.as

package {
    import flash.display.MovieClip;
    import flash.system.ApplicationDomain;

    public class Test extends MovieClip {
        public function Test() {
            var names:Vector.<String> =
                ApplicationDomain.currentDomain.getQualifiedDefinitionNames();

            for each (var name:String in names) {
                if (name == "p1::Same" || name == "p2::Same") {
                    trace(name);
                }
            }
        }
    }
}
