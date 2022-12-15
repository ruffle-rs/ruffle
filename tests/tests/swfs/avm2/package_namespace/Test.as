// NOTE:
// this test was manually hacked with JPEXS - do not just recompile this file and call it a day.

package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
        }
    }
}


import flash.events.MouseEvent;

// NOTE changed from PackageNamespace("flash.events") to Namespace("flash.events")
trace(MouseEvent);

namespace ns = "ns";
namespace empty = "";
class C {
	public var a = 1;
	public var b = 2; // NOTE changed from PackageNamespace("") to Namespace("")
	ns var c = 3;
	ns var d = 4; // NOTE changed from Namespace("ns") to PackageNamespace("ns")
}
var c = new C();
trace(c.a);
trace(c.b);
trace(c.empty::a);
trace(c.empty::b);
trace(c.ns::c);
trace(c.ns::d);


