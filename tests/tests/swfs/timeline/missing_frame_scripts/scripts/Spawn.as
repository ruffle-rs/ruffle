package
{
   import flash.display.MovieClip;
   
   [Embed(source="/_assets/assets.swf", symbol="symbol13")]
   public class Spawn extends MovieClip
   {
      public var secretChild:*;
      
      public var repeats:uint = 0;
      
      public function Spawn()
      {
         trace("Spawn Constructor");
         super();
         trace("Spawn addFrameScript");
         addFrameScript(0,this.frame1,1,this.frame2);
      }
      
      internal function frame1() : *
      {
         trace("Spawn frame1");
         if(this.repeats == 0)
         {
            this.secretChild = new MyContainer();
         }
      }
      
      internal function frame2() : *
      {
         trace("Spawn frame2");
         if(this.repeats > 1)
         {
            trace("Spawn stopped");
            stop();
         }
         else
         {
            this.repeats += 1;
         }
      }
   }
}

