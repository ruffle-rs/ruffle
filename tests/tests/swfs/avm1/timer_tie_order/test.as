// Keep the number of timers created below 127: with more, Flash Player was
// observed to occasionally run timers due at the same time out of creation
// order.

var DELAY = 50;
// Far enough apart that a pause (e.g. GC) does not merge groups of delays.
var GROUP_GAP = 250;
var CASE_DURATION = 1000;

function log(name) {
    trace(name);
}

function intervalOnce(name) {
    var id = setInterval(function() {
        log(name);
        clearInterval(id);
    }, DELAY);
}

function startInterval(name) {
    var repeat = 0;
    var id = setInterval(function() {
        log(name + " repeat " + repeat);
        repeat++;
        if (repeat == 3) {
            clearInterval(id);
        }
    }, DELAY);
}

function spawn(name) {
    log(name);
    setTimeout(log, DELAY, name + "1");
    setTimeout(log, DELAY, name + "2");
}

function sameDelay() {
    for (var i = 0; i < 8; i++) {
        setTimeout(log, DELAY, "timeout " + i);
    }
}

function mixedDelays() {
    var multipliers = [2, 1, 3, 1, 2, 3, 1, 2];
    for (var i = 0; i < multipliers.length; i++) {
        setTimeout(log, GROUP_GAP * multipliers[i], "timeout " + i + " x" + multipliers[i]);
    }
}

var target = {};
target.fire = function(name) {
    log(name);
    clearInterval(this.id);
};

function mixedKinds() {
    setTimeout(log, DELAY, "setTimeout 0");
    intervalOnce("setInterval 1");
    target.id = setInterval(target, "fire", DELAY, "setInterval method 2");
    setTimeout(target, "fire", DELAY, "setTimeout method 3");
    setTimeout(log, DELAY, "setTimeout 4");
}

function intervals() {
    for (var i = 0; i < 3; i++) {
        startInterval("interval " + i);
    }
}

function cleared() {
    var ids = [];
    ids.push(setTimeout(function() {
        log("timeout 0, clearing 3");
        clearTimeout(ids[3]);
    }, DELAY));
    for (var i = 1; i < 6; i++) {
        ids.push(setTimeout(log, DELAY, "timeout " + i));
    }
    clearTimeout(ids[2]);
    clearInterval(ids[4]);
}

function recreated() {
    var a = setTimeout(log, DELAY, "timeout A");
    setTimeout(log, DELAY, "timeout B");
    setTimeout(log, DELAY, "timeout C");
    clearTimeout(a);
    setTimeout(log, DELAY, "timeout A");
}

function createdInCallbacks() {
    setTimeout(spawn, DELAY, "P");
    setTimeout(spawn, DELAY, "Q");
}

function overdueTogether() {
    // The gap is larger than a step of the clock, which can happen between
    // the two calls, so the 10ms timeout is always due first.
    setTimeout(log, 30, "timeout 30ms");
    setTimeout(log, 10, "timeout 10ms");
    // Make both overdue before timers are checked again, with room for
    // a step of the clock.
    var start = getTimer();
    while (getTimer() - start < 50) {
    }
}

function overdueIntervals() {
    var seen = {};
    var ids = [];
    var run = function(name) {
        // Stop at the first repeat.
        if (seen[name]) {
            for (var i = 0; i < ids.length; i++) {
                clearInterval(ids[i]);
            }
            return;
        }
        seen[name] = true;
        log(name);
    };
    ids.push(setInterval(run, 30, "interval 30ms"));
    ids.push(setInterval(run, 10, "interval 10ms"));
    // Make both overdue before timers are checked again, with room for
    // a step of the clock.
    var start = getTimer();
    while (getTimer() - start < 50) {
    }
}

var cases = [
    "same delay", sameDelay,
    "mixed delays", mixedDelays,
    "mixed kinds", mixedKinds,
    "intervals", intervals,
    "cleared", cleared,
    "recreated", recreated,
    "created in callbacks", createdInCallbacks,
    "overdue together", overdueTogether,
    "overdue intervals", overdueIntervals
];

function nextCase() {
    if (cases.length == 0) {
        trace("Done");
        return;
    }
    trace("// " + cases.shift());
    var run = cases.shift();
    run();
    setTimeout(nextCase, CASE_DURATION);
}

nextCase();
