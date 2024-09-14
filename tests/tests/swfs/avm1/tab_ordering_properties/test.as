function printProperties(obj) {
    trace('    tabEnabled = ' + obj.tabEnabled);
    trace('    tabIndex = ' + obj.tabIndex);
    trace('    tabChildren = ' + obj.tabChildren);
    for (var i in obj) {
        if (i == 'tabEnabled') {
            trace('    enumerated tabEnabled');
        }
    }
    for (var i in obj) {
        if (i == 'tabIndex') {
            trace('    enumerated tabIndex');
        }
    }
    for (var i in obj) {
        if (i == 'tabChildren') {
            trace('    enumerated tabChildren');
        }
    }
}

function testProperties(obj) {
    trace('  default');
    printProperties(obj);

    obj.tabEnabled = true;
    obj.tabIndex = 0;
    obj.tabChildren = true;

    trace('  after set 1');
    printProperties(obj);

    obj.tabEnabled = false;
    obj.tabIndex = 4;
    obj.tabChildren = false;

    trace('  after set 2');
    printProperties(obj);

    obj.tabEnabled = undefined;
    obj.tabIndex = undefined;
    obj.tabChildren = undefined;

    trace('  after set 3');
    printProperties(obj);

    obj.tabEnabled = -4;
    obj.tabIndex = -4;
    obj.tabChildren = -4;

    trace('  after set 4');
    printProperties(obj);

    obj.tabEnabled = 2147483647;
    obj.tabIndex = 2147483647;
    obj.tabChildren = 2147483647;

    trace('  after set 5');
    printProperties(obj);

    obj.tabEnabled = 2147483648;
    obj.tabIndex = 2147483648;
    obj.tabChildren = 2147483648;

    trace('  after set 6');
    printProperties(obj);

    obj.tabEnabled = 'x';
    obj.tabIndex = 'x';
    obj.tabChildren = 'x';

    trace('  after set 7');
    printProperties(obj);

    obj.tabEnabled = -2147483648;
    obj.tabIndex = -2147483648;
    obj.tabChildren = -2147483648;

    trace('  after set 8');
    printProperties(obj);

    obj.tabEnabled = new Object();
    obj.tabIndex = new Object();
    obj.tabChildren = new Object();

    trace('  after set 9');
    printProperties(obj);

    obj.tabEnabled = 1.1;
    obj.tabIndex = 1.1;
    obj.tabChildren = 1.1;

    trace('  after set 10');
    printProperties(obj);
}

trace('===== text =====');
testProperties(text);
trace('===== non-editable text =====');
text2.type = "dynamic";
testProperties(text2);
trace('===== button =====');
testProperties(button);
trace('===== movie clip =====');
testProperties(_root);
