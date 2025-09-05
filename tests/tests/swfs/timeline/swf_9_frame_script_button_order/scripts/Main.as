package
{
   import flash.display.MovieClip;
   
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
      }
   }
}

