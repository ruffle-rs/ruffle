package {
import flash.display.*;
import flash.text.*;
import flash.net.*;
import flash.system.*;
import flash.events.*;

public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont1", embedAsCFF="false", unicodeRange="U+0061-U+0061")]
    private var TestFont1:Class;

    [Embed(source="TestFont.ttf", fontName="Test Font3", embedAsCFF="false", unicodeRange="U+0061-U+0061")]
    private var TestFont2:Class;

    [Embed(source="TestFont.ttf", fontName="TestFont2", embedAsCFF="false", unicodeRange="U+0061-U+0061")]
    private var TestFont3:Class;

    [Embed(source="TestFont.ttf", fontName="TestFont7", embedAsCFF="false", unicodeRange="U+0061-U+0061")]
    private var TestFont4:Class;

    [Embed(source="TestFont.ttf", fontName="TeStfONt6", embedAsCFF="false", unicodeRange="U+0061-U+0061")]
    private var TestFont5:Class;

    [Embed(source="TestFont.ttf", fontName="testfont5", embedAsCFF="false", unicodeRange="U+0061-U+0061")]
    private var TestFont6:Class;

    public function Test() {
        Font.registerFont(TestFont2);
        Font.registerFont(TestFont1);

        var fontLoader:Loader = new Loader();
        var context:LoaderContext = new LoaderContext();
        context.allowCodeImport = true;
        fontLoader.contentLoaderInfo.addEventListener(Event.COMPLETE, onFontLoaded);
        fontLoader.load(new URLRequest("font.swf"), context);
    }

    private function onFontLoaded(event:Event):void {
        var fontClass:Class = event.target.applicationDomain.getDefinition("FontSwf_TestFont4") as Class;
        Font.registerFont(fontClass);

        for each (var font: Font in Font.enumerateFonts()) {
            trace(font.fontName + ", " + font.fontStyle + ", " + font.fontType);
        }
    }
}

}
