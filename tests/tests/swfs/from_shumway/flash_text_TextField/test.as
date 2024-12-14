/*
 Compiled with: (from shumway tld, assuming the internal playerglobal.abc in the shumway checkout's parent dir)
 java -jar utils/asc.jar -AS3 -strict -import ../playerglobal.abc -in test/printers.as -swf TextFieldTest,600,600 test/swfs/flash_text_TextField.as
 */

package {
import flash.display.Sprite;
import flash.events.Event;

public class TextFieldTest extends Sprite {
  public function TextFieldTest() {
    stage.frameRate = 20;
    child = new TextFieldObject();
    child.width = 200;
    child.height = 20;
    addChild(child);
    child.runSyncTests();
    child.runAsyncTests();
  }

  public var child;
}
}

import flash.display.*;
import flash.events.*;
import flash.text.*;

import shunit.*;

class TextFieldObject extends TextField {
  public function TextFieldObject() {
    addEventListener(MouseEvent.CLICK, clickHandler);
    addEventListener(MouseEvent.MOUSE_UP, mouseUpHandler);
  }

  function clickHandler(e) {
    trace("click");
  }

  function mouseUpHandler(e) {
    trace("mouseUp");
  }

  public function runSyncTests() {
    shunit.printSuccess = false;
    printEquals(autoSize, TextFieldAutoSize.NONE, "flash.text::TextField/get autoSize() // default value");
    autoSize = TextFieldAutoSize.CENTER;
    printEquals(autoSize, TextFieldAutoSize.CENTER, "flash.text::TextField/{get|set} autoSize()");
    autoSize = TextFieldAutoSize.LEFT;
    printEquals(autoSize, TextFieldAutoSize.LEFT, "flash.text::TextField/{get|set} autoSize()");
    autoSize = TextFieldAutoSize.RIGHT;
    printEquals(autoSize, TextFieldAutoSize.RIGHT, "flash.text::TextField/{get|set} autoSize()");
    autoSize = TextFieldAutoSize.NONE;
    printEquals(autoSize, TextFieldAutoSize.NONE, "flash.text::TextField/{get|set} autoSize()");

    printEquals(defaultTextFormat.size, 12, "flash.text::TextField/{get|set} defaultTextFormat()");

    printEquals(text, '', "flash.text::TextField/get text() // default value");
    printEquals(htmlText, '', "flash.text::TextField/get htmlText() // default value");
    printEquals(textWidth, 0, "flash.text::TextField/get textWidth() // default value");
    printEquals(textHeight, 0, "flash.text::TextField/get textHeight() // default value");
    var testText = "hello, world!";
    text = testText;
    printEquals(text, testText, "flash.text::TextField/{get|set} text()");
//    printEquals(htmlText, '<P ALIGN="LEFT"><FONT FACE="Times Roman" SIZE="12" COLOR="#000000" LETTERSPACING="0" KERNING="0">hello, world!</FONT></P>',
//                "flash.text::TextField/set text(), get htmlText()");
//    printEquals(textHeight, 12, "flash.text::TextField/get textHeight()");
    htmlText = '<FONT FACE="Times New Roman" SIZE="30px" COLOR="#00ff00" LETTERSPACING="2" KERNING="0"><B><invalid>foo</invalid></B></FONT></P';
    printEquals(text, "foo", "flash.text::TextField/set htmlText(), get text()");
//    printEquals(htmlText, '<P ALIGN="LEFT"><FONT FACE="Times New Roman" SIZE="30" COLOR="#00FF00" LETTERSPACING="2" KERNING="0"><B>foo</B></FONT></P>',
//                "flash.text::TextField/{get|set} htmlText()");

    printEquals(selectable, true, "flash.text::TextField/get selectable() // default value");
    selectable = false;
    printEquals(selectable, false, "flash.text::TextField/{get|set} selectable()");
    selectable = true;
    printEquals(selectable, true, "flash.text::TextField/{get|set} selectable()");

    printEquals(wordWrap, false, "flash.text::TextField/get wordWrap() // default value");
    wordWrap = true;
    printEquals(wordWrap, true, "flash.text::TextField/{get|set} wordWrap()");
    wordWrap = false;
    printEquals(wordWrap, false, "flash.text::TextField/{get|set} wordWrap()");

    var originalFormat = new TextFormat("Verdana", 20);
    setTextFormat(originalFormat);
    originalFormat.size = 10;
    var format = getTextFormat() || {};
    printTruthy(format.size != originalFormat.size, "flash.text::TextField/set TextFormat() copies result");
    printTruthy(format != originalFormat, "flash.text::TextField/get TextFormat() copies result");
    printEquals(format.size+0, 20, "flash.text::TextField/{get|set}TextFormat()");

    printEquals(background, false, "flash.text::TextField/get background() // default value");
    background = true;
    printEquals(background, true, "flash.text::TextField/{get|set} background()");
    background = false;
    printEquals(background, false, "flash.text::TextField/{get|set} background()");

    printEquals(backgroundColor, 0xFFFFFF, "flash.text::TextField/get backgroundColor() // default value");
    backgroundColor = 0x00FF00;
    printEquals(backgroundColor, 0x00FF00, "flash.text::TextField/{get|set} backgroundColor()");
    backgroundColor = 0x00FFFF;
    printEquals(backgroundColor, 0x00FFFF, "flash.text::TextField/{get|set} backgroundColor()");

    printEquals(border, false, "flash.text::TextField/get border() // default value");
    border = true;
    printEquals(border, true, "flash.text::TextField/{get|set} border()");
    border = false;
    printEquals(border, false, "flash.text::TextField/{get|set} border()");

    printEquals(borderColor, 0x000000, "flash.text::TextField/get borderColor() // default value");
    borderColor = 0x00FF00;
    printEquals(borderColor, 0x00FF00, "flash.text::TextField/{get|set} borderColor()");
    borderColor = 0x00FFFF;
    printEquals(borderColor, 0x00FFFF, "flash.text::TextField/{get|set} borderColor()");

    printEquals(condenseWhite, false, "flash.text::TextField/get condenseWhite() // default value");
    condenseWhite = true;
    printEquals(condenseWhite, true, "flash.text::TextField/{get|set} condenseWhite()");
    condenseWhite = false;
    printEquals(condenseWhite, false, "flash.text::TextField/{get|set} condenseWhite()");

    printEquals(multiline, false, "flash.text::TextField/get multiline() // default value");
    multiline = true;
    printEquals(multiline, true, "flash.text::TextField/{get|set} multiline()");
    multiline = false;
    printEquals(multiline, false, "flash.text::TextField/{get|set} multiline()");

    var format = new TextFormat('Arial');
    defaultTextFormat = format;
    multiline = true;

    htmlText = '<textformat leading="2">line1</textformat> <font size="14">bigger</font> ' +
               '<textformat leading="4">ignored leading</textformat>';
    var metrics = getLineMetrics(0);
    printEquals(metrics.x, 2, 'flash.text::TextField/getTextLineMetrics().x');
//    printEquals(metrics.width, 151, 'flash.text::TextField/getTextLineMetrics().width');
    printEquals(metrics.height, 18, 'flash.text::TextField/getTextLineMetrics().height');
    printEquals(metrics.ascent, 13, 'flash.text::TextField/getTextLineMetrics().ascent');
    printEquals(metrics.descent, 3, 'flash.text::TextField/getTextLineMetrics().descent');
    printEquals(metrics.leading, 2, 'flash.text::TextField/getTextLineMetrics().leading');

    text = 'line1';
    printEquals(numLines, 1, "flash.text::TextField/get numLines()");
    printEquals(scrollV, 1, "flash.text::TextField/get scrollV()");
    printEquals(maxScrollV, 1, "flash.text::TextField/get maxScrollV()");
    printEquals(bottomScrollV, 1, "flash.text::TextField/get bottomScrollV()");
    text = 'line1\nline2';
    printEquals(numLines, 2, "flash.text::TextField/get numLines()");
    printEquals(scrollV, 1, "flash.text::TextField/get scrollV()");
    printEquals(maxScrollV, 2, "flash.text::TextField/get maxScrollV()");
    printEquals(bottomScrollV, 1, "flash.text::TextField/get bottomScrollV()");
    scrollV = 2;
    printEquals(scrollV, 2, "flash.text::TextField/get scrollV()");
    printEquals(maxScrollV, 2, "flash.text::TextField/get maxScrollV()");
    printEquals(bottomScrollV, 2, "flash.text::TextField/get bottomScrollV()");
    height = 300;
    scrollV = 1;
//    trace(scrollV, maxScrollV, bottomScrollV);
    text = 'line1\nline2';
//    trace(scrollV, maxScrollV, bottomScrollV);
  }

  public function runAsyncTests() {
    addEventListener(Event.ENTER_FRAME, enterFrameHandler);
  }

  private var frameCount = 0;

  function enterFrameHandler(event:Event):void {
    frameCount++;
    var target = event.target;
    switch (frameCount) {
      default:
        removeEventListener("enterFrame", enterFrameHandler);
        break;
    }
  }
}
