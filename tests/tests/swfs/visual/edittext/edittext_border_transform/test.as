
function newClip(x, y, a, b, c, d) {
    var mc = _root.createEmptyMovieClip(
            "mc_" + x + "_" + y,
            _root.getNextHighestDepth());
    mc.transform.matrix = new flash.geom.Matrix(a, b, c, d, x, y);
    return mc;
}

function renderField(mc, x, y, w, h, sx, sy, r) {
    var tf = mc.createTextField(
            "field_" + x + "_" + y,
            mc.getNextHighestDepth(),
            0, 0, 1, 1);
    tf._x = x;
    tf._y = y;
    tf._width = w;
    tf._height = h;
    tf._xscale = sx;
    tf._yscale = sy;
    tf._rotation = r;
    tf.borderColor = 0xFF00FF;
    tf.backgroundColor = 0x0000FF;

    var listener = new Object();
    listener.onKeyDown = function() {
        if (Key.getCode() == 49) {
            tf.border = true;
            tf.background = false;
            tf.embedFonts = false;
        }
        if (Key.getCode() == 50) {
            tf.border = false;
            tf.background = true;
            tf.embedFonts = false;
        }
        if (Key.getCode() == 51) {
            tf.border = true;
            tf.background = true;
            tf.embedFonts = false;
        }
        if (Key.getCode() == 52) {
            tf.border = true;
            tf.background = false;
            tf.embedFonts = true;
        }
        if (Key.getCode() == 53) {
            tf.border = false;
            tf.background = true;
            tf.embedFonts = true;
        }
        if (Key.getCode() == 54) {
            tf.border = true;
            tf.background = true;
            tf.embedFonts = true;
        }
    };
    Key.addListener(listener);
}

// Positive scale
renderField(_root, 5, 5, 12, 12, 100, 100, 0);
renderField(_root, 20, 5, 6, 12, 200, 100, 0);
renderField(_root, 35, 5, 12, 6, 100, 200, 0);

// Negative scale
renderField(_root, 5 + 12, 20 + 12, 12, 12, -100, -100, 0);
renderField(_root, 20 + 12, 20 + 12, 6, 12, -200, -100, 0);
renderField(_root, 35 + 12, 20 + 12, 12, 6, -100, -200, 0);

// Rotation
renderField(_root, 5 + 12, 35, 12, 12, 100, 100, 90);
renderField(_root, 20 + 12, 35 + 12, 6, 12, 200, 100, 180);
renderField(_root, 35 + 12, 35, 12, 6, 100, 200, 45);

// Shear
var mc;
mc = newClip(5, 60, 1, 0.5, 0, 1);
renderField(mc, 0, 0, 12, 12, 100, 100, 0);
mc = newClip(35, 60, 1, 0, 0.5, 1);
renderField(mc, 0, 0, 12, 12, 100, 100, 0);
