function printProps(obj) {
    trace(obj);
    trace("  x = " + obj._x);
    trace("  y = " + obj._y);
    trace("  w = " + obj._width);
    trace("  h = " + obj._height);
}

function testCreateTextField(i, value) {
    printProps(_root.createTextField(
        "text" + i,
        _root.getNextHighestDepth(),
        value, value, value, value));
}

var text1 = _root.createTextField(
    "text1",
    _root.getNextHighestDepth(),
    2.1, 3.94, 2.1, 3.94);
printProps(text1);

text1._x = 2.1;
text1._y = 3.94;
text1._width = 2.1;
text1._height = 3.94;
printProps(text1);

var i = 2;
testCreateTextField(i++, 0);
testCreateTextField(i++, "0");
testCreateTextField(i++, 0.1);
testCreateTextField(i++, true);
testCreateTextField(i++, false);
testCreateTextField(i++, "");
testCreateTextField(i++, new Object());
testCreateTextField(i++, -2);
testCreateTextField(i++, -2.9);
testCreateTextField(i++, "-2");
testCreateTextField(i++, "text");
testCreateTextField(i++, null);
testCreateTextField(i++, undefined);
testCreateTextField(i++, 1.0 / 0.0);
testCreateTextField(i++, 0.0 / 0.0);

var tf = _root.createTextField(
    "text" + i++,
    _root.getNextHighestDepth(),
    50, 50, -50, -50);
printProps(tf);
tf.background = true;
tf.backgroundColor = 0;
