var s = new Sound();
var t = s.getTransform();

for (var i in t) {
    trace(i + ": " + t[i]);
}

fscommand("quit");
