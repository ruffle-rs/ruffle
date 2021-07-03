package {
	public class Test {
	}
}

function trace_vec(v) {
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a0 = new <int>[1,2];");
var a0 = new <int>[1,2];

trace("/// var a1 = new <int>[4,3];");
var a1 = new <int>[4,3];

trace("/// var a:Vector.<Vector.<int>> = new <Vector.<int>>[a0, a1];");
var a:Vector.<Vector.<int>> = new <Vector.<int>>[a0, a1];

trace("/// var b0 = new <int>[5,16];");
var b0 = new <int>[5,16];

trace("/// var b1 = new <int>[19,8];");
var b1 = new <int>[19,8];

trace("/// var b:Vector.<Vector.<int>> = new <Vector.<int>>[b0, b1];");
var b:Vector.<Vector.<int>> = new <Vector.<int>>[b0, b1];

trace("/// a.indexOf(a0)");
trace(a.indexOf(a0));

trace("/// a.indexOf(a1)");
trace(a.indexOf(a1));

trace("/// a.indexOf(new <int>[4,3])");
trace(a.indexOf(new <int>[4, 3]));

trace("/// a.indexOf(b0)");
trace(a.indexOf(b0));

trace("/// a.indexOf(b1)");
trace(a.indexOf(b1));

trace("/// a.indexOf(new <int>[19,8])");
trace(a.indexOf(new <int>[19,8]));

trace("/// b.indexOf(a0)");
trace(b.indexOf(a0));

trace("/// b.indexOf(a1)");
trace(b.indexOf(a1));

trace("/// b.indexOf(new <int>[4,3])");
trace(b.indexOf(new <int>[4, 3]));

trace("/// b.indexOf(b0)");
trace(b.indexOf(b0));

trace("/// b.indexOf(b1)");
trace(b.indexOf(b1));

trace("/// b.indexOf(new <int>[19,8])");
trace(b.indexOf(new <int>[19,8]));