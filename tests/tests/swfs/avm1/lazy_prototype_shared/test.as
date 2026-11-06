// Flash Player defines some properties of native classes lazily, on the
// prototype of a newly created native instance, at most once per prototype
// object. This tests whether that is tracked separately for each class, with
// Video, TextFormat and PrintJob.
//
// test.swf (from test.swfasm) places a Video as `v2` on frame 2. Going to
// frame 1 and back to frame 2 creates a new Video.
function desc(o, name) {
    return !o.hasOwnProperty(name) ? "-" : typeof o[name];
}
function has(o) {
    return "font=" + desc(o, "font") + " size=" + desc(o, "size") + " width=" + desc(o, "width") + " height=" + desc(o, "height");
}
function placeVideo() {
    _root.gotoAndStop(1);
    _root.gotoAndStop(2);
    return _root.v2;
}
if (this.started == undefined) {
    this.started = true;
    stop();
    this.onEnterFrame = function() {
        delete this.onEnterFrame;

        var p1 = {};
        TextFormat.prototype = p1;
        Video.prototype = p1;
        var tf = new TextFormat();
        trace("p1, after new TextFormat: " + has(p1));
        var v = placeVideo();
        trace("p1, after placing a Video: " + has(p1) + ", v.__proto__ == p1: " + (v.__proto__ == p1));

        var p2 = {};
        TextFormat.prototype = p2;
        Video.prototype = p2;
        v = placeVideo();
        trace("p2, after placing a Video: " + has(p2) + ", v.__proto__ == p2: " + (v.__proto__ == p2));
        tf = new TextFormat();
        trace("p2, after new TextFormat: " + has(p2) + ", tf.__proto__ == p2: " + (tf.__proto__ == p2));

        // Three classes, deleting the properties in between.
        var p3 = {};
        Video.prototype = p3;
        TextFormat.prototype = p3;
        PrintJob.prototype = p3;
        v = placeVideo();
        trace("p3, after placing a Video: " + has(p3) + " orientation=" + desc(p3, "orientation"));
        delete p3.width;
        delete p3.height;
        tf = new TextFormat();
        trace("p3, deleted width and height, after new TextFormat: " + has(p3) + " orientation=" + desc(p3, "orientation"));
        var pj = new PrintJob();
        trace("p3, after new PrintJob: " + has(p3) + " orientation=" + desc(p3, "orientation"));
        trace("done");
    };
}
