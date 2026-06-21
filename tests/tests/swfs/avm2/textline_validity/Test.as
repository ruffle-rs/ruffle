package {
import flash.display.*;
import flash.text.engine.*;

public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="true", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    public function Test() {
        trace("TextLineValidity");
        trace(TextLineValidity.INVALID);
        trace(TextLineValidity.POSSIBLY_INVALID);
        trace(TextLineValidity.STATIC);
        trace(TextLineValidity.VALID);
        trace("");

        trace("Valid line");
        testValidity(function():TextLine { return getValidLine(); });
        trace("Invalid line");
        testValidity(function():TextLine { return getInvalidLine(); });
        trace("Static line");
        testValidity(function():TextLine { return getStaticLine(); });
        trace("User invalid line");
        testValidity(function():TextLine { return getUserInvalidLine(); });

        // possiblyInvalid appears to be a fully internal state with no way of acquiring it in code.
    }

    private function testValidity(factory:Function):void {
        trace("validity=" + factory().validity);
        trace("  -> invalid=" + testSettingValidity(factory(), "invalid"));
        trace("  -> Invalid=" + testSettingValidity(factory(), "Invalid"));
        trace("  -> INVALID=" + testSettingValidity(factory(), "INVALID"));
        trace("  -> possiblyInvalid=" + testSettingValidity(factory(), "possiblyInvalid"));
        trace("  -> possiblyinvalid=" + testSettingValidity(factory(), "possiblyinvalid"));
        trace("  -> POSSIBLY_INVALID=" + testSettingValidity(factory(), "POSSIBLY_INVALID"));
        trace("  -> static=" + testSettingValidity(factory(), "static"));
        trace("  -> STATIC=" + testSettingValidity(factory(), "STATIC"));
        trace("  -> valid=" + testSettingValidity(factory(), "valid"));
        trace("  -> Valid=" + testSettingValidity(factory(), "Valid"));
        trace("  -> unknown=" + testSettingValidity(factory(), "unknown"));
        trace("  -> null=" + testSettingValidity(factory(), null));
    }

    private function testSettingValidity(textLine:TextLine, validity:String):String {
        try {
            textLine.validity = validity;
            return textLine.validity;
        } catch (e:Error) {
            return "Error: " + e.getStackTrace();
        }
        return "???";
    }

    private function getUserInvalidLine():TextLine {
        var line:TextLine = getValidLine();
        line.validity = "userInvalid";
        return line;
    }

    private function getStaticLine():TextLine {
        var line:TextLine = getValidLine();
        line.validity = "static";
        return line;
    }

    private function getInvalidLine():TextLine {
        var element:TextElement = new TextElement("a", getElementFormat());
        var block:TextBlock = new TextBlock(element);
        var line:TextLine = block.createTextLine(null, 10000);
        block.content = element;
        return line;
    }

    private function getValidLine():TextLine {
        var block:TextBlock = new TextBlock(new TextElement("a", getElementFormat()));
        return block.createTextLine(null, 10000);
    }

    private function getElementFormat():ElementFormat {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "TestFont";
        fd.fontLookup = FontLookup.EMBEDDED_CFF;

        return new ElementFormat(fd, 20);
    }
}
}
