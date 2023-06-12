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
// note: these are to prevent compiler from rejecting nonsense
var num = 1;
var nullish = null;
var undef = undefined;
var o = new C();

trace("// 1009: use of null")
try { nullish.field; } catch(e: TypeError) { trace(e.errorID); } // note: can't print error due to Ruffle-specific error info
try { nullish.field = 1; } catch(e: TypeError) { trace(e.errorID); } // note: can't print error due to Ruffle-specific error info
try { delete nullish.field; } catch(e: TypeError) { trace(e.errorID); } // note: can't print error due to Ruffle-specific error info
try { nullish.method(); } catch(e: TypeError) { trace(e.errorID); } // note: can't print error due to Ruffle-specific error info
try { o as nullish; } catch(e: TypeError) { trace(e); }
try { o as num; } catch(e: TypeError) { trace(e); } // yeah, FP throws a null error here
try { "asdf" in nullish; } catch(e: TypeError) { trace(e); }
try { with(nullish) {} } catch(e: TypeError) { trace(e); }
try { nullish.(@asdf==1); } catch(e: TypeError) { trace(e); }

trace("// 1010: use of undefined")
try { undef.field; } catch(e: TypeError) { trace(e.errorID); } // note: can't print error due to Ruffle-specific error info
try { undef.field = 1; } catch(e: TypeError) { trace(e.errorID); } // note: can't print error due to Ruffle-specific error info
try { delete undef.field; } catch(e: TypeError) { trace(e.errorID); } // note: can't print error due to Ruffle-specific error info
try { undef.method(); } catch(e: TypeError) { trace(e.errorID); } // note: can't print error due to Ruffle-specific error info
try { o as undef; } catch(e: TypeError) { trace(e); }
try { "asdf" in undef; } catch(e: TypeError) { trace(e); }
try { with(undef) {} } catch(e: TypeError) { trace(e); }
try { undef.(@asdf==1); } catch(e: TypeError) { trace(e); }
try { undef instanceof C; } catch(e: TypeError) { trace(e); }

trace("// 1037: setproperty to method")
try { o.method = 1; } catch(e: ReferenceError) { trace(e); }

trace("// 1041: right-hand side of instanceof must be a class or function")
try { o instanceof nullish; } catch(e: TypeError) { trace(e); }
try { o instanceof undef; } catch(e: TypeError) { trace(e); }
try { o instanceof num; } catch(e: TypeError) { trace(e); }
try { o instanceof o; } catch(e: TypeError) { trace(e); }

trace("// 1041: right-hand side of operator must be a class")
try { o as o; } catch(e: TypeError) { trace(e); }

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
try { delete o.asdf; trace("success"); } catch(e: ReferenceError) { trace(e); }
try { delete o.field; trace("success"); } catch(e: ReferenceError) { trace(e); }
try { trace(nullish instanceof C); } catch(e: ReferenceError) { trace(e); }
