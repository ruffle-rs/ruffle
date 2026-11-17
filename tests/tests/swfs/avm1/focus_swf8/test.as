function drawRect(m, color) {
    with(m) {
        beginFill(color);
        lineTo(200, 0);
        lineTo(200, 50);
        lineTo(0, 50);
        lineTo(0, 0);
        endFill();
    }
}

var txt1 = this.createTextField("txt1", 1, 0, 0, 200, 50);
txt1.type = "input";

var txt2 = this.createTextField("txt2", 2, 0, 50, 200, 50);

this.createEmptyMovieClip("mc", 3);
mc._y = 100;
drawRect(mc, 0xFF0000);
mc.focusEnabled = true;
mc.tabEnabled = true;

this.createEmptyMovieClip("btn", 4);
btn._y = 150;
drawRect(btn, 0x00FF00);
btn.onRelease = function() {
    trace("btn clicked");
}

function onSetFocus(oldfocus, newfocus) {
    trace(oldfocus + " " + newfocus);
}

Selection.addListener(this);
