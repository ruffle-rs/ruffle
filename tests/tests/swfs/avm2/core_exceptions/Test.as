// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {
        }
    }
}


namespace ns = "ns";

class C {
	public var field;
	public function get getonly() { return 1; }
	public function set setonly(value) { }
	public function method() { return 1; }
}
var num = 1;
var nullish = null;
var undef = undefined;
var o = new C();

trace("// 1009: use of null")
try { nullish.field; } catch(e: TypeError) { trace(e.errorId); } // note: can't print error due to Ruffle-specific error info
try { nullish.field = 1; } catch(e: TypeError) { trace(e.errorId); } // note: can't print error due to Ruffle-specific error info
try { delete nullish.field; } catch(e: TypeError) { trace(e.errorId); } // note: can't print error due to Ruffle-specific error info
try { nullish.method(); } catch(e: TypeError) { trace(e.errorId); } // note: can't print error due to Ruffle-specific error info
trace("// 1010: use of undefined")
try { undef.field; } catch(e: TypeError) { trace(e.errorID); } // note: can't print error due to Ruffle-specific error info
try { undef.field = 1; } catch(e: TypeError) { trace(e.errorID); } // note: can't print error due to Ruffle-specific error info
try { delete undef.field; } catch(e: TypeError) { trace(e.errorID); } // note: can't print error due to Ruffle-specific error info
try { undef.method(); } catch(e: TypeError) { trace(e.errorID); } // note: can't print error due to Ruffle-specific error info
trace("// 1037: setproperty to method")
try { o.method = 1; } catch(e: ReferenceError) { trace(e); }
trace("// 1056: setproperty fail")
try { o.asdf = 1; } catch(e: ReferenceError) { trace(e); }
try { o.ns::asdf = 1; } catch(e: ReferenceError) { trace(e); }
try { num.field = 1; } catch(e: ReferenceError) { trace(e.errorID); } // note: FP prints `Number` due to traits shenanigans
trace("// 1069: getproperty not found")
try { o.asdf; } catch(e: ReferenceError) { trace(e); }
try { o.ns::asdf; } catch(e: ReferenceError) { trace(e); }
try { num.field; } catch(e: ReferenceError) { trace(e.errorID); } // note: FP prints `Number` due to traits shenanigans
trace("// 1074: setproperty on read-only")
try { o.getonly = 1; } catch(e: ReferenceError) { trace(e); }
trace("// 1077: getproperty on write-only")
try { o.setonly; } catch(e: ReferenceError) { trace(e); }
trace("// 1120: deleteproperty on non-object")
try { delete num.field; } catch(e: ReferenceError) { trace(e.errorID); } // note: FP prints `Number` due to traits shenanigans

trace("// should not throw")
try { delete o.asdf; } catch(e: ReferenceError) { trace(e); } // should not throw
try { delete o.field; } catch(e: ReferenceError) { trace(e); } // should not throw