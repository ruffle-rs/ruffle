/*
   Compiled with:
   java -jar utils/asc.jar -import playerglobal.abc -swf ZeroClipboardTest,600,600 test/swfs/ZeroClipboardTest.as
*/

﻿package 
{
    import flash.display.*;
    import flash.events.*;
    import flash.system.*;
    import flash.utils.*;
    import flash.geom.*;
    
    public class ZeroClipboardTest extends flash.display.Sprite
    {
        public function ZeroClipboardTest()
        {
            super();
            stage.align = "TL";
            stage.scaleMode = "noScale";
            this.button = new flash.display.Sprite();
            this.button.buttonMode = true;
            this.button.useHandCursor = false;
            this.button.graphics.beginFill(13434624);
            this.button.graphics.drawRect(0, 0, stage.stageWidth, stage.stageHeight);
            this.button.alpha = 0.8;
            addChild(this.button);

            this.button.addEventListener(flash.events.MouseEvent.CLICK, this.mouseClick);

            trace('init');
            setTimeout(function () {
              setSize(100, 100);
            }, 10);
        }

        internal function mouseClick(e:flash.events.MouseEvent):void
        {
          trace('mouseclick');
        }

        public function setSize(w:Number, h:Number):void
        {
            trace("setSize " + w + "," + h);
            this.button.width = w;
            this.button.height = h;
        }

        internal var button:flash.display.Sprite;
    }
}
