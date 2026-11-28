// Sends data with XML.sendAndLoad and LoadVars.sendAndLoad, one request after
// another, and traces the responses. With `log_fetch`, the test runner also
// traces each request, so the sent data can be compared with what Flash
// Player sends (see README.md).

var url = "http://localhost:8000/send?debug-success";
var urlWithoutQuery = "http://localhost:8000/send";

var xml = new XML("<request a=\"1\">two words</request>");

var overriddenXml = new XML("<request/>");
overriddenXml.toString = function() {
    return "<overridden/>";
};

var vars = new LoadVars();
vars.a = "1";
vars.b = "two words";
vars.c = "x&y=z";

var overriddenVars = new LoadVars();
overriddenVars.a = "1";
overriddenVars.toString = function() {
    return "custom=1";
};

var cases = [
    ["XML", function(target) { return xml.sendAndLoad(url, target); }],
    ["overridden XML", function(target) { return overriddenXml.sendAndLoad(url, target); }],
    ["LoadVars POST", function(target) { return vars.sendAndLoad(url, target, "POST"); }],
    ["LoadVars GET", function(target) { return vars.sendAndLoad(urlWithoutQuery, target, "GET"); }],
    ["overridden LoadVars", function(target) { return overriddenVars.sendAndLoad(url, target, "POST"); }]
];

var index = 0;

function sendNext() {
    if (index == cases.length) {
        trace("Done");
        return;
    }

    var name = cases[index][0];
    var send = cases[index][1];
    index++;

    var target = new LoadVars();
    target.onData = function(src) {
        trace(name + ": onData(" + src + ")");
        sendNext();
    };

    trace(name + ": returned " + send(target));
}

sendNext();
