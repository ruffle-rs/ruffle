package
{
   import flash.display.MovieClip;
   
   public class MyLeaf extends MovieClip
   {
      public static var counter:uint = 1;
      
      public var id:uint = counter;
      
      public var repeats:uint = 0;
      
      public var name:String = "";
      
      public function MyLeaf()
      {
         if(this.id == 1)
         {
            this.name = "GrandChild";
         }
         else
         {
            this.name = "LeafChild";
         }
         counter += 1;
         trace(this.name + " constructor");
         super();
      }
      
      public function collectDescendants() : Array
      {
         return [];
      }
      
      public function addScripts() : *
      {
         trace(this.name + " addFrameScript");
         addFrameScript(0,this.frame1,1,this.frame2);
      }
      
      internal function frame1() : *
      {
         trace(this.name + " frame1");
      }
      
      internal function frame2() : *
      {
         trace(this.name + " frame2");
         if(this.repeats > 0)
         {
            stop();
         }
         else
         {
            this.repeats += 1;
         }
      }
   }
}

