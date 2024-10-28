// Test made using JPEXS Free Flash Decompiler.
// The frame rate in the SWF header is set to 0.0.

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
         trace("// stage.frameRate (SWF header = 0.0)");
         trace(stage.frameRate);
         trace("// stage.frameRate = 0;");
         stage.frameRate = 0;
         trace(stage.frameRate);
         trace("// stage.frameRate = 2000;");
         stage.frameRate = 2000;
         trace(stage.frameRate);
      }
   }
}
