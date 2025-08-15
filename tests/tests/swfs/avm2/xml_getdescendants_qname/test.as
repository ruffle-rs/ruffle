// compiled with mxmlc, with bytecode modifications in FFDEC

import flash.utils.getQualifiedClassName;
import flash.utils.getTimer;

class C{
    public function test(x){
        trace('getdescendants QName(null, null)')
        var a = x; // note: put `getdescendants QName(null, null)` here
        for each(var i in a) { trace(i.nodeKind()); }
        trace();
        trace('getdescendants Multiname(null, [Namespace("")])')
        var b = x; // note: put `getdescendants Multiname(null, [Namespace("")])` here
        for each(var i in b) { trace(i.nodeKind()); }
        trace();
        trace('getdescendants Multiname(null, [Namespace(""), Namespace("x")])')
        var c = x; // note: put `getdescendants Multiname(null, [Namespace(""), Namespace("x")])` here
        for each(var i in c) { trace(i.nodeKind()); }
    }
}
XML.ignoreWhitespace = false;
var s = "<a>   <b>  x  <c>   y </c>  </b> </a>";
var x = new XML(s);
new C().test(x);

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test(){
        }
    }
}
