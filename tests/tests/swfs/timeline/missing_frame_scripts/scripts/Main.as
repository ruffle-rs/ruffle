package
{
   import flash.display.MovieClip;
   
   public class Main extends MovieClip
   {
      public var child:*;
      
      public var mainSideChild:*;
      
      public var secretChild:*;
      
      public var repeats:uint = 0;
      
      public function Main()
      {
         trace("Main Constructor");
         super();
         trace("Main addFrameScript");
         addFrameScript(0,this.frame1,1,this.frame2);
      }
      
      internal function frame1() : *
      {
         trace("Main frame1");
      }
      
      internal function frame2() : *
      {
         trace("Main frame2");
         if(this.repeats > 1)
         {
            trace("Main stopped");
            stop();
         }
         else
         {
            this.repeats += 1;
         }
      }
   }
}

