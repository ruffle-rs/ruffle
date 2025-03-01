var DONT_ENUM = 1 << 0;
var DONT_DELETE = 1 << 1;
var READ_ONLY = 1 << 2;

var runObjectTest = function(testName, testFunc) {
    trace("// " + testName);
    var _loc3_ = {};
    testFunc(_loc3_);
};

runObjectTest("get_undefined",function(o) {
    trace(o.not_defined);
});

runObjectTest("set_get",function(o) {
    o.forced = "forced";
    o.natural = "natural";
    trace(o.forced);
    trace(o.natural);
});

runObjectTest("set_readonly",function(o) {
    o.normal = "initial";
    o.readonly = "initial";
    ASSetPropFlags(o, "readonly", READ_ONLY);
    o.normal = "replaced";
    o.readonly = "replaced";
    trace(o.normal);
    trace(o.readonly);
});

runObjectTest("deletable_not_readonly",function(o) {
    o.test = "initial";
    ASSetPropFlags(o, "test", DONT_DELETE);
    trace(delete o.test);
    trace(o.test);
    o.test = "replaced";
    trace(delete o.test);
    trace(o.test);
});

runObjectTest("virtual_get",function(o) {
    var getter = function() {
        return "Virtual!";
    };
    o.addProperty("test", getter, null);
    trace(o.test);
    o.test = "Ignored!";
    trace(o.test);
});

runObjectTest("delete", function(o) {
    var getter = function() {
        return "Virtual!";
    };
    o.addProperty("virtual", getter, null);
    o.addProperty("virtual_un", getter, null);
    ASSetPropFlags(o, "virtual_un", DONT_DELETE);
    o.stored = "Stored!";
    o.stored_un = "Stored!";
    ASSetPropFlags(o, "stored_un", DONT_DELETE);
    trace(delete o.virtual);
    trace(delete o.virtual_un);
    trace(delete o.stored);
    trace(delete o.stored_un);
    trace(delete o.non_existent);
    trace(o.virtual);
    trace(o.virtual_un);
    trace(o.stored);
    trace(o.stored_un);
});

runObjectTest("get_keys", function(o) {
    var getter = function() {
        return null;
    };
    o.stored = null;
    o.stored_hidden = null;
    ASSetPropFlags(o, "stored_hidden", DONT_ENUM);
    o.addProperty("virtual", getter, null);
    o.addProperty("virtual_hidden", getter, null);
    ASSetPropFlags(o, "virtual_hidden", DONT_ENUM);

    var list = [];
    for (var prop in o) {
        list.push(prop);
    }

    var listContains = function(element) {
        var i = 0;
        while (i < list.length) {
            if (list[i] == element) {
                return true;
            }
            i ++;
        }
        return false;
    };

    trace(listContains("stored"));
    trace(listContains("stored_hidden"));
    trace(listContains("virtual"));
    trace(listContains("virtual_hidden"));
});
