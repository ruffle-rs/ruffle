
function test1() {
    trace("test1");

    _root.createTextField(
        "text",
        _root.getNextHighestDepth(),
        10, 150, 100, 60);

    var target = text;
    loadVariables("variables", target);
    function checkVariables() {
        if (target.variableA != undefined) {
            trace("Finished loading " + target + ": " + target.variableA);
            clearInterval(interval);
            test2();
        }
    }

    trace("Waiting for variables");
    var interval:Number = setInterval(checkVariables, 100);
}

function test2() {
    trace("test2");

    var target = _root;
    loadVariables("variables", target);
    function checkVariables() {
        if (target.variableA != undefined) {
            trace("Finished loading " + target + ": " + target.variableA);
            clearInterval(interval);
            test3();
        }
    }

    trace("Waiting for variables");
    var interval:Number = setInterval(checkVariables, 100);
}

function test3() {
    trace("test3");

    // Just make sure we don't panic
    loadVariables("variables", []);
    trace("Done");
}

test1();
