package {
	public class Test {
	}
}

function trace_vec(v) {
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2], new <int>[4,3]];");
var a:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[1,2], new <int>[4,3]];

trace("/// var b:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16], new <int>[19,8]];");
var b:Vector.<Vector.<int>> = new <Vector.<int>>[new <int>[5,16], new <int>[19,8]];

trace("/// a.every(function (v) { return v.every(function (v) { return v > 0; }); });");
trace(a.every(function (v) { return v.every(function (v) { return v > 0; }); }));

trace("/// a.every(function (v) { return v.every(function (v) { return v > 2; }); });");
trace(a.every(function (v) { return v.every(function (v) { return v > 2; }); }));

trace("/// b.every(function (v) { return v.every(function (v) { return v > 4; }); });");
trace(b.every(function (v) { return v.every(function (v) { return v > 4; }); }));

trace("/// b.every(function (v) { return v.every(function (v) { return v > 10; }); });");
trace(b.every(function (v) { return v.every(function (v) { return v > 10; }); }));