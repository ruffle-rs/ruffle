var date = new ArgumentDefinition("date")
    .add(0)
    .add("31")
    .add(32)
    .add(-1)
    .add(NaN)
    .add({}, "{}")
    .add(undefined)
    .add(null)
    .add(true)
    .add(false)
    .add("invalid string")
    .add(Infinity)
    .add(32.5);

var month = new ArgumentDefinition("month")
    .add(0)
    .add("11")
    .add(12)
    .add(-1)
    .add(NaN)
    .add({}, "{}")
    .add(undefined)
    .add(null)
    .add(true)
    .add(false)
    .add("invalid string")
    .add(Infinity)
    .add(11.5);

var year = new ArgumentDefinition("year")
    .add(0)
    .add("2000")
    .add(-200)
    .add(NaN)
    .add({}, "{}")
    .add(undefined)
    .add(null)
    .add(true)
    .add(false)
    .add("invalid string")
    .add(Infinity)
    .add(7002.5);

var hour = new ArgumentDefinition("hour")
    .add(0)
    .add("23")
    .add(24)
    .add(-1)
    .add(NaN)
    .add({}, "{}")
    .add(undefined)
    .add(null)
    .add(true)
    .add(false)
    .add("invalid string")
    .add(Infinity)
    .add(24.5);

var minute = new ArgumentDefinition("minute")
    .add(0)
    .add("59")
    .add(60)
    .add(-1)
    .add(NaN)
    .add({}, "{}")
    .add(undefined)
    .add(null)
    .add(true)
    .add(false)
    .add("invalid string")
    .add(Infinity)
    .add(60.5);

var second = new ArgumentDefinition("second", minute);

var millisecond = new ArgumentDefinition("millisecond")
    .add(0)
    .add("999")
    .add(1000)
    .add(-1)
    .add(NaN)
    .add({}, "{}")
    .add(undefined)
    .add(null)
    .add(true)
    .add(false)
    .add("invalid string")
    .add(Infinity)
    .add(1000.5);

var time = new ArgumentDefinition("time")
    .add(0)
    .add("1594844387653")
    .add(-1594844387653)
    .add(NaN)
    .add({}, "{}")
    .add(undefined)
    .add(null)
    .add(true)
    .add(false)
    .add("invalid string")
    .add(Infinity)
    .add(5.5);

function constructor() {
    switch (arguments.length) {
        case 0: return new Date();
        case 1: return new Date(arguments[0]);
        case 2: return new Date(arguments[0], arguments[1]);
        case 3: return new Date(arguments[0], arguments[1], arguments[2]);
        case 4: return new Date(arguments[0], arguments[1], arguments[2], arguments[3]);
        case 5: return new Date(arguments[0], arguments[1], arguments[2], arguments[3], arguments[4]);
        case 6: return new Date(arguments[0], arguments[1], arguments[2], arguments[3], arguments[4], arguments[5]);
        case 7: return new Date(arguments[0], arguments[1], arguments[2], arguments[3], arguments[4], arguments[5], arguments[6]);
        default: throw "Too many args for constructor! Found " + Utils.stringify(arguments.length);
    }
}

function defaultConstructor() {
    return new Date(1582971753000);
}

function fullRepr(name, date) {
    var result = "";
    if (date instanceof Date) {
        result += name + ": " + Utils.stringify(date) + "\n";
        result += name + ".getDate(): " + Utils.stringify(date.getDate()) + "\n";
        result += name + ".getDay(): " + Utils.stringify(date.getDay()) + "\n";
        result += name + ".getFullYear(): " + Utils.stringify(date.getFullYear()) + "\n";
        result += name + ".getHours(): " + Utils.stringify(date.getHours()) + "\n";
        result += name + ".getMilliseconds(): " + Utils.stringify(date.getMilliseconds()) + "\n";
        result += name + ".getMinutes(): " + Utils.stringify(date.getMinutes()) + "\n";
        result += name + ".getMonth(): " + Utils.stringify(date.getMonth()) + "\n";
        result += name + ".getSeconds(): " + Utils.stringify(date.getSeconds()) + "\n";
        result += name + ".getTime(): " + Utils.stringify(date.getTime()) + "\n";
        result += name + ".getTimezoneOffset(): " + Utils.stringify(date.getTimezoneOffset()) + "\n";
        result += name + ".getUTCDate(): " + Utils.stringify(date.getUTCDate()) + "\n";
        result += name + ".getUTCDay(): " + Utils.stringify(date.getUTCDay()) + "\n";
        result += name + ".getUTCFullYear(): " + Utils.stringify(date.getUTCFullYear()) + "\n";
        result += name + ".getUTCHours(): " + Utils.stringify(date.getUTCHours()) + "\n";
        result += name + ".getUTCMilliseconds(): " + Utils.stringify(date.getUTCMilliseconds()) + "\n";
        result += name + ".getUTCMinutes(): " + Utils.stringify(date.getUTCMinutes()) + "\n";
        result += name + ".getUTCMonth(): " + Utils.stringify(date.getUTCMonth()) + "\n";
        result += name + ".getUTCSeconds(): " + Utils.stringify(date.getUTCSeconds()) + "\n";
        result += name + ".getUTCYear(): " + Utils.stringify(date.getUTCYear()) + "\n";
        result += name + ".getYear(): " + Utils.stringify(date.getYear()) + "\n";
        result += name + ".valueOf(): " + Utils.stringify(date.valueOf()) + "\n";
    } else {
        result += name + ": " + Utils.stringify(date) + "\n";
    }
    trace(result);
}

function partialRepr(name, date) {
    trace(name + ": " + date + " (" + date.valueOf() + ")\n");
}


//var test = new ClassDefinition("Date", Date, defaultConstructor, fullRepr);


// test.testConstructor(constructor, [year, month, date, hour, minute, second, millisecond]);

//test.testStaticMethod("UTC", [year, month, date, hour, minute, second, millisecond]);

//
//
//
//
//
//
//
//
//
//
//
//
//
//