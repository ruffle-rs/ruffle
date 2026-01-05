Stage.scaleMode = "noScale";

var nextX = 10;
var nextY = 30;

function renderField(mc, a, b, c, d) {
    var x = nextX;
    var y = nextY;
    var mc2 = mc.createEmptyMovieClip("mc_" + x + "_" + y, mc.getNextHighestDepth());
    mc2.transform.matrix = new flash.geom.Matrix(a, b, c, d, x, y);

    var tf = mc2.createTextField(
            "field_" + x + "_" + y,
            mc2.getNextHighestDepth(),
            0, 0, 5, 5);
    tf.borderColor = 0xFF00FF;
    tf.backgroundColor = 0x0000FF;
    tf.border = true;
    tf.background = false;
    tf.embedFonts = false;

    var tf = mc2.createTextField(
            "field_" + x + "_" + y + "_2",
            mc2.getNextHighestDepth(),
            7, 0, 8, 5);
    tf.borderColor = 0xFF00FF;
    tf.backgroundColor = 0x0000FF;
    tf.border = true;
    tf.background = false;
    tf.embedFonts = false;

    var tf = mc2.createTextField(
            "field_" + x + "_" + y + "_3",
            mc2.getNextHighestDepth(),
            0, 7, 15, 8);
    tf.borderColor = 0xFF00FF;
    tf.backgroundColor = 0x0000FF;
    tf.border = true;
    tf.background = false;
    tf.embedFonts = false;

    nextX += 40;
}

renderField(_root, 1, 0, 0, 1);
renderField(_root, 1, 0.00488281273, 0, 1);
renderField(_root, 1, 0, 0.00488281273, 1);
renderField(_root, 1, 0.00488281273, 0.00488281273, 1);

nextX = 10;
nextY = 50;

renderField(_root, 1, 0, 0, 1);
renderField(_root, 1, -0.00488281273, 0, 1);
renderField(_root, 1, 0, -0.00488281273, 1);
renderField(_root, 1, -0.00488281273, -0.00488281273, 1);

nextX = 10;
nextY = 70;

renderField(_root, 2, 0, 0, 2);
renderField(_root, 2, 0.00488281273, 0, 2);
renderField(_root, 2, 0, 0.00488281273, 2);
renderField(_root, 2, 0.00488281273, 0.00488281273, 2);

nextX = 10;
nextY = 120;

renderField(_root, 2, 0, 0, 2);
renderField(_root, 2, -0.00488281273, 0, 2);
renderField(_root, 2, 0, -0.00488281273, 2);
renderField(_root, 2, -0.00488281273, -0.00488281273, 2);

nextX = 10;
nextY = 170;

var mc = _root.createEmptyMovieClip("outer", _root.getNextHighestDepth());
mc.transform.matrix = new flash.geom.Matrix(1, 500.001, 0, 1, nextX, nextY);
var mc2 = mc.createEmptyMovieClip("inner", mc.getNextHighestDepth());
mc2.transform.matrix = new flash.geom.Matrix(1, -500, 0, 1, 0, 0);
var tf = mc2.createTextField(
        "field_inner",
        mc2.getNextHighestDepth(),
        0, 0, 10, 10);
tf.borderColor = 0xFF00FF;
tf.backgroundColor = 0x0000FF;
tf.border = true;
tf.background = false;
tf.embedFonts = false;
