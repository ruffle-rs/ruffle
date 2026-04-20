Stage.scaleMode = "noScale";

function renderField(mc, x, y, w, h, r) {
    var tf = mc.createTextField(
            "field_" + x + "_" + y,
            mc.getNextHighestDepth(),
            0, 0, 1, 1);
    tf._x = x;
    tf._y = y;
    tf._width = w;
    tf._height = h;
    tf._rotation = r;
    tf.borderColor = 0xFF00FF;
    tf.backgroundColor = 0x0000FF;
    tf.border = true;
    tf.background = false;
    tf.embedFonts = false;
}

renderField(_root, 10, 10, 10, 10, 0);
renderField(_root, 30, 10, 10, 10, 0.0001);
renderField(_root, 50, 10, 10, 10, 0.001);
renderField(_root, 10, 30, 10, 10, 0.01);
renderField(_root, 30, 30, 10, 10, 0.2);
renderField(_root, 50, 30, 10, 10, 0.3);

var mc = _root.createEmptyMovieClip("inner", _root.getNextHighestDepth());
mc._rotation = 0.1;
mc._x = 0;
mc._y = 35;
renderField(mc, 10, 10, 10, 10, 0);
renderField(mc, 30, 10, 10, 10, 0.0001);
renderField(mc, 50, 10, 10, 10, 0.001);
renderField(mc, 10, 30, 10, 10, 0.01);
renderField(mc, 30, 30, 10, 10, 0.2);
