class Child extends MovieClip {
    var childName:String;

    function onLoad() {
        trace(childName + " onLoad");
    }

    function onMouseDown() {
        trace(childName + " onMouseDown");
    }

    function onMouseUp() {
        trace(childName + " onMouseUp");
    }

    function onMouseMove() {
        trace(childName + " onMouseMove");
    }
}
