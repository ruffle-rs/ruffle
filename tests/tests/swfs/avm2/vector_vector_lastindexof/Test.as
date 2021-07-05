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

trace("/// a.lastIndexOf(a0)");
trace(a.lastIndexOf(a0));

trace("/// a.lastIndexOf(a1)");
trace(a.lastIndexOf(a1));

trace("/// a.lastIndexOf(new <int>[4,3])");
trace(a.lastIndexOf(new <int>[4, 3]));

trace("/// a.lastIndexOf(b0)");
trace(a.lastIndexOf(b0));

trace("/// a.lastIndexOf(b1)");
trace(a.lastIndexOf(b1));

trace("/// a.lastIndexOf(new <int>[19,8])");
trace(a.lastIndexOf(new <int>[19,8]));

trace("/// b.lastIndexOf(a0)");
trace(b.lastIndexOf(a0));

trace("/// b.lastIndexOf(a1)");
trace(b.lastIndexOf(a1));

trace("/// b.lastIndexOf(new <int>[4,3])");
trace(b.lastIndexOf(new <int>[4, 3]));

trace("/// b.lastIndexOf(b0)");
trace(b.lastIndexOf(b0));

trace("/// b.lastIndexOf(b1)");
trace(b.lastIndexOf(b1));

trace("/// b.lastIndexOf(new <int>[19,8])");
trace(b.lastIndexOf(new <int>[19,8]));

trace("/// a.lastIndexOf(a0, 0)");
trace(a.lastIndexOf(a0, 0));

trace("/// a.lastIndexOf(a1, 0)");
trace(a.lastIndexOf(a1, 0));

trace("/// a.lastIndexOf(new <int>[4,3], 0)");
trace(a.lastIndexOf(new <int>[4, 3], 0));

trace("/// a.lastIndexOf(b0, 0)");
trace(a.lastIndexOf(b0, 0));

trace("/// a.lastIndexOf(b1, 0)");
trace(a.lastIndexOf(b1, 0));

trace("/// a.lastIndexOf(new <int>[19,8], 0)");
trace(a.lastIndexOf(new <int>[19,8], 0));

trace("/// b.lastIndexOf(a0, 0)");
trace(b.lastIndexOf(a0, 0));

trace("/// b.lastIndexOf(a1, 0)");
trace(b.lastIndexOf(a1, 0));

trace("/// b.lastIndexOf(new <int>[4,3], 0)");
trace(b.lastIndexOf(new <int>[4, 3], 0));

trace("/// b.lastIndexOf(b0, 0)");
trace(b.lastIndexOf(b0, 0));

trace("/// b.lastIndexOf(b1, 0)");
trace(b.lastIndexOf(b1, 0));

trace("/// b.lastIndexOf(new <int>[19,8], 0)");
trace(b.lastIndexOf(new <int>[19,8], 0));

trace("/// a.lastIndexOf(a0, -1)");
trace(a.lastIndexOf(a0, -1));

trace("/// a.lastIndexOf(a1, -1)");
trace(a.lastIndexOf(a1, -1));

trace("/// a.lastIndexOf(new <int>[4,3], -1)");
trace(a.lastIndexOf(new <int>[4, 3], -1));

trace("/// a.lastIndexOf(b0, -1)");
trace(a.lastIndexOf(b0, -1));

trace("/// a.lastIndexOf(b1, -1)");
trace(a.lastIndexOf(b1, -1));

trace("/// a.lastIndexOf(new <int>[19,8], -1)");
trace(a.lastIndexOf(new <int>[19,8], -1));

trace("/// b.lastIndexOf(a0, -1)");
trace(b.lastIndexOf(a0, -1));

trace("/// b.lastIndexOf(a1, -1)");
trace(b.lastIndexOf(a1, -1));

trace("/// b.lastIndexOf(new <int>[4,3], -1)");
trace(b.lastIndexOf(new <int>[4, 3], -1));

trace("/// b.lastIndexOf(b0, -1)");
trace(b.lastIndexOf(b0, -1));

trace("/// b.lastIndexOf(b1, -1)");
trace(b.lastIndexOf(b1, -1));

trace("/// b.lastIndexOf(new <int>[19,8], -1)");
trace(b.lastIndexOf(new <int>[19,8], -1));