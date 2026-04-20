package
{
   import flash.display.MovieClip;

   public class Test extends MovieClip
   {
      public function Test()
      {
         super();
      }
   }
}

import flash.display.BitmapData;
import flash.geom.ColorTransform;
import flash.geom.Rectangle;

var bitmap:BitmapData = new BitmapData(8,8,false,0);

// Try to do a colorTransform with an out of bounds Rectangle.
// This doesn't crash FP
bitmap.colorTransform(new Rectangle(9,9,10,10),new ColorTransform(1,1,1,1,1,1,1,1));
trace("// bitmap.rect (sanity check)");
trace(bitmap.rect.toString());

