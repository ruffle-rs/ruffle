var lv = new LoadVars();

lv["23vsd!@#$%^&*()_+-=;':\",.<>/?[{]}\|`~] "] = "23vsd!@#$%^&*()_+-=;':\",.<>/?[{]}\|`~] ";
trace(lv.toString());

trace(new LoadVars().toString());

var lv = new LoadVars();
var f = {};
f.toString = function() { throw "some error"; };
var g = {};
g.toString = function() { throw "other error"; };
lv["a"] = f;
lv["b"] = g;
try {
    trace(lv.toString());
} catch (err) {
    trace("Caught: " + err);
}

var lv = new LoadVars();
lv.addProperty("k", function() { return this.k; }, null);
lv.addProperty("l", function() { return "m"; }, null);
try {
    trace(lv.toString());
} catch (err) {
    trace("Caught: " + err);
}

var lv = new LoadVars();
lv.addProperty("k", function() { throw "some error"; }, null);
try {
    trace(lv.toString());
} catch (err) {
    trace("Caught: " + err);
}
