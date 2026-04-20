// note: source is informational, the test relies on the tags inside

package
{
   import flash.display.MovieClip;
   import flash.text.TextField;
   import flash.text.TextFormat;

   public class Main extends MovieClip
   {
      public var text1:TextField;
      public var text2:TextField;
      public var text3:TextField;
      public var text4:TextField;
      public var text5:TextField;

      public function Main()
      {
         super();
         stage.scaleMode = "noScale";
         traceIndent(text1);
         traceIndent(text2);
         traceIndent(text3);
         traceIndent(text4);
         traceIndent(text5);
         testSettingIndent(0);
         testSettingIndent(0.1);
         testSettingIndent(0.4);
         testSettingIndent(0.5);
         testSettingIndent(0.6);
         testSettingIndent(0.8);
         testSettingIndent(1);
         testSettingIndent(1.1);
         testSettingIndent(1.4);
         testSettingIndent(1.5);
         testSettingIndent(1.6);
         testSettingIndent(1.8);
         testSettingIndent(2);
         testSettingIndent(-10);
         testSettingIndent(100);
         testSettingIndent(64010);
         printLines(text1);
         printLines(text2);
         printLines(text3);
         printLines(text4);
         printLines(text5);
      }

      internal function printLines(tf:*) : *
      {
         var line:*;
         trace("lines for " + tf.name + ":");
         line = 0;
         while(line < 3)
         {
            try
            {
               trace(tf.getLineText(line));
            }
            catch(e:Error)
            {
               trace("");
            }
            line += 1;
         }
      }

      internal function traceIndent(tf:*) : *
      {
         trace("Indent for " + tf.name + ":");
         trace(tf.defaultTextFormat.indent);
         trace(tf.getTextFormat(0,1).indent);
      }

      internal function testSettingIndent(value:*) : *
      {
         var _loc2_:* = new TextFormat();
         _loc2_.indent = value;
         text1.defaultTextFormat = _loc2_;
         trace("Setting " + value + " -> " + text1.defaultTextFormat.indent);
      }
   }
}
