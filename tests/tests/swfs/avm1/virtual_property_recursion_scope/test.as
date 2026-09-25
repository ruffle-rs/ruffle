var p = {};
var callsGetter = 0;
var callsSetter = 0;
var n = 0;

var o1 = {}
var o2 = {};
var o3 = {};
o1.__proto__ = p;
o2.__proto__ = p;

p.addProperty("prop", function() {
	callsGetter += 1;
	if (this == o1) {
		n += 1;
		o2.prop;
	} else {
		o1.prop;
	}
	return "x";
}, function() {
	callsSetter += 1;
	if (this == o1) {
		n += 1;
		o2.prop = "f";
	} else {
		o1.prop = "f";
	}
	return "y";
});

o3.addProperty("prop", function() {
	callsGetter += 1;
	o2.prop;
	return "x";
}, function() {
	callsSetter += 1;
	o2.prop = "f";
	return "y";
});

var o4 = {};
var o5 = {};
var getter = function() {
	callsGetter += 1;
	if (this == o4) {
		n += 1;
		o5.prop;
	} else {
		o4.prop;
	}
	return "x";
};
o4.addProperty("prop", getter, null);
o5.addProperty("prop", getter, null);

var o6 = {};
var o6getter = function() {
	callsGetter += 1;
	delete o6.prop;
	o6.addProperty("prop", o6getter, null);
	return o6.prop;
};
o6.addProperty("prop", o6getter, null);

o1.prop = "b";

trace("calls: " + callsGetter + "," + callsSetter + ", " + n);

o2.prop = "h";

trace("calls: " + callsGetter + "," + callsSetter + ", " + n);

o3.prop = "q";

trace("calls: " + callsGetter + "," + callsSetter + ", " + n);

trace("o1.prop: " + o1.prop);
trace("calls: " + callsGetter + "," + callsSetter + ", " + n);
trace("o2.prop: " + o2.prop);
trace("calls: " + callsGetter + "," + callsSetter + ", " + n);
trace("o3.prop: " + o3.prop);
trace("calls: " + callsGetter + "," + callsSetter + ", " + n);
trace("o4.prop: " + o4.prop);
trace("calls: " + callsGetter + "," + callsSetter + ", " + n);
trace("o6.prop: " + o6.prop);
trace("calls: " + callsGetter + "," + callsSetter + ", " + n);

trace("Done");
