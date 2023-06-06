// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {

        }
    }
}

function enumerate(x) {
	trace("enumerating: " + x)
    var out = [];
	for (var name in x) {
        out.push(name + " = " + x[name])
	}
    out.sort();
    trace(out);
}
var x = {"key": "value", "key2": "value2"};

enumerate(x);
trace("Delete key2");
delete x["key2"];
enumerate(x);
enumerate({});
enumerate(null);
enumerate(undefined);
