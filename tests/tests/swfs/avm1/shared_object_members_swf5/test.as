// Members of SharedObject available to SWF 5 content, before and after
// clearing the SWF 6 flag (128) of the prototype's members.

var so = SharedObject.getLocal("so");
var protoNames = ["connect", "send", "flush", "close", "getSize", "setFps", "clear", "onStatus", "onSync", "data"];
var staticNames = ["deleteAll", "getDiskUsage", "getLocal", "getRemote", "prototype"];

function traceMembers() {
    for (var i = 0; i < protoNames.length; i++) {
        trace("so." + protoNames[i] + ": " + typeof so[protoNames[i]]);
    }
    for (var i = 0; i < staticNames.length; i++) {
        trace("SharedObject." + staticNames[i] + ": " + typeof SharedObject[staticNames[i]]);
    }
}

trace("// Before");
traceMembers();
ASSetPropFlags(SharedObject.prototype, null, 0, 128);
trace("// After clearing the SWF 6 flag");
traceMembers();
