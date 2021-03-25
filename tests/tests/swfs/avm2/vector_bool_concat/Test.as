package {
	public class Test {
	}
}

function trace_vec(v) {
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<Boolean> = new <Boolean>[true, false];");
var a:Vector.<Boolean> = new <Boolean>[true, false];

trace("/// var b: Vector.<Boolean> = new <Boolean>[false, true, false];");
var b:Vector.<Boolean> = new <Boolean>[false, true, false];

trace("/// var c = a.concat(b);");
var c = a.concat(b);

trace("/// (contents of c...)");
trace_vec(c);