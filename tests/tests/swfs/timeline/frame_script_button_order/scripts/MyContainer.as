package
{
   import flash.display.MovieClip;
   
   public class MyContainer extends MovieClip
   {
      public var otherChild:*;
      
      public var myOtherChild:*;
      
      public var dumbButton1:*;
      
      public var dumbButton2:*;
      
      public var dumbChild:*;
      
      public function MyContainer()
      {
         trace("MyContainer Constructor");
         super();
         trace("MyContainer addFrameScript");
      }
   }
}

