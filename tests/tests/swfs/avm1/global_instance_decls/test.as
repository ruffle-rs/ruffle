function contains(arr, val) {
    for (var i in arr) {
        if (arr[i] === val) {
            return true;
        }
    }
    return false;
}

function testInstance(className, instance) {
    if (instance === undefined) {
        var clazz = eval(className);
        instance = new clazz();
    }
    trace(className + ", instance=[" + instance + "], type=[" + (typeof(instance)) + "]");

    var enumerated = [];
    for (var key in instance) {
        enumerated.push(key);
    }

    ASSetPropFlags(instance, null, 0, 1);

    var allProps = [];
    var own = [];
    var originalValues = {};
    for (var key in instance) {
        allProps.push(key);
        originalValues["key_" + key] = instance[key];

        var isOwn = instance.hasOwnProperty(key);
        if (isOwn) {
            own.push(key);
        }
    }

    var modifiable = [];
    for (var key in instance) {
        instance[key] = "TEST_VALUE_RUFFLE";
        if (instance[key] === "TEST_VALUE_RUFFLE") {
            modifiable.push(key);
        }
    }

    var deletable = [];
    for (var i in allProps) {
        var key = allProps[i];
        delete instance[key];

        var deleted = true;
        for (var key2 in instance) {
            if (key2 === key) {
                deleted = false;
                break;
            }
        }
        if (deleted) {
            deletable.push(key);
        }
    }

    for (var i in allProps) {
        var key = allProps[i];
        var properties = "";

        if (contains(own, key)) {
            properties += ", own";
        }

        if (contains(enumerated, key)) {
            properties += ", DONT_ENUM";
        }

        if (!contains(modifiable, key)) {
            properties += ", READ_ONLY";
        }

        if (!contains(deletable, key)) {
            properties += ", DONT_DELETE";
        }

        var originalVal = originalValues["key_" + key];

        properties += ", type=[" + (typeof(originalVal)) + "]";

        trace("  " + key + properties);
    }
}

testInstance("_global.TextSnapshot");
testInstance("_global.PrintJob");
testInstance("_global.MovieClipLoader");
testInstance("_global.LocalConnection");
testInstance("_global.textRenderer");
testInstance("_global.flash.automation.Configuration");
testInstance("_global.flash.automation.ActionGenerator");
testInstance("_global.flash.automation.StageCapture");
testInstance("_global.flash.external.ExternalInterface");
testInstance("_global.flash.net.FileReferenceList");
testInstance("_global.flash.net.FileReference");
testInstance("_global.flash.geom.Transform");
testInstance("_global.flash.geom.ColorTransform");
testInstance("_global.flash.geom.Matrix");
testInstance("_global.flash.geom.Matrix", new flash.geom.Matrix(1));
testInstance("_global.flash.geom.Matrix", new flash.geom.Matrix(1, 2, 3, 4));
testInstance("_global.flash.geom.Point");
testInstance("_global.flash.geom.Point", new flash.geom.Point(2));
testInstance("_global.flash.geom.Point", new flash.geom.Point(2, 5));
testInstance("_global.flash.geom.Rectangle");
testInstance("_global.flash.geom.Rectangle", new flash.geom.Rectangle(1));
testInstance("_global.flash.geom.Rectangle", new flash.geom.Rectangle(1, 2));
testInstance("_global.flash.geom.Rectangle", new flash.geom.Rectangle(1, 2, 3));
testInstance("_global.flash.geom.Rectangle", new flash.geom.Rectangle(1, 2, 3, 4));
testInstance("_global.flash.filters.DisplacementMapFilter");
testInstance("_global.flash.filters.ColorMatrixFilter");
testInstance("_global.flash.filters.ConvolutionFilter");
testInstance("_global.flash.filters.GradientBevelFilter");
testInstance("_global.flash.filters.GradientGlowFilter");
testInstance("_global.flash.filters.BevelFilter");
testInstance("_global.flash.filters.GlowFilter");
testInstance("_global.flash.filters.BlurFilter");
testInstance("_global.flash.filters.DropShadowFilter");
testInstance("_global.flash.filters.BitmapFilter");
testInstance("_global.flash.display.BitmapData");
testInstance("_global.flash.display.BitmapData", new flash.display.BitmapData(100));
testInstance("_global.flash.display.BitmapData", new flash.display.BitmapData(100, 100));
testInstance("_global.flash.display.BitmapData", new flash.display.BitmapData("100", "100"));
testInstance("_global.flash.text.TextRenderer");
testInstance("_global.System.security.PolicyFileResolver");
testInstance("_global.System.security.__constructor__");
testInstance("_global.System.Product");
testInstance("_global.Video");
testInstance("_global.Stage");
testInstance("_global.TextFormat");
testInstance("_global.TextField");
testInstance("_global.Button");
testInstance("_global.Mouse");
testInstance("_global.Selection");
testInstance("_global.LoadVars");
testInstance("_global.XML");
testInstance("_global.XML", new XML("<element><name>test</name><value>420</value></element>"));
testInstance("_global.XMLNode");
testInstance("_global.XMLNode", new XML("<element><name>test</name><value>420</value></element>").firstChild);
testInstance("_global.Sound");
testInstance("_global.Math");
testInstance("_global.Array");
testInstance("_global.String");
testInstance("_global.Date", new Date(2020, 3, 4));
testInstance("_global.Boolean");
testInstance("_global.Number");
testInstance("_global.clearRequestHeaders");
testInstance("_global.addRequestHeader");
testInstance("_global.RemoteLSOUsage");
testInstance("_global.AssetCache");
testInstance("_global.AsSetupError");
testInstance("_global.Error");
testInstance("_global.ContextMenu");
testInstance("_global.ContextMenuItem");
testInstance("_global.SharedObject");
testInstance("_global.Microphone");
testInstance("_global.Camera");
testInstance("_global.NetStream");
testInstance("_global.NetConnection");
testInstance("_global.Color");
testInstance("_global.AsBroadcaster");
testInstance("_global.XMLSocket");
testInstance("_global.MovieClip");
testInstance("_global.Function");
testInstance("_global.Object");
