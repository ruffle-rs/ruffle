package
{
   import flash.display.MovieClip;
   
   [Embed(source="/_assets/assets.swf", symbol="symbol2")]
   public class MyChild extends MovieClip
   {
      
      public var myField:uint;
      
      public var myFieldWithInit:Object = "Default value";
      
      public function MyChild()
      {
         trace("MyChild Constructor");
         super();
      }
   }
}

