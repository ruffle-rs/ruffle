package
{
   import flash.display.MovieClip;
   
   [Embed(source="/_assets/assets.swf", symbol="symbol13")]
   public class NodeChild extends MovieClip
   {
      
      public var repeats:uint = 0;
      
      public var child:*;
      
      public function NodeChild()
      {
         trace("NodeChild constructor");
         super();
         trace("NodeChild addFrameScript");
         addFrameScript(0,this.frame1,1,this.frame2);
      }
      
      internal function frame1() : *
      {
         trace("NodeChild frame1");
         if(this.repeats == 0)
         {
            this.child.gotoAndStop(2);
         }
      }
      
      internal function frame2() : *
      {
         trace("NodeChild frame2");
         if(this.repeats > 0)
         {
            stop();
         }
         this.repeats += 1;
      }
   }
}

