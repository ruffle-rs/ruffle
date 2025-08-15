// Test made using JPEXS Free Flash Decompiler.
// The frame rate in the SWF header is set to -43.75.

package
{
   import flash.display.Sprite;
   
   public class Test extends Sprite
   {
      public function Test()
      {
         super();
         this.init();
      }
      
      private function init() : void
      {
         trace("// stage.frameRate (SWF header = -43.75)");
         trace(stage.frameRate);
         trace("// stage.frameRate = -43.75;");
         stage.frameRate = -43.75;
         trace(stage.frameRate);
         trace("// stage.frameRate = 212.25;");
         stage.frameRate = 212.25;
         trace(stage.frameRate);
      }
   }
}
