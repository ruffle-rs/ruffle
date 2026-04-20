trace("shape0");
trace(shape0._x);
trace(shape0._y);
trace(shape0._width);
trace(shape0._height);

trace("shape1");
trace(shape1._x);
trace(shape1._y);
trace(shape1._width);
trace(shape1._height);

shape1.onPress = function() {
    trace("Pressed shape1");
};

trace("_root");
trace(_root._x);
trace(_root._y);
trace(_root._width);
trace(_root._height);

_root.createTextField("text0", 5, -50, -50, 20, 10);
text0.border = true;
text0.background = true;
text0.borderColor = 0x00FFFF;
text0.backgroundColor = 0x00FFFF;

trace("text0");
trace(text0._x);
trace(text0._y);
trace(text0._width);
trace(text0._height);
