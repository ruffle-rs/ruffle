 
package
{
   import flash.display.MovieClip;
   import flash.net.SharedObject;
   
   public class Test extends MovieClip
   {
       
      
      public function Test()
      {
         stage.removeChildAt(0);
         var so:SharedObject = SharedObject.getLocal("testObject");
         super();
         trace(so.data);
         trace(so.data.A);
         so.data.A = 1;
         trace(so.data.A);
      }
   }
}

