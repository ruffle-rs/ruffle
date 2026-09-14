var tf = createTextField("tf", getNextHighestDepth(), 0, 0, 100, 100);

trace(tf._listeners.length);
trace(tf._listeners);

tf.name = "test name";
trace(tf._listeners[0].name);
trace(tf._listeners[0] == tf);

var listener = {};
listener.onChanged = function(a, b) {
    trace("onChanged: " + a + "," + b);
};
listener.onKillFocus = function(a, b) {
    trace("onKillFocus: " + a + "," + b);
};
listener.onScroller = function(a, b) {
    trace("onScroller: " + a + "," + b);
};
listener.onSetFocus = function(a, b) {
    trace("onSetFocus: " + a + "," + b);
};
listener.onX = function(a, b) {
    trace("onX: " + a + "," + b);
};
tf.addListener(listener);

trace(tf._listeners.length);
tf._listeners[1].onChanged("a");

trace("// Broadcast");

tf.broadcastMessage("onChanged", "a", "b");
tf.broadcastMessage("onKillFocus", "A", "B");
tf.broadcastMessage("onX", "X");
tf.broadcastMessage("onY", "Y");

trace("// Remove");

tf.removeListener(listener);
trace(tf._listeners.length);

trace("========================");

tf.addListener(listener);

tf.text = "hello";
tf.scroll = 2;
Selection.setFocus(tf);
Selection.setFocus(null);

trace("// Done");
