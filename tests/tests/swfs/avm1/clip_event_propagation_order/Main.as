class Main extends MovieClip {
    function onLoad() {
        trace("Main onLoad");

        // Create children at different depths
        attachMovie("child", "child_depth10", 10, {childName: "child_depth10"});
        attachMovie("child", "child_depth20", 20, {childName: "child_depth20"});
        attachMovie("child", "child_depth30", 30, {childName: "child_depth30"});

        trace("Children created");
    }

    function onMouseDown() {
        trace("Main onMouseDown");
    }

    function onMouseUp() {
        trace("Main onMouseUp");
    }

    function onMouseMove() {
        trace("Main onMouseMove");
    }
}
