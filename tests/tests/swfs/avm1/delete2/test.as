
var a = "a";
var b = "b";
var c = "c";

trace("delete2 'a'");
@PCode {
	Push "a"
	Delete2
	Trace
}
trace(a);

trace("delete2 ' b'");
@PCode {
	Push " b"
	Delete2
	Trace
}
trace(b);

trace("delete2 'b '");
@PCode {
	Push "b "
	Delete2
	Trace
}
trace(b);

trace("delete2 'C'");
@PCode {
	Push "C"
	Delete2
	Trace
}
trace(c);

var o = {};
o.a = "o.a";
o.b = "o.b";

trace("delete2 'o.a'");
@PCode {
	Push "o.a"
	Delete2
	Trace
}

for (var k in o) {
	trace("  " + k);
}
trace(o);
trace(o.a);

trace("delete2 'o:b'");
@PCode {
	Push "o:b"
	Delete2
	Trace
}
trace(o);
trace(o.b);

o.t = "o.t";
trace("delete2 'o.t.y'");
@PCode {
	Push "o.t.y"
	Delete2
	Trace
}
trace(o.t);

o.t = 2;
trace("delete2 'o.t.y'");
@PCode {
	Push "o.t.y"
	Delete2
	Trace
}
trace(o.t);

o.t = undefined;
trace("delete2 'o.t.y'");
@PCode {
	Push "o.t.y"
	Delete2
	Trace
}
trace(o.t);

o.t = null;
trace("delete2 'o.t.y'");
@PCode {
	Push "o.t.y"
	Delete2
	Trace
}
trace(o.t);

o.t = true;
trace("delete2 'o.t.y'");
@PCode {
	Push "o.t.y"
	Delete2
	Trace
}
trace(o.t);

o.t = _root;
trace("delete2 'o.t.y'");
@PCode {
	Push "o.t.y"
	Delete2
	Trace
}
trace(o.t);

_root.y = "_root.y";
trace("delete2 'o.t.y'");
@PCode {
	Push "o.t.y"
	Delete2
	Trace
}
trace(o.t.y);
trace(_root.y);

o.c = {};
o.c.d = "o.c.d";
o.c.e = "o.c.e";
o.c.f = "o.c.f";
o.c.g = "o.c.g";

trace("delete2 'o.c.d'");
@PCode {
	Push "o.c.d"
	Delete2
	Trace
}
trace(o.c.d);

trace("delete2 'o.c:e'");
@PCode {
	Push "o.c:e"
	Delete2
	Trace
}
trace(o.c.e);

trace("delete2 'o:c.f'");
@PCode {
	Push "o:c.f"
	Delete2
	Trace
}
trace(o.c.f);

trace("delete2 'o:c:g'");
@PCode {
	Push "o:c:g"
	Delete2
	Trace
}
trace(o.c.g);

var d = "d";
function test() {
	trace(d);
	trace("inner delete2 'd'");
	@PCode {
		Push "d"
		Delete2
		Trace
	}
	trace(d);
}

test();
trace(d);

var p = {};
p.r = "p.r";
var q = "q";

with (p) {
	trace(q);
	trace("with delete2 'q'");
	@PCode {
		Push "q"
		Delete2
		Trace
	}
	trace(q);

	q = "p.q";
	trace("with delete2 'q'");
	@PCode {
		Push "q"
		Delete2
		Trace
	}
	trace(q);

	trace("with delete2 'p.r'");
	@PCode {
		Push "p.r"
		Delete2
		Trace
	}
	trace(r);
	trace(p.r);
}
