package
{
   import flash.display.MovieClip;
   import flash.events.Event;
   
   public class Main extends MovieClip
   {
      
      public var child:*;
      
      public var mainSideChild:*;
      
      public var secretChild:*;
      
      public function Main()
      {
         trace("Main Constructor");
         super();
         trace("Main addFrameScript");
         addFrameScript(0,this.frame1,1,this.frame2,2,this.frame3);
         addEventListener(Event.ADDED_TO_STAGE,added_to_stage);
      }
      
      internal function added_to_stage(event:*) : *
      {
         trace("Main added_to_stage");
         gotoAndStop(3);
      }
      
      internal function frame1() : *
      {
         trace("Main frame1");
      }
      
      internal function frame2() : *
      {
         trace("Main frame2");
         stop();
      }
      
      internal function frame3() : *
      {
         trace("Main frame3");
      }
   }
}

