// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
    	public function Test(){
    	}
    }
}

// note: this entire test is to be replaced by more comprehensive test
// once XML gets implemented.
// This test only checks that `type.@name` looks like a string containing the type name.

import flash.utils.describeType;
import flash.utils.Dictionary;
class C{}
var o = {};

var name; // mxmlc disallows .@name.toString() for some reason

trace(describeType(o).@name == "Object");
name = describeType(o).@name;
trace(name.toString() == "Object");

trace(describeType(C).@name);
name = describeType(C).@name;
trace(name.toString());
trace(describeType(new C()).@name);
trace(describeType(int).@name);
trace(describeType(1).@name);
trace(describeType(Class).@name);
trace(describeType(Dictionary).@name);
trace(describeType(new Dictionary()).@name);
