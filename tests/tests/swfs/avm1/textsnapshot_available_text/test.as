trace("_root: " + new TextSnapshot(_root).getText(0, 100));

trace("// Add a dynamic text field.");
createTextField("tf", getNextHighestDepth(), 0, 0, 10, 10);
tf.text = "TF";

trace("_root: " + new TextSnapshot(_root).getText(0, 100));
var tsChild = new TextSnapshot(child);
trace("child: " + tsChild.getText(0, 100));

trace("// Duplicate child.");
duplicateMovieClip("child", "child_clone", getNextHighestDepth());

trace("_root: " + new TextSnapshot(_root).getText(0, 100));
trace("old child: " + tsChild.getText(0, 100));
trace("child: " + new TextSnapshot(child).getText(0, 100));
var tsChildClone = new TextSnapshot(child_clone);
trace("child_clone: " + tsChildClone.getText(0, 100));

trace("// Duplicate child_clone.");
duplicateMovieClip("child_clone", "child_clone2", getNextHighestDepth());

trace("old child: " + tsChild.getText(0, 100));
trace("child: " + new TextSnapshot(child).getText(0, 100));
trace("old child_clone: " + tsChildClone.getText(0, 100));
trace("child_clone: " + new TextSnapshot(child_clone).getText(0, 100));
trace("child_clone2: " + new TextSnapshot(child_clone2).getText(0, 100));

trace("// Duplicate child again.");
duplicateMovieClip("child", "child_clone3", getNextHighestDepth());

trace("child: " + new TextSnapshot(child).getText(0, 100));
trace("child_clone: " + new TextSnapshot(child_clone).getText(0, 100));
trace("child_clone2: " + new TextSnapshot(child_clone2).getText(0, 100));
trace("child_clone3: " + new TextSnapshot(child_clone3).getText(0, 100));
