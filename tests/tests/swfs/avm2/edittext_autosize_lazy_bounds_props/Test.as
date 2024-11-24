package {
import flash.display.Sprite;
import flash.display.DisplayObject;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.text.StyleSheet;
import flash.events.Event;
import flash.events.KeyboardEvent;
import flash.accessibility.AccessibilityImplementation;
import flash.accessibility.AccessibilityProperties;
import flash.filters.DropShadowFilter;
import flash.geom.Rectangle;
import flash.geom.Matrix;
import flash.geom.Matrix3D;
import flash.geom.PerspectiveProjection;
import flash.geom.ColorTransform;
import flash.geom.Vector3D;
import flash.geom.Point;
import flash.ui.ContextMenu;

/**
 * When does Flash Player lazily set autosize bounds in a text field?
 *
 * Basically, we have to find two actions (A, B), such that when performed
 * together without updating bounds in between, will produce a different state (S)
 * compared to when the bounds in between are updated.
 * Having those, we can test whether another action (Q) updates bounds
 * by executing A,Q,B and checking the state S.
 *
 * In our case:
 *   A: autoSize = "center"
 *   B: wordWrap = true
 *   S: x,y,width,height
 *
 * In case wordWrap is enabled, it won't update text field's width, so when
 * wordWrap is enabled before bounds update, autoSize will not update the width,
 * but when it's enabled after bounds update, the width will already have been updated.
 */
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    private var desc: String = null;

    public function Test() {
        stage.scaleMode = "noScale";

        testTextFieldProperties();
        testInheritedProperties();

        testTextFieldMethods();
        testInheritedMethods();

        trace("Tests finished");
    }

    private function testTextFieldProperties() {
        testBoundsUpdate("get alwaysShowSelection", function(text:TextField, cb:Function) {
            dump(text.alwaysShowSelection);
            cb();
        });
        testBoundsUpdate("set alwaysShowSelection", function(text:TextField, cb:Function) {
            text.alwaysShowSelection = true;
            cb();
        });

        testBoundsUpdate("get antiAliasType", function(text:TextField, cb:Function) {
            dump(text.antiAliasType);
            cb();
        });
        testBoundsUpdate("set antiAliasType", function(text:TextField, cb:Function) {
            text.antiAliasType = "advanced";
            cb();
        });

        testBoundsUpdate("get autoSize", function(text:TextField, cb:Function) {
            dump(text.autoSize);
            cb();
        });
        testBoundsUpdate("set autoSize", function(text:TextField, cb:Function) {
            text.autoSize = "right";
            cb();
        });

        testBoundsUpdate("get background", function(text:TextField, cb:Function) {
            dump(text.background);
            cb();
        });
        testBoundsUpdate("set background", function(text:TextField, cb:Function) {
            text.background = true;
            cb();
        });

        testBoundsUpdate("get backgroundColor", function(text:TextField, cb:Function) {
            dump(text.backgroundColor);
            cb();
        });
        testBoundsUpdate("set backgroundColor", function(text:TextField, cb:Function) {
            text.backgroundColor = 420;
            cb();
        });

        testBoundsUpdate("get border", function(text:TextField, cb:Function) {
            dump(text.border);
            cb();
        });
        testBoundsUpdate("set border", function(text:TextField, cb:Function) {
            text.border = true;
            cb();
        });

        testBoundsUpdate("get borderColor", function(text:TextField, cb:Function) {
            dump(text.borderColor);
            cb();
        });
        testBoundsUpdate("set borderColor", function(text:TextField, cb:Function) {
            text.borderColor = 420;
            cb();
        });

        testBoundsUpdate("get bottomScrollV", function(text:TextField, cb:Function) {
            dump(text.bottomScrollV);
            cb();
        });

        testBoundsUpdate("get caretIndex", function(text:TextField, cb:Function) {
            dump(text.caretIndex);
            cb();
        });

        testBoundsUpdate("get condenseWhite", function(text:TextField, cb:Function) {
            dump(text.condenseWhite);
            cb();
        });
        testBoundsUpdate("set condenseWhite", function(text:TextField, cb:Function) {
            text.condenseWhite = true;
            cb();
        });

        testBoundsUpdate("get defaultTextFormat", function(text:TextField, cb:Function) {
            dump(text.defaultTextFormat);
            cb();
        });
        testBoundsUpdate("set defaultTextFormat", function(text:TextField, cb:Function) {
            var tf = new TextFormat();
            tf.size = 42;
            text.defaultTextFormat = tf;
            cb();
        });

        testBoundsUpdate("get displayAsPassword", function(text:TextField, cb:Function) {
            dump(text.displayAsPassword);
            cb();
        });
        testBoundsUpdate("set displayAsPassword", function(text:TextField, cb:Function) {
            text.displayAsPassword = true;
            cb();
        });

        testBoundsUpdate("get embedFonts", function(text:TextField, cb:Function) {
            dump(text.embedFonts);
            cb();
        });
        testBoundsUpdate("set embedFonts", function(text:TextField, cb:Function) {
            text.embedFonts = true;
            cb();
        });

        testBoundsUpdate("get gridFitType", function(text:TextField, cb:Function) {
            dump(text.gridFitType);
            cb();
        });
        testBoundsUpdate("set gridFitType", function(text:TextField, cb:Function) {
            text.gridFitType = "subpixel";
            cb();
        });

        testBoundsUpdate("get htmlText", function(text:TextField, cb:Function) {
            dump(text.htmlText);
            cb();
        });
        testBoundsUpdate("set htmlText", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.htmlText = "x";
            cb();
        });

        testBoundsUpdate("get length", function(text:TextField, cb:Function) {
            dump(text.length);
            cb();
        });

        testBoundsUpdate("get maxChars", function(text:TextField, cb:Function) {
            dump(text.maxChars);
            cb();
        });
        testBoundsUpdate("set maxChars", function(text:TextField, cb:Function) {
            text.maxChars = 5;
            cb();
        });

        testBoundsUpdate("get maxScrollH", function(text:TextField, cb:Function) {
            dump(text.maxScrollH);
            cb();
        });

        testBoundsUpdate("get maxScrollV", function(text:TextField, cb:Function) {
            dump(text.maxScrollV);
            cb();
        });

        testBoundsUpdate("get mouseWheelEnabled", function(text:TextField, cb:Function) {
            dump(text.mouseWheelEnabled || true);
            cb();
        });
        testBoundsUpdate("set mouseWheelEnabled", function(text:TextField, cb:Function) {
            text.mouseWheelEnabled = false;
            cb();
        });

        testBoundsUpdate("get multiline", function(text:TextField, cb:Function) {
            dump(text.multiline);
            cb();
        });
        testBoundsUpdate("set multiline", function(text:TextField, cb:Function) {
            text.multiline = true;
            cb();
        });

        testBoundsUpdate("get numLines", function(text:TextField, cb:Function) {
            dump(text.numLines);
            cb();
        });

        testBoundsUpdate("get restrict", function(text:TextField, cb:Function) {
            dump(text.restrict);
            cb();
        });
        testBoundsUpdate("set restrict", function(text:TextField, cb:Function) {
            text.restrict = "x";
            cb();
        });

        testBoundsUpdate("get scrollH", function(text:TextField, cb:Function) {
            dump(text.scrollH);
            cb();
        });
        testBoundsUpdate("set scrollH", function(text:TextField, cb:Function) {
            text.scrollH = 1;
            cb();
        });

        testBoundsUpdate("get scrollV", function(text:TextField, cb:Function) {
            dump(text.scrollV);
            cb();
        });
        testBoundsUpdate("set scrollV", function(text:TextField, cb:Function) {
            text.scrollV = 1;
            cb();
        });

        testBoundsUpdate("get selectable", function(text:TextField, cb:Function) {
            dump(text.selectable);
            cb();
        });
        testBoundsUpdate("set selectable", function(text:TextField, cb:Function) {
            text.selectable = false;
            cb();
        });

        testBoundsUpdate("get selectionBeginIndex", function(text:TextField, cb:Function) {
            dump(text.selectionBeginIndex);
            cb();
        });

        testBoundsUpdate("get selectionEndIndex", function(text:TextField, cb:Function) {
            dump(text.selectionEndIndex);
            cb();
        });

        testBoundsUpdate("get sharpness", function(text:TextField, cb:Function) {
            dump(text.sharpness);
            cb();
        });
        testBoundsUpdate("set sharpness", function(text:TextField, cb:Function) {
            text.sharpness = 1;
            cb();
        });

        testBoundsUpdate("get styleSheet", function(text:TextField, cb:Function) {
            dump(text.styleSheet);
            cb();
        });
        testBoundsUpdate("set styleSheet", function(text:TextField, cb:Function) {
            text.styleSheet = new StyleSheet();
            cb();
        });

        testBoundsUpdate("get text", function(text:TextField, cb:Function) {
            dump(text.text);
            cb();
        });
        testBoundsUpdate("set text", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            cb();
        });

        testBoundsUpdate("get textColor", function(text:TextField, cb:Function) {
            dump(text.textColor);
            cb();
        });
        testBoundsUpdate("set textColor", function(text:TextField, cb:Function) {
            text.textColor = 420;
            cb();
        });

        testBoundsUpdate("get textHeight", function(text:TextField, cb:Function) {
            dump(text.textHeight);
            cb();
        });

        testBoundsUpdate("get textInteractionMode", function(text:TextField, cb:Function) {
            dump(text.textInteractionMode);
            cb();
        });

        testBoundsUpdate("get textWidth", function(text:TextField, cb:Function) {
            dump(text.textWidth);
            cb();
        });

        testBoundsUpdate("get thickness", function(text:TextField, cb:Function) {
            dump(text.thickness);
            cb();
        });
        testBoundsUpdate("set thickness", function(text:TextField, cb:Function) {
            text.thickness = 1;
            cb();
        });

        testBoundsUpdate("get type", function(text:TextField, cb:Function) {
            dump(text.type);
            cb();
        });
        testBoundsUpdate("set type", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "a";
            text.type = "input";
            cb();
        });

        testBoundsUpdate("get useRichTextClipboard", function(text:TextField, cb:Function) {
            dump(text.useRichTextClipboard);
            cb();
        });
        testBoundsUpdate("set useRichTextClipboard", function(text:TextField, cb:Function) {
            text.useRichTextClipboard = true;
            cb();
        });

        testBoundsUpdate("get wordWrap", function(text:TextField, cb:Function) {
            dump(text.wordWrap);
            cb();
        });
        testBoundsUpdate("set wordWrap", function(text:TextField, cb:Function) {
            text.wordWrap = true;
            cb();
        });
    }

    private function testInheritedProperties() {
        testBoundsUpdate("get accessibilityImplementation", function(text:TextField, cb:Function) {
            dump(text.accessibilityImplementation);
            cb();
        });
        testBoundsUpdate("set accessibilityImplementation", function(text:TextField, cb:Function) {
            text.accessibilityImplementation = new AccessibilityImplementation();
            cb();
        });

        testBoundsUpdate("get accessibilityProperties", function(text:TextField, cb:Function) {
            dump(text.accessibilityProperties);
            cb();
        });
        testBoundsUpdate("set accessibilityProperties", function(text:TextField, cb:Function) {
            text.accessibilityProperties = new AccessibilityProperties();
            cb();
        });

        testBoundsUpdate("get alpha", function(text:TextField, cb:Function) {
            dump(text.alpha);
            cb();
        });
        testBoundsUpdate("set alpha", function(text:TextField, cb:Function) {
            text.alpha = 0.5;
            cb();
        });

        testBoundsUpdate("get blendMode", function(text:TextField, cb:Function) {
            dump(text.blendMode);
            cb();
        });
        testBoundsUpdate("set blendMode", function(text:TextField, cb:Function) {
            text.blendMode = "difference";
            cb();
        });

        testBoundsUpdate("get cacheAsBitmap", function(text:TextField, cb:Function) {
            dump(text.cacheAsBitmap);
            cb();
        });
        testBoundsUpdate("set cacheAsBitmap", function(text:TextField, cb:Function) {
            text.cacheAsBitmap = true;
            cb();
        });

        testBoundsUpdate("get contextMenu", function(text:TextField, cb:Function) {
            dump(text.contextMenu);
            cb();
        });
        testBoundsUpdate("set contextMenu", function(text:TextField, cb:Function) {
            text.contextMenu = new ContextMenu();;
            cb();
        });

        testBoundsUpdate("get doubleClickEnabled", function(text:TextField, cb:Function) {
            dump(text.doubleClickEnabled);
            cb();
        });
        testBoundsUpdate("set doubleClickEnabled", function(text:TextField, cb:Function) {
            text.doubleClickEnabled = true;
            cb();
        });

        testBoundsUpdate("get filters", function(text:TextField, cb:Function) {
            dump(text.filters);
            cb();
        });
        testBoundsUpdate("set filters", function(text:TextField, cb:Function) {
            text.filters = [new DropShadowFilter()];
            cb();
        });

        testBoundsUpdate("get focusRect", function(text:TextField, cb:Function) {
            dump(text.focusRect);
            cb();
        });
        testBoundsUpdate("set focusRect", function(text:TextField, cb:Function) {
            text.focusRect = true;
            cb();
        });

        testBoundsUpdate("get height", function(text:TextField, cb:Function) {
            dump(text.height);
            cb();
        });
        testBoundsUpdate("set height", function(text:TextField, cb:Function) {
            text.height = 100;
            cb();
        });
        testBoundsUpdate("set height 2", function(text:TextField, cb:Function) {
            text.height = 100;
            dump(text.height);
            text.height = 100;
            cb();
        });

        testBoundsUpdate("get loaderInfo", function(text:TextField, cb:Function) {
            dump(text.loaderInfo);
            cb();
        });

        testBoundsUpdate("get mask", function(text:TextField, cb:Function) {
            dump(text.mask);
            cb();
        });
        testBoundsUpdate("set mask", function(text:TextField, cb:Function) {
            text.mask = new Sprite();
            cb();
        });
        testBoundsUpdate("set mask in tree", function(text:TextField, cb:Function) {
            addChild(text);
            var mask = new Sprite();
            addChild(mask);
            text.mask = mask;
            cb();
        });

        testBoundsUpdate("get metaData", function(text:TextField, cb:Function) {
            dump(text.metaData);
            cb();
        });
        testBoundsUpdate("set metaData", function(text:TextField, cb:Function) {
            var metaData:Object = {};
            metaData.x = "y";
            text.metaData = metaData;
            cb();
        });

        testBoundsUpdate("get mouseEnabled", function(text:TextField, cb:Function) {
            dump(text.mouseEnabled);
            cb();
        });
        testBoundsUpdate("set mouseEnabled", function(text:TextField, cb:Function) {
            text.mouseEnabled = false;
            cb();
        });

        testBoundsUpdate("get mouseX", function(text:TextField, cb:Function) {
            dump(Math.abs(Math.floor(text.mouseX / 20000)));
            cb();
        });

        testBoundsUpdate("get mouseY", function(text:TextField, cb:Function) {
            dump(Math.abs(Math.floor(text.mouseY / 20000)));
            cb();
        });

        testBoundsUpdate("get name", function(text:TextField, cb:Function) {
            dump(text.name);
            cb();
        });
        testBoundsUpdate("set name", function(text:TextField, cb:Function) {
            text.name = "x";
            cb();
        });

        testBoundsUpdate("get needsSoftKeyboard", function(text:TextField, cb:Function) {
            dump(text.needsSoftKeyboard);
            cb();
        });
        testBoundsUpdate("set needsSoftKeyboard", function(text:TextField, cb:Function) {
            text.needsSoftKeyboard = true;
            cb();
        });

        testBoundsUpdate("get opaqueBackground", function(text:TextField, cb:Function) {
            dump(text.opaqueBackground);
            cb();
        });
        testBoundsUpdate("set opaqueBackground", function(text:TextField, cb:Function) {
            text.opaqueBackground = 420;
            cb();
        });

        testBoundsUpdate("get parent", function(text:TextField, cb:Function) {
            dump(text.parent);
            cb();
        });

        testBoundsUpdate("get root", function(text:TextField, cb:Function) {
            dump(text.root);
            cb();
        });

        testBoundsUpdate("get rotation", function(text:TextField, cb:Function) {
            dump(text.rotation);
            cb();
        });
        testBoundsUpdate("set rotation", function(text:TextField, cb:Function) {
            text.rotation = 1;
            cb();
        });

        testBoundsUpdate("get rotationX", function(text:TextField, cb:Function) {
            dump(text.rotationX);
            cb();
        });
        testBoundsUpdate("set rotationX", function(text:TextField, cb:Function) {
            text.rotationX = 1;
            cb();
        });

        testBoundsUpdate("get rotationY", function(text:TextField, cb:Function) {
            dump(text.rotationY);
            cb();
        });
        testBoundsUpdate("set rotationY", function(text:TextField, cb:Function) {
            text.rotationY = 1;
            cb();
        });

        testBoundsUpdate("get rotationZ", function(text:TextField, cb:Function) {
            dump(text.rotationZ);
            cb();
        });
        testBoundsUpdate("set rotationZ", function(text:TextField, cb:Function) {
            text.rotationZ = 360;
            cb();
        });

        testBoundsUpdate("get scale9Grid", function(text:TextField, cb:Function) {
            dump(text.scale9Grid);
            cb();
        });

        testBoundsUpdate("get scaleX", function(text:TextField, cb:Function) {
            dump(text.scaleX);
            cb();
        });
        testBoundsUpdate("set scaleX", function(text:TextField, cb:Function) {
            text.scaleX = 2;
            cb();
        });

        testBoundsUpdate("get scaleY", function(text:TextField, cb:Function) {
            dump(text.scaleY);
            cb();
        });
        testBoundsUpdate("set scaleY", function(text:TextField, cb:Function) {
            text.scaleY = 2;
            cb();
        });

        testBoundsUpdate("get scaleZ", function(text:TextField, cb:Function) {
            dump(text.scaleZ);
            cb();
        });
        testBoundsUpdate("set scaleZ", function(text:TextField, cb:Function) {
            text.scaleZ = 2;
            cb();
        });

        testBoundsUpdate("get scrollRect", function(text:TextField, cb:Function) {
            dump(text.scrollRect);
            cb();
        });
        testBoundsUpdate("set scrollRect", function(text:TextField, cb:Function) {
            text.scrollRect = new Rectangle(1, 2, 3, 4);
            cb();
        });

        testBoundsUpdate("get softKeyboardInputAreaOfInterest", function(text:TextField, cb:Function) {
            dump(text.softKeyboardInputAreaOfInterest);
            cb();
        });
        testBoundsUpdate("set softKeyboardInputAreaOfInterest", function(text:TextField, cb:Function) {
            text.softKeyboardInputAreaOfInterest = new Rectangle(1, 2, 3, 4);
            cb();
        });

        testBoundsUpdate("get stage", function(text:TextField, cb:Function) {
            dump(text.stage);
            cb();
        });

        testBoundsUpdate("get tabEnabled", function(text:TextField, cb:Function) {
            dump(text.tabEnabled);
            cb();
        });
        testBoundsUpdate("set tabEnabled", function(text:TextField, cb:Function) {
            text.tabEnabled = false;
            cb();
        });

        testBoundsUpdate("get tabIndex", function(text:TextField, cb:Function) {
            dump(text.tabIndex);
            cb();
        });
        testBoundsUpdate("set tabIndex", function(text:TextField, cb:Function) {
            text.tabIndex = 5;
            cb();
        });

        testBoundsUpdate("get transform.colorTransform", function(text:TextField, cb:Function) {
            dump(text.transform.colorTransform);
            cb();
        });
        testBoundsUpdate("set transform.colorTransform", function(text:TextField, cb:Function) {
            text.transform.colorTransform = new ColorTransform(2);
            cb();
        });

        testBoundsUpdate("get transform.concatenatedColorTransform", function(text:TextField, cb:Function) {
            dump(text.transform.concatenatedColorTransform);
            cb();
        });

        testBoundsUpdate("get transform.concatenatedMatrix", function(text:TextField, cb:Function) {
            dump(text.transform.concatenatedMatrix);
            cb();
        });

        testBoundsUpdate("get transform.matrix", function(text:TextField, cb:Function) {
            dump(text.transform.matrix);
            cb();
        });
        testBoundsUpdate("set transform.matrix", function(text:TextField, cb:Function) {
            var m:Matrix = text.transform.matrix;
            m.tx += 1;
            text.transform.matrix = m;
            cb();
        });

        testBoundsUpdate("get transform.matrix3D", function(text:TextField, cb:Function) {
            dump(text.transform.matrix3D);
            cb();
        });
        testBoundsUpdate("set transform.matrix3D", function(text:TextField, cb:Function) {
            var m:Matrix3D = new Matrix3D();
            m.position = new Vector3D(0, 0, 3);
            text.transform.matrix3D = m;
            cb();
        });

        testBoundsUpdate("get transform.perspectiveProjection", function(text:TextField, cb:Function) {
            dump(text.transform.perspectiveProjection);
            cb();
        });
        testBoundsUpdate("set transform.perspectiveProjection", function(text:TextField, cb:Function) {
            var pp:PerspectiveProjection = new PerspectiveProjection();
            pp.fieldOfView = 42;
            text.transform.perspectiveProjection = pp;
            cb();
        });

        testBoundsUpdate("get transform.pixelBounds", function(text:TextField, cb:Function) {
            dump(text.transform.pixelBounds.x);
            cb();
        });

        testBoundsUpdate("get visible", function(text:TextField, cb:Function) {
            dump(text.visible);
            cb();
        });
        testBoundsUpdate("set visible", function(text:TextField, cb:Function) {
            text.visible = false;
            cb();
        });

        testBoundsUpdate("get width", function(text:TextField, cb:Function) {
            dump(text.width);
            cb();
        });
        testBoundsUpdate("set width", function(text:TextField, cb:Function) {
            text.width = 200;
            cb();
        });
        testBoundsUpdate("set width 2", function(text:TextField, cb:Function) {
            text.width = 200;
            dump(text.width);
            text.width = 200;
            cb();
        });

        testBoundsUpdate("get x", function(text:TextField, cb:Function) {
            dump(text.x);
            cb();
        });
        testBoundsUpdate("set x", function(text:TextField, cb:Function) {
            text.x = 1;
            cb();
        });

        testBoundsUpdate("get y", function(text:TextField, cb:Function) {
            dump(text.y);
            cb();
        });
        testBoundsUpdate("set y", function(text:TextField, cb:Function) {
            text.y = 1;
            cb();
        });

        testBoundsUpdate("get z", function(text:TextField, cb:Function) {
            dump(text.z);
            cb();
        });
        testBoundsUpdate("set z", function(text:TextField, cb:Function) {
            text.z = 1;
            cb();
        });
    }

    private function testTextFieldMethods() {
        testBoundsUpdate("appendText", function(text:TextField, cb:Function) {
            text.appendText("");
            cb();
        });
        testBoundsUpdate("appendText 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.appendText("x");
            cb();
        });

        testBoundsUpdate("getCharBoundaries", function(text:TextField, cb:Function) {
            text.getCharBoundaries(0);
            cb();
        });
        testBoundsUpdate("getCharBoundaries 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.getCharBoundaries(0);
            cb();
        });

        testBoundsUpdate("getCharIndexAtPoint", function(text:TextField, cb:Function) {
            text.getCharIndexAtPoint(2, 2);
            cb();
        });
        testBoundsUpdate("getCharIndexAtPoint 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.getCharIndexAtPoint(2, 2);
            cb();
        });

        testBoundsUpdate("getFirstCharInParagraph", function(text:TextField, cb:Function) {
            text.getFirstCharInParagraph(0);
            cb();
        });
        testBoundsUpdate("getFirstCharInParagraph 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.getFirstCharInParagraph(0);
            cb();
        });

        testBoundsUpdate("getImageReference", function(text:TextField, cb:Function) {
            text.getImageReference("x");
            cb();
        });

        testBoundsUpdate("getLineIndexAtPoint", function(text:TextField, cb:Function) {
            text.getLineIndexAtPoint(2, 2);
            cb();
        });
        testBoundsUpdate("getLineIndexAtPoint 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.getLineIndexAtPoint(2, 2);
            cb();
        });

        testBoundsUpdate("getLineIndexOfChar", function(text:TextField, cb:Function) {
            text.getLineIndexOfChar(0);
            cb();
        });
        testBoundsUpdate("getLineIndexOfChar 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.getLineIndexOfChar(0);
            cb();
        });

        testBoundsUpdate("getLineLength", function(text:TextField, cb:Function) {
            text.getLineLength(0);
            cb();
        });
        testBoundsUpdate("getLineLength 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.getLineLength(0);
            cb();
        });

        testBoundsUpdate("getLineMetrics", function(text:TextField, cb:Function) {
            text.getLineMetrics(0);
            cb();
        });
        testBoundsUpdate("getLineMetrics 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.getLineMetrics(0);
            cb();
        });

        testBoundsUpdate("getLineOffset", function(text:TextField, cb:Function) {
            text.getLineOffset(0);
            cb();
        });
        testBoundsUpdate("getLineOffset 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.getLineOffset(0);
            cb();
        });

        testBoundsUpdate("getLineText", function(text:TextField, cb:Function) {
            text.getLineText(0);
            cb();
        });
        testBoundsUpdate("getLineText 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.getLineText(0);
            cb();
        });

        testBoundsUpdate("getParagraphLength", function(text:TextField, cb:Function) {
            text.getParagraphLength(0);
            cb();
        });
        testBoundsUpdate("getParagraphLength 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.getParagraphLength(0);
            cb();
        });

        testBoundsUpdate("getTextFormat", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.getTextFormat(0, 1);
            cb();
        });

        testBoundsUpdate("replaceSelectedText", function(text:TextField, cb:Function) {
            text.replaceSelectedText("");
            cb();
        });

        testBoundsUpdate("replaceText", function(text:TextField, cb:Function) {
            text.replaceText(0, 0, "");
            cb();
        });
        testBoundsUpdate("replaceText 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.replaceText(0, 1, "xx");
            cb();
        });

        testBoundsUpdate("setSelection", function(text:TextField, cb:Function) {
            text.setSelection(0, 0);
            cb();
        });
        testBoundsUpdate("setSelection 2", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.setSelection(0, 1);
            cb();
        });

        testBoundsUpdate("setTextFormat", function(text:TextField, cb:Function) {
            setUpTextFormat(text);
            text.text = "x";
            text.setTextFormat(text.defaultTextFormat, 0, 1);
            cb();
        });
    }

    private function testInheritedMethods() {
        testBoundsUpdate("addEventListener", function(text:TextField, cb:Function) {
            text.addEventListener("change", function(evt:*):void {});
            cb();
        });
        testBoundsUpdate("dispatchEvent", function(text:TextField, cb:Function) {
            text.dispatchEvent(new Event("change"));
            cb();
        });
        testBoundsUpdate("getBounds", function(text:TextField, cb:Function) {
            text.getBounds(stage);
            cb();
        });
        testBoundsUpdate("getRect", function(text:TextField, cb:Function) {
            text.getRect(stage);
            cb();
        });
        testBoundsUpdate("globalToLocal", function(text:TextField, cb:Function) {
            text.globalToLocal(new Point(0, 0));
            cb();
        });
        testBoundsUpdate("hasEventListener", function(text:TextField, cb:Function) {
            text.hasEventListener("change");
            cb();
        });
        testBoundsUpdate("hasOwnProperty", function(text:TextField, cb:Function) {
            text.hasOwnProperty("type");
            cb();
        });
        testBoundsUpdate("hitTestObject", function(text:TextField, cb:Function) {
            text.hitTestObject(stage);
            cb();
        });
        testBoundsUpdate("hitTestPoint", function(text:TextField, cb:Function) {
            text.hitTestPoint(2, 2);
            cb();
        });
        testBoundsUpdate("isPrototypeOf", function(text:TextField, cb:Function) {
            text.isPrototypeOf(TestFont);
            cb();
        });
        testBoundsUpdate("local3DToGlobal", function(text:TextField, cb:Function) {
            text.local3DToGlobal(new Vector3D(1, 2, 3));
            cb();
        });
        testBoundsUpdate("localToGlobal", function(text:TextField, cb:Function) {
            text.localToGlobal(new Point(0, 0));
            cb();
        });
        testBoundsUpdate("propertyIsEnumerable", function(text:TextField, cb:Function) {
            text.propertyIsEnumerable("type");
            cb();
        });
        testBoundsUpdate("removeEventListener", function(text:TextField, cb:Function) {
            text.removeEventListener("change", function(evt:*):void {});
            cb();
        });
        testBoundsUpdate("requestSoftKeyboard", function(text:TextField, cb:Function) {
            text.requestSoftKeyboard();
            cb();
        });
        testBoundsUpdate("toLocaleString", function(text:Object, cb:Function) {
            text.toLocaleString();
            cb();
        });
        testBoundsUpdate("toString", function(text:TextField, cb:Function) {
            text.toString();
            cb();
        });
        testBoundsUpdate("valueOf", function(text:Object, cb:Function) {
            text.valueOf();
            cb();
        });
        testBoundsUpdate("willTrigger", function(text:TextField, cb:Function) {
            text.willTrigger("change");
            cb();
        });
    }

    private function testBoundsUpdate(desc: String, fun: Function):void {
        this.desc = desc;
        var text = new TextField();
        text.width = 100;
        text.height = 20;

        text.autoSize = "center";
        fun(text, function():void {
            text.wordWrap = true;
            trace("Testing: " + desc);
            trace("  " + text.x + "," + text.y + "," + text.width + "," + text.height);
        });
    }

    /// This method makes sure we're deterministic when doing stuff with text.
    /// Based on the test results, both defaultTextFormat and embedFonts do
    /// not perform bounds update so we're safe for more advanced tests too.
    private function setUpTextFormat(text: TextField):void {
        var tf = new TextFormat();
        tf.size = 20;
        tf.font = "TestFont";
        text.defaultTextFormat = tf;
        text.embedFonts = true;
    }

    /// Just to make sure we use the value and it's really being read.
    private function dump(o: *):void {
        trace("// " + desc + ": " + o);
    }
}
}
