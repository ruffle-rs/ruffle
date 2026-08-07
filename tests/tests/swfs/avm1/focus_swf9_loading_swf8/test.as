var txt = this.createTextField("txt", 1, 0, 0, 200, 50);
txt.type = "input";

this.createEmptyMovieClip("child", 3);
child._y = 50;

new MovieClipLoader().loadClip("child.swf", child);

function onSetFocus(oldfocus, newfocus) {
    trace(oldfocus + " " + newfocus);
}

Selection.addListener(this);
