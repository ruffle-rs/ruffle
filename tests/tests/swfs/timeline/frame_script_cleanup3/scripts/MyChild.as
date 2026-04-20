package
{
   import flash.display.MovieClip;
   
   public class MyChild extends MovieClip
   {
      public static var counter:uint = 1;
      
      public var grandchild:*;
      
      public var id:uint = counter;
      
      public var repeats:uint = 0;
      
      public var name:String = "Child";
      
      public function MyChild()
      {
         counter += 1;
         trace(this.name + " constructor");
         super();
      }
      
      public function collectDescendants() : Array
      {
         return [this.grandchild].concat(this.grandchild.collectDescendants());
      }
      
      public function addScripts() : *
      {
         trace(this.name + " addFrameScript");
         addFrameScript(0,this.frame1,1,this.frame2);
      }
      
      public function addMoreScripts() : *
      {
         trace(this.name + " more addFrameScript");
         addFrameScript(1,this.frame2_extra);
      }
      
      internal function frame1() : *
      {
         trace(this.name + " frame1");
      }
      
      internal function frame2() : *
      {
         trace(this.name + " frame2");
      }
      
      internal function frame2_extra() : *
      {
         trace(this.name + " extra frame2");
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

