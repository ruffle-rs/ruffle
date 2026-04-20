package
{
   import flash.display.MovieClip;
   
   [Embed(source="/_assets/assets.swf", symbol="symbol12")]
   public class MyContainer extends MovieClip
   {
      public var myChild1:*;
      
      public var myChild2:*;
      
      public var repeats:uint = 0;
      
      public function MyContainer()
      {
         trace("Container Constructor");
         super();
         trace("Container addFrameScript");
         addFrameScript(0,this.frame1,1,this.frame2);
      }
      
      internal function frame1() : *
      {
         trace("Container frame1");
      }
      
      internal function frame2() : *
      {
         trace("Container frame2");
         if(this.repeats > 1)
         {
            trace("Container stopped");
            stop();
         }
         else
         {
            this.repeats += 1;
         }
      }
   }
}

