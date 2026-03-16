package
{
   import flash.display.MovieClip;
   
   public class Test extends MovieClip
   {
      
      public function Test()
      {
         super();
         var sub:Subclass = new Subclass();
         trace(sub.field1);
         trace(sub["field1"]);
         trace(sub.field2);
         trace(sub["field2"]);
      }
   }
}

