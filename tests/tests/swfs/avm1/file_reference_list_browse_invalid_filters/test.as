var list = new flash.net.FileReferenceList();

var goodFilter = new Object();
goodFilter.description = "txt";
goodFilter.extension = "*.txt";

trace("// empty array");
trace(list.browse([]));

trace("// non-object element in array");
trace(list.browse(["not an object"]));

var noDesc = new Object();
noDesc.extension = "*.txt";
trace("// filter missing description");
trace(list.browse([noDesc]));

var noExt = new Object();
noExt.description = "txt";
trace("// filter missing extension");
trace(list.browse([noExt]));

var emptyDesc = new Object();
emptyDesc.description = "";
emptyDesc.extension = "*.txt";
trace("// empty description");
trace(list.browse([emptyDesc]));

var emptyExt = new Object();
emptyExt.description = "txt";
emptyExt.extension = "";
trace("// empty extension");
trace(list.browse([emptyExt]));

trace("// string arg");
trace(list.browse("hello"));

trace("// number arg");
trace(list.browse(42));

trace("// null arg");
trace(list.browse(null));

trace("// undefined arg (single explicit undefined)");
trace(list.browse(undefined));

// Filter fields only present on the prototype, not as own properties.

var ProtoFilter = function() {};
ProtoFilter.prototype.description = "txt";
ProtoFilter.prototype.extension = "*.txt";
var protoFilter = new ProtoFilter();
trace("// filter inherits description+extension from prototype");
trace(list.browse([protoFilter]));

// A valid filter — opens the dialog. The dialog stays open for the
// rest of the test, so subsequent valid-filter calls return false. The
// toString-throw cases below still propagate their exceptions normally.

var getterFilter = new Object();
getterFilter.addProperty("description", function() { return "txt"; }, null);
getterFilter.addProperty("extension", function() { return "*.txt"; }, null);
trace("// filter description+extension via addProperty getter");
trace(list.browse([getterFilter]));

// `description.toString` throws during string coercion — propagates to AS.

var throwDesc = new Object();
throwDesc.description = {toString: function() { throw "oops-desc"; }};
throwDesc.extension = "*.txt";
trace("// description toString throws");
try {
    trace(list.browse([throwDesc]));
} catch (e) {
    trace("caught: " + e);
}

// `extension.toString` throws during string coercion — also propagates.

var throwExt = new Object();
throwExt.description = "txt";
throwExt.extension = {toString: function() { throw "oops-ext"; }};
trace("// extension toString throws");
try {
    trace(list.browse([throwExt]));
} catch (e) {
    trace("caught: " + e);
}

// `description` addProperty getter that throws — exception swallowed,
// field treated as missing. Symmetric to the extension getter case.

var throwDescGetter = new Object();
throwDescGetter.addProperty("description", function() { throw "oops-desc-getter"; }, null);
throwDescGetter.extension = "*.txt";
trace("// description getter throws");
try {
    trace(list.browse([throwDescGetter]));
} catch (e) {
    trace("caught: " + e);
}

// `extension` addProperty getter that throws — exception swallowed.

var throwExtGetter = new Object();
throwExtGetter.description = "txt";
throwExtGetter.addProperty("extension", function() { throw "oops-ext-getter"; }, null);
trace("// extension getter throws");
try {
    trace(list.browse([throwExtGetter]));
} catch (e) {
    trace("caught: " + e);
}

// Getter returns an empty string — rejected by the empty-string check
// that runs after the getter is invoked.

var emptyGetter = new Object();
emptyGetter.addProperty("description", function() { return ""; }, null);
emptyGetter.extension = "*.txt";
trace("// description getter returns empty string");
trace(list.browse([emptyGetter]));

// `macType.toString` throws — propagates like description/extension.

var throwMac = new Object();
throwMac.description = "txt";
throwMac.extension = "*.txt";
throwMac.macType = {toString: function() { throw "oops-mac"; }};
trace("// macType toString throws");
try {
    trace(list.browse([throwMac]));
} catch (e) {
    trace("caught: " + e);
}
