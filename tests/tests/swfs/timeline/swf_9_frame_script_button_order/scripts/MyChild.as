package
{
   import flash.display.MovieClip;
   
   public class MyChild extends MovieClip
   {
      public var myField:uint;
      
      public var myFieldWithInit:Object = "Default value";
      
      public function MyChild()
      {
         trace("MyChild Constructor");
         super();
         trace("MyChild addFrameScript");
      }
   }
}

