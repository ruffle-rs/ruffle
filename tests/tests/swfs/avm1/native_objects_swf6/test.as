var FAKE_DATE_PROTO = {
    __proto__: Date.prototype,
    __constructor__: Date,
    __initializeNative: function() {
        super();
    }
};

function getNativeStatus(obj) {
    var t = typeof obj;
    if (t !== "object" && t !== "movieclip" && t !== "function") {
        return "non-object: " + t;
    }

    var original = obj.__proto__;

    // Some objects have read-only prototypes, so we need to handle that.
    obj.__proto__ = null;
    var readOnly = obj.__proto__ !== null;
    if (readOnly) {
        ASSetPropFlags(obj, "__proto__", 0, 4 /* readonly */);
    }

    // A native object cannot be initialized twice, so try to initialize
    // Date, and check if it worked.
    obj.__proto__ = FAKE_DATE_PROTO;
    if(obj.__proto__ !== FAKE_DATE_PROTO) {
        return "ERROR: failed to set __proto__";
    }
    obj.__initializeNative();
    var native = (typeof Date.prototype.getDate.call(obj)) === "undefined";


    obj.__proto__ = original; // restore

    // build status string
    var status = native ? "native" : "non-native";
    if (t !== "object") status += "-ish (" + t + ")";
    if (readOnly) status += " (readonly proto)";
    return status;
}

function check(name, value) {
    trace(name + ": " + getNativeStatus(value));
}

check("{}", {});
check("[]", []);
check("\"\"", "");
check("5", 5);
check("function() {}", function() {});
check("true", true);
check("null", null);
check("undefined", undefined);

check("Accessibility", Accessibility);
check("new Array()", new Array());
check("new Function()", new Function());
check("Function()", Function());
check("new Object()", new Object());
check("Object()", Object());
check("new String()", new String());
check("new Number()", new Number());
check("new Number(3)", new Number(3));
check("new Boolean()", new Boolean());
check("new Boolean(true)", new Boolean(true));

check("new Button()", new Button());
check("timelineButton", timelineButton);
check("new Camera()", new Camera());
check("new Color()", new Color());
check("new Color(_root)", new Color(_root));
check("new ContextMenu()", new ContextMenu());
check("new ContextMenuItem()", new ContextMenuItem());
check("new ContextMenuItem([...])", new ContextMenuItem("name", function(){}));
// can't check Date, it's our measuring device
check("new Error()", new Error());
check("new Error(\"e\")", new Error("e"));
check("new Function()", new Function());
check("Key", Key);
check("new LoadVars()", new LoadVars());
check("new LocalConnection()", new LocalConnection());
check("Math", Math);
check("new Microphone()", new Microphone());
check("Mouse", Mouse);
check("new MovieClip()", new MovieClip());
createEmptyMovieClip("movieClip", 10);
check("movieClip", movieClip);
check("timelineSprite", timelineSprite);
check("new MovieClipLoader()", new MovieClipLoader());
check("new NetConnection()", new NetConnection());
check("new NetStream()", new NetStream());
check("new NetStream(new NetConnection())", new NetStream(new NetConnection()));
check("new PrintJob()", new PrintJob());
check("Selection", Selection);
check("new SharedObject() /* not a class */", new SharedObject());
check("new Sound()", new Sound());
check("new Sound(2)", new Sound(2));
check("new Sound(_root)", new Sound(_root));
check("Stage", Stage);
check("System", System);
check("System.capabilities", System.capabilities);
check("new System.Product()", new System.Product());
check("System.security", System.security);
check("System.IME", System.IME);
createTextField("textField", 11, 0, 0, 10, 10);
check("new TextField()", new TextField());
check("textField", textField);
check("timelineEditText", timelineEditText);
check("new TextField.StyleSheet()", new TextField.StyleSheet());
check("new TextFormat()", new TextFormat());
check("new TextSnapshot()", new TextSnapshot());
check("new TextSnapshot(3)", new TextSnapshot(3));
check("new TextSnapshot(timelineButton)", new TextSnapshot(timelineButton));
check("new TextSnapshot(_root)", new TextSnapshot(_root));
check("new TextSnapshot(_root, 4)", new TextSnapshot(_root, 4));
check("new TextSnapshot(_root, _root)", new TextSnapshot(_root, _root));
check("new Video()", new Video());
check("timelineVideo", timelineVideo);
check("new XML()", new XML());
check("new XML(\"\")", new XML(""));
check("new XML(\"<a></a>\")", new XML("<a></a>"));
check("new XMLNode()", new XMLNode());
check("new XMLNode(1)", new XMLNode());
check("new XMLNode(1, \"name\")", new XMLNode(1, "name"));
check("new XMLNode(1, \"name\", 1)", new XMLNode(1, "name", 1));
check("new XMLNode(1, \"name\", true)", new XMLNode(1, "name", true));
check("new XMLNode(1, \"name\", 1, 2)", new XMLNode(1, "name", 1, 2));
check("new XMLSocket()", new XMLSocket());

check("flash", flash);

check("flash.automation", flash.automation);
check("new flash.automation.ActionGenerator() /* not a class */", new flash.automation.ActionGenerator());
check("new flash.automation.Configuration() /* not a class */", new flash.automation.Configuration());
check("new flash.automation.StageCapture()", new flash.automation.StageCapture());

check("flash.display", flash.display);
check("new flash.display.BitmapData()", new flash.display.BitmapData());
check("new flash.display.BitmapData(1,1)", new flash.display.BitmapData(1,1));

check("flash.external", flash.external);
check("new flash.external.ExternalInterface() /* not a class */", new flash.external.ExternalInterface());

check("flash.filters", flash.filters);
check("new flash.filters.BevelFilter()", new flash.filters.BevelFilter());
check("new flash.filters.BitmapFilter()", new flash.filters.BitmapFilter());
check("new flash.filters.BlurFilter()", new flash.filters.BlurFilter());
check("new flash.filters.ColorMatrixFilter()", new flash.filters.ColorMatrixFilter());
check("new flash.filters.ColorMatrixFilter([...])", new flash.filters.ColorMatrixFilter([1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0]));
check("new flash.filters.ConvolutionFilter()", new flash.filters.ConvolutionFilter());
check("new flash.filters.ConvolutionFilter([...])", new flash.filters.ConvolutionFilter(3, 3, [1, 1, 1, 1, 1, 1, 1, 1, 1]));
check("new flash.filters.DisplacementMapFilter()", new flash.filters.DisplacementMapFilter());
check("new flash.filters.DisplacementMapFilter([...])", new flash.filters.DisplacementMapFilter(new flash.display.BitmapData(1,1), new flash.geom.Point(-30, -30), 1, 1, 10, 10));
check("new flash.filters.DropShadowFilter()", new flash.filters.DropShadowFilter());
check("new flash.filters.GlowFilter()", new flash.filters.GlowFilter());
check("new flash.filters.GradientBevelFilter()", new flash.filters.GradientBevelFilter());
check("new flash.filters.GradientGlowFilter()", new flash.filters.GradientGlowFilter());

check("flash.geom", flash.geom);
check("new flash.geom.ColorTransform()", new flash.geom.ColorTransform());
check("new flash.geom.Matrix()", new flash.geom.Matrix());
check("new flash.geom.Point()", new flash.geom.Point());
check("new flash.geom.Rectangle()", new flash.geom.Rectangle());
check("new flash.geom.Rectangle(1,2,3,4)", new flash.geom.Rectangle(1,2,3,4));
check("new flash.geom.Transform()", new flash.geom.Transform());
check("new flash.geom.Transform(_root)", new flash.geom.Transform(_root));

check("flash.text", flash.text);
check("new flash.text.TextRenderer()", new flash.text.TextRenderer());

check("flash.net", flash.net);
check("new flash.net.FileReference()", new flash.net.FileReference());
check("new flash.net.FileReferenceList()", new flash.net.FileReferenceList());

fscommand("quit");
