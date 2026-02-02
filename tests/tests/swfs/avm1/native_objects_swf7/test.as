function getNativeStatus(obj) {
    if (typeof obj !== "object") {
        return "non-object: " + typeof obj;
    }

    // A native object cannot be initialized twice, so try to initialize
    // Date, and check if it worked.
    obj.__initializeNative = function() {
        this.__proto__ = {
            __proto__: Date.prototype,
            __constructor__: Date
        };
        if ((typeof this.getDate()) !== "undefined") {
            trace("ERROR");
        }
        super();
    };
    obj.__initializeNative();
    if ((typeof obj.getDate()) !== "undefined") {
        return "non-native";
    } else {
        return "native";
    }
}

function check(name, value) {
    trace(name + ": " + getNativeStatus(value));
}

check("{}", {});
check("[]", []);
check("\"\"", "");
check("5", 5);
check("true", true);
check("null", null);
check("undefined", undefined);

check("new Array()", new Array());
check("new Object()", new Object());
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
check("SharedObject", SharedObject);
check("new Sound()", new Sound());
check("new Sound(2)", new Sound(2));
check("new Sound(_root)", new Sound(_root));
check("Stage", Stage);
check("System", System);
createTextField("textField", 11, 0, 0, 10, 10);
check("new TextField()", new TextField());
check("textField", textField);
check("timelineEditText", timelineEditText);
check("new TextFormat()", new TextFormat());
check("new TextSnapshot()", new TextSnapshot());
check("new TextSnapshot(_root)", new TextSnapshot(_root));
check("new Video()", new Video());
check("timelineVideo", timelineVideo);
check("new XML()", new XML());
check("new XML(\"\")", new XML(""));
check("new XML(\"<a></a>\")", new XML("<a></a>"));
check("new XMLNode()", new XMLNode());
check("new XMLNode(1)", new XMLNode());
check("new XMLNode(1, \"name\")", new XMLNode(1, "name"));
check("new XMLSocket()", new XMLSocket());

check("new flash.display.BitmapData()", new flash.display.BitmapData());
check("new flash.display.BitmapData(1,1)", new flash.display.BitmapData(1,1));

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

check("new flash.geom.ColorTransform()", new flash.geom.ColorTransform());
check("new flash.geom.Matrix()", new flash.geom.Matrix());
check("new flash.geom.Point()", new flash.geom.Point());
check("new flash.geom.Rectangle()", new flash.geom.Rectangle());
check("new flash.geom.Rectangle(1,2,3,4)", new flash.geom.Rectangle(1,2,3,4));
check("new flash.geom.Transform()", new flash.geom.Transform());
check("new flash.geom.Transform(_root)", new flash.geom.Transform(_root));

check("new flash.text.TextRenderer()", new flash.text.TextRenderer());
