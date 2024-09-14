var values = [
    undefined,
    null,
    true,
    undefined,
    null,
    false,
    1,
    0,
    0.1,
    1.1,
    -1,
    0/0,
    'test',
    'NaN',
    '',
    2147483648,
    4294967296,
    new Object(),
    '1',
    '0'
]

function printValues(caseInsensitive) {
    trace('    _focusrect = ' + _focusrect);
    trace('    _root._focusrect = ' + _root._focusrect);
    trace('    clip._focusrect = ' + clip._focusrect);
    trace('    button._focusrect = ' + button._focusrect);
    trace('    text._focusrect = ' + text._focusrect);
    if (caseInsensitive) {
        trace('    _foCusRect = ' + _foCusRect);
        trace('    _root._foCusRect = ' + _root._foCusRect);
        trace('    clip._foCusRect = ' + clip._foCusRect);
        trace('    button._foCusRect = ' + button._foCusRect);
        trace('    text._foCusRect = ' + text._foCusRect);
    }
}

trace("MovieClip property: " + MovieClip.prototype.hasOwnProperty("_focusrect"));
trace("Button property: " + Button.prototype.hasOwnProperty("_focusrect"));
trace("TextField property: " + TextField.prototype.hasOwnProperty("_focusrect"));

trace('default:');
printValues();

trace('global:');
for (var i in values) {
    trace('  setting _focusrect to "' + values[i] + '"');
    _focusrect = values[i];
    printValues(false);
}
for (var i in values) {
    trace('  setting _foCusRect to "' + values[i] + '"');
    _focusRect = values[i];
    printValues(false);
}

function testFocusRect(obj) {
    for (var i in values) {
        trace('  setting _focusrect to "' + values[i] + '"');
        obj._focusrect = values[i];
        printValues(false);
    }
    for (var i in values) {
        trace('  setting _foCusRect to "' + values[i] + '"');
        obj._foCusRect = values[i];
        printValues(false);
    }
}

trace('_root:');
testFocusRect(_root);
trace('MovieClip:');
testFocusRect(clip);
trace('Button:');
testFocusRect(button);
trace('TextField:');
testFocusRect(text);

trace('Testing _focusrect case-sensitivity');
_focusrect = true;
clip._focusrect = true;
button._focusrect = true;
text._focusrect = true;
printValues(true);
_focusrect = false;
clip._focusrect = false;
button._focusrect = false;
text._focusrect = false;
printValues(true);

trace('Testing _focusrect persistence');
_focusrect = false;
_root.loadMovie("test2.swf");
