// Test made using JPEXS Free Flash Decompiler.
// The frame rate in the SWF header is set to 30.

package
{
   import flash.display.Sprite;
   import flash.events.Event;
   
   public class Test extends Sprite
   {
      public function Test()
      {
         super();
         this.init();
      }
      
      private function init() : void
      {
         var enterFrame = function()
         {
            trace("// Event.ENTER_FRAME");
         };
         addEventListener(Event.ENTER_FRAME, enterFrame);
         trace("// stage.frameRate (SWF header = 30)");
         trace(stage.frameRate);
         trace("// stage.frameRate = NaN;");
         stage.frameRate = NaN;
         trace(stage.frameRate);
      }
   }
}
