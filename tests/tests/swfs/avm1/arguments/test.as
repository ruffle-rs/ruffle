function dump() {
    trace("// arguments");
    trace(arguments);
    trace("");

    trace("// arguments.length");
    trace(arguments.length);
    trace("");

    trace("// arguments instanceof Array");
    trace(arguments instanceof Array);
    trace("");

    trace("// arguments.__proto__ === Array.prototype");
    trace(arguments.__proto__ === Array.prototype);
    trace("");

    trace("// arguments.callee");
    trace(arguments.callee);
    trace("");

    trace("// arguments.callee === dump");
    trace(arguments.callee === dump);
    trace("");

    trace("// arguments.caller");
    trace(arguments.caller);
    trace("");

    trace("// arguments.caller === indirectDump");
    trace(arguments.caller === indirectDump);
    trace("");

    for (var key in arguments) {
        trace("// arguments[" + key + "]");
        trace(arguments[key]);
        trace("");
    }
}

function indirectDump() {
    trace("// dump.apply(dump, arguments)");
    dump.apply(dump, arguments);
}

trace("// dump()");
dump();
trace("");
trace("");

trace("// dump(\"a\")");
dump("a");
trace("");
trace("");

trace("// dump(\"a\", \"b\")");
dump("a", "b");
trace("");
trace("");

trace("// indirectDump(\"a\", \"b\", undefined, \"d\")");
indirectDump("a", "b", undefined, "d");
trace("");
trace("");

fscommand("quit");
