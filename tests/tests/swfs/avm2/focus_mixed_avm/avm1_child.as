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

var txt = this.createTextField("txt", 1, 0, 0, 200, 50);
txt.type = "input";

this.createEmptyMovieClip("btn", 4);
btn._y = 50;
drawRect(btn, 0x00FFFF);
btn.onRelease = function() {
    trace("btn clicked");
}

function onSetFocus(oldfocus, newfocus) {
    if (newfocus == null && oldfocus == null) {
        return;
    }
    if (oldfocus == txt) {
        oldfocus = "avm1_child.txt"
    }
    if (newfocus == txt) {
        newfocus = "avm1_child.txt"
    }
    trace("Selection.onSetFocus: " + oldfocus + " " + newfocus);
}

Selection.addListener(this);

txt.onSetFocus = function(o) {
   trace("txt.onSetFocus " + o);
};

txt.onKillFocus = function(o) {
   trace("txt.onKillFocus " + o);
};
