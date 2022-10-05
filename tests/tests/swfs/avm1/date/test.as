function traceDate(date) {
    trace(date);
    trace(
        "FullYear = " + date.getFullYear() +
        ", Year = " + date.getYear() +
        ", Month = " + date.getMonth() +
        ", Date = " + date.getDate() +
        ", Day = " + date.getDay() +
        ", Hours = " + date.getHours() +
        ", Minutes = " + date.getMinutes() +
        ", Seconds = " + date.getSeconds() +
        ", Milliseconds = " + date.getMilliseconds() +
        ", Time = " + date.getTime() +
        ", TimezoneOffset = " + date.getTimezoneOffset() +
        ", UTCFullYear = " + date.getUTCFullYear() +
        ", UTCYear = " + date.getUTCYear() +
        ", UTCMonth = " + date.getUTCMonth() +
        ", UTCDate = " + date.getUTCDate() +
        ", UTCDay = " + date.getUTCDay() +
        ", UTCHours = " + date.getUTCHours() +
        ", UTCMinutes = " + date.getUTCMinutes() +
        ", UTCSeconds = " + date.getUTCSeconds() +
        ", UTCMilliseconds = " + date.getUTCMilliseconds()
    );
}

function testConstructor() {
    // Test invocation without `new`.
    trace("// Date()");
    trace(Date());
    trace("");

    trace("// typeof Date()");
    trace(typeof Date());
    trace("");

    // Test no arguments.
    trace("// new Date()");
    traceDate(new Date());
    trace("");

    // Test construction from another date.
    trace("// new Date(new Date())");
    var date = new Date();
    traceDate(new Date(date));
    trace("");

    // Test 1 argument.
    var times = [
        0,
        5.5,
        500,
        -500,
        1000,
        -1000,
        1594844387653,
        -1594844387653,
        8.64e15,
        8.64e15 + 1,
        -8.64e15,
        -8.64e15 - 1,
        NaN,
        Infinity,
        -Infinity
    ];
    for (var i = 0; i < times.length; i++) {
        trace("// new Date(" + times[i] + ")");
        traceDate(new Date(times[i]));
        trace("");
    }

    // Test 2 arguments.
    trace("// new Date(1, 2)");
    traceDate(new Date(1, 2));
    trace("");

    // Test 3 arguments.
    trace("// new Date(1, 2, 3)");
    traceDate(new Date(1, 2, 3));
    trace("");

    // Test 4 arguments.
    trace("// new Date(1, 2, 3, 4)");
    traceDate(new Date(1, 2, 3, 4));
    trace("");

    // Test 5 arguments.
    trace("// new Date(1, 2, 3, 4, 5)");
    traceDate(new Date(1, 2, 3, 4, 5));
    trace("");

    // Test 6 arguments.
    trace("// new Date(1, 2, 3, 4, 5, 6)");
    traceDate(new Date(1, 2, 3, 4, 5, 6));
    trace("");

    // Test 7 arguments.
    var argsList = [
        [0, 0, 0, 0, 0, 0, 0],
        [2000, 11, 31, 23, 59, 59, 999],
        [7002.5, 13.5, 32.5, 24.5, 60.5, 60.5, 1000.5],
        [-1, -2, -3, -4, -5, -6, -7],
        [123456789, 123, 456, 789, 123, 456, 2000],

        [NaN, NaN, NaN, NaN, NaN, NaN, NaN],
        [NaN, 0, 0, 0, 0, 0, 0],
        [0, NaN, 0, 0, 0, 0, 0],
        [0, 0, NaN, 0, 0, 0, 0],
        [0, 0, 0, NaN, 0, 0, 0],
        [0, 0, 0, 0, NaN, 0, 0],
        [0, 0, 0, 0, 0, NaN, 0],
        [0, 0, 0, 0, 0, 0, NaN],

        [Infinity, Infinity, Infinity, Infinity, Infinity, Infinity, Infinity],
        [Infinity, 0, 0, 0, 0, 0, 0],
        [0, Infinity, 0, 0, 0, 0, 0],
        [0, 0, Infinity, 0, 0, 0, 0],
        [0, 0, 0, Infinity, 0, 0, 0],
        [0, 0, 0, 0, Infinity, 0, 0],
        [0, 0, 0, 0, 0, Infinity, 0],
        [0, 0, 0, 0, 0, 0, Infinity],

        [-Infinity, -Infinity, -Infinity, -Infinity, -Infinity, -Infinity, -Infinity],
        [-Infinity, 0, 0, 0, 0, 0, 0],
        [0, -Infinity, 0, 0, 0, 0, 0],
        [0, 0, -Infinity, 0, 0, 0, 0],
        [0, 0, 0, -Infinity, 0, 0, 0],
        [0, 0, 0, 0, -Infinity, 0, 0],
        [0, 0, 0, 0, 0, -Infinity, 0],
        [0, 0, 0, 0, 0, 0, -Infinity]
    ];
    for (var i = 0; i < argsList.length; i++) {
        var args = argsList[i];
        trace("// new Date(" + args.join(", ") + ")");
        traceDate(new Date(args[0], args[1], args[2], args[3], args[4], args[5], args[6]));
        trace("");
    }
}

function testUTC() {
    // Test no arguments.
    trace("// Date.UTC()");
    trace(Date.UTC());
    trace("");

    // Test 1 argument.
    trace("// Date.UTC(1)");
    trace(Date.UTC(1));
    trace("");

    // Test 2 arguments.
    trace("// Date.UTC(1, 2)");
    trace(Date.UTC(1, 2));
    trace("");

    // Test 3 arguments.
    trace("// Date.UTC(1, 2, 3)");
    trace(Date.UTC(1, 2, 3));
    trace("");

    // Test 4 arguments.
    trace("// Date.UTC(1, 2, 3, 4)");
    trace(Date.UTC(1, 2, 3, 4));
    trace("");

    // Test 5 arguments.
    trace("// Date.UTC(1, 2, 3, 4, 5)");
    trace(Date.UTC(1, 2, 3, 4, 5));
    trace("");

    // Test 6 arguments.
    trace("// Date.UTC(1, 2, 3, 4, 5, 6)");
    trace(Date.UTC(1, 2, 3, 4, 5, 6));
    trace("");

    // Test 7 arguments.
    var argsList = [
        [0, 0, 0, 0, 0, 0, 0],
        [2000, 11, 31, 23, 59, 59, 999],
        [7002.5, 13.5, 32.5, 24.5, 60.5, 60.5, 1000.5],
        [-1, -2, -3, -4, -5, -6, -7],
        [123456789, 123, 456, 789, 123, 456, 2000],

        [NaN, 0, 0, 0, 0, 0, 0],
        [0, NaN, 0, 0, 0, 0, 0],
        [0, 0, NaN, 0, 0, 0, 0],
        [0, 0, 0, NaN, 0, 0, 0],
        [0, 0, 0, 0, NaN, 0, 0],
        [0, 0, 0, 0, 0, NaN, 0],
        [0, 0, 0, 0, 0, 0, NaN],

        [Infinity, 0, 0, 0, 0, 0, 0],
        [0, Infinity, 0, 0, 0, 0, 0],
        [0, 0, Infinity, 0, 0, 0, 0],
        [0, 0, 0, Infinity, 0, 0, 0],
        [0, 0, 0, 0, Infinity, 0, 0],
        [0, 0, 0, 0, 0, Infinity, 0],
        [0, 0, 0, 0, 0, 0, Infinity],

        [-Infinity, 0, 0, 0, 0, 0, 0],
        [0, -Infinity, 0, 0, 0, 0, 0],
        [0, 0, -Infinity, 0, 0, 0, 0],
        [0, 0, 0, -Infinity, 0, 0, 0],
        [0, 0, 0, 0, -Infinity, 0, 0],
        [0, 0, 0, 0, 0, -Infinity, 0],
        [0, 0, 0, 0, 0, 0, -Infinity]
    ];
    for (var i = 0; i < argsList.length; i++) {
        var args = argsList[i];
        trace("// Date.UTC(" + args.join(", ") + ")");
        trace(Date.UTC(args[0], args[1], args[2], args[3], args[4], args[5], args[6]));
        trace("");
    }
}

function testSetDay(name, offset) {
    // Test no arguments.
    trace("// var date = new Date(1594844387653);");
    var date = new Date(1594844387653);

    trace("// date. " + name + "()");
    trace(date[name]());

    trace("// date");
    traceDate(date);
    trace("");

    var argsList = [
        [1594844387653, 0, 0, 0],
        [1594844387653, 2000, 11, 31],
        [1594844387653, 7002.5, 13.5, 32.5],
        [1594844387653, -1, -2, -3],
        [1594844387653, 123456789, 123, 456],
        [1594844387653, NaN, 0, 0],
        [1594844387653, 0, NaN, 0],
        [1594844387653, 0, 0, NaN],
        [1594844387653, Infinity, 0, 0],
        [1594844387653, 0, Infinity, 0],
        [1594844387653, 0, 0, Infinity],
        [1594844387653, -Infinity, 0, 0],
        [1594844387653, 0, -Infinity, 0],
        [1594844387653, 0, 0, -Infinity],
        [0, 2000, 11, 31],
        [-1594844387653, 2000, 11, 31],
        [8.64e15 + 1, 2000, 11, 31],
        [-8.64e15, 2000, 11, 31],
        [-8.64e15 - 1, 2000, 11, 31],
        [NaN, 2000, 11, 31],
        [Infinity, 2000, 11, 31],
        [-Infinity, 2000, 11, 31]
    ];

    // Test 1 argument.
    for (var i = 0; i < argsList.length; i++) {
        var args = argsList[i];

        trace("// var date = new Date(" + args[0] + ");");
        var date = new Date(args[0]);

        trace("// date." + name + "(" + args[offset + 1] + ")");
        trace(date[name](args[offset + 1]));

        trace("// date");
        traceDate(date);
        trace("");
    }

    if (offset < 2) {
        // Test 2 arguments.
        for (var i = 0; i < argsList.length; i++) {
            var args = argsList[i];

            trace("// var date = new Date(" + args[0] + ");");
            var date = new Date(args[0]);

            trace("// date. " + name + "(" + args[offset + 1] + ", " + args[offset + 2] + ")");
            trace(date[name](args[offset + 1], args[offset + 2]));

            trace("// date");
            traceDate(date);
            trace("");
        }
    }

    if (offset < 1) {
        // Test 3 arguments.
        for (var i = 0; i < argsList.length; i++) {
            var args = argsList[i];

            trace("// var date = new Date(" + args[0] + ");");
            var date = new Date(args[0]);

            trace("// date. " + name + "(" + args[offset + 1] + ", " + args[offset + 2] + ", " + args[offset + 3] + ")");
            trace(date[name](args[offset + 1], args[offset + 2], args[offset + 3]));

            trace("// date");
            traceDate(date);
            trace("");
        }
    }
}

function testSetTime(name, offset) {
    // Test no arguments.
    trace("// var date = new Date(1594844387653);");
    var date = new Date(1594844387653);

    trace("// date. " + name + "()");
    trace(date[name]());

    trace("// date");
    traceDate(date);
    trace("");

    var argsList = [
        [1594844387653, 0, 0, 0, 0],
        [1594844387653, 23, 59, 59, 999],
        [1594844387653, 24.5, 60.5, 60.5, 1000.5],
        [1594844387653, -4, -5, -6, -7],
        [1594844387653, 123, 456, 789, 2000],
        [1594844387653, NaN, 0, 0, 0],
        [1594844387653, 0, NaN, 0, 0],
        [1594844387653, 0, 0, NaN, 0],
        [1594844387653, 0, 0, 0, NaN],
        [1594844387653, Infinity, 0, 0, 0],
        [1594844387653, 0, Infinity, 0, 0],
        [1594844387653, 0, 0, Infinity, 0],
        [1594844387653, 0, 0, 0, Infinity],
        [1594844387653, -Infinity, 0, 0, 0],
        [1594844387653, 0, -Infinity, 0, 0],
        [1594844387653, 0, 0, -Infinity, 0],
        [1594844387653, 0, 0, 0, -Infinity],
        [0, 23, 59, 59, 999],
        [-1594844387653, 23, 59, 59, 999],
        [8.64e15 + 1, 23, 59, 59, 999],
        [-8.64e15, 23, 59, 59, 999],
        [-8.64e15 - 1, 23, 59, 59, 999],
        [NaN, 23, 59, 59, 999],
        [Infinity, 23, 59, 59, 999],
        [-Infinity, 23, 59, 59, 999]
    ];

    // Test 1 argument.
    for (var i = 0; i < argsList.length; i++) {
        var args = argsList[i];

        trace("// var date = new Date(" + args[0] + ");");
        var date = new Date(args[0]);

        trace("// date. " + name + "(" + args[1] + ")");
        trace(date[name](args[1]));

        trace("// date");
        traceDate(date);
        trace("");
    }

    if (offset < 3) {
        // Test 2 arguments.
        for (var i = 0; i < argsList.length; i++) {
            var args = argsList[i];

            trace("// var date = new Date(" + args[0] + ");");
            var date = new Date(args[0]);

            trace("// date. " + name + "(" + args[offset + 1] + ", " + args[offset + 2] + ")");
            trace(date[name](args[offset + 1], args[offset + 2]));

            trace("// date");
            traceDate(date);
            trace("");
        }
    }

    if (offset < 2) {
        // Test 3 arguments.
        for (var i = 0; i < argsList.length; i++) {
            var args = argsList[i];

            trace("// var date = new Date(" + args[0] + ");");
            var date = new Date(args[0]);

            trace("// date. " + name + "(" + args[offset + 1] + ", " + args[offset + 2] + ", " + args[offset + 3] + ")");
            trace(date[name](args[offset + 1], args[offset + 2], args[offset + 3]));

            trace("// date");
            traceDate(date);
            trace("");
        }
    }

    if (offset < 1) {
        // Test 4 arguments.
        for (var i = 0; i < argsList.length; i++) {
            var args = argsList[i];

            trace("// var date = new Date(" + args[0] + ");");
            var date = new Date(args[0]);

            trace("// date. " + name + "(" + args[offset + 1] + ", " + args[offset + 2] + ", " + args[offset + 3] + ", " + args[offset + 4] + ")");
            trace(date[name](args[offset + 1], args[offset + 2], args[offset + 3], args[offset + 4]));

            trace("// date");
            traceDate(date);
            trace("");
        }
    }
}

function testSetTimestamp() {
    // Test no arguments.
    trace("// var date = new Date(1594844387653);");
    var date = new Date(1594844387653);

    trace("// date.setTime()");
    trace(date.setTime());

    trace("// date");
    traceDate(date);
    trace("");

    // Test 1 argument.
    var times = [
        0,
        5.5,
        500,
        -500,
        1000,
        -1000,
        1594844387653,
        -1594844387653,
        8.64e15,
        8.64e15 + 1,
        -8.64e15,
        -8.64e15 - 1,
        NaN,
        Infinity,
        -Infinity
    ];
    for (var i = 0; i < times.length; i++) {
        trace("// var date = new Date(1594844387653);");
        var date = new Date(1594844387653);

        trace("// date.setTime(" + times[i] + ")");
        trace(date.setTime(times[i]));

        trace("// date");
        traceDate(date);
        trace("");
    }
}

// TODO: Assert that `Date.prototype.valueOf === Date.prototype.getTime`.

testConstructor();
testUTC();
testSetDay("setFullYear", 0);
testSetDay("setMonth", 1);
testSetDay("setDate", 2);
testSetTime("setHours", 0);
testSetTime("setMinutes", 1);
testSetTime("setSeconds", 2);
testSetTime("setMilliseconds", 3);
testSetTimestamp();
testSetDay("setYear", 0);
testSetDay("setUTCFullYear", 0);
testSetDay("setUTCMonth", 1);
testSetDay("setUTCDate", 2);
testSetTime("setUTCHours", 0);
testSetTime("setUTCMinutes", 1);
testSetTime("setUTCSeconds", 2);
testSetTime("setUTCMilliseconds", 3);
