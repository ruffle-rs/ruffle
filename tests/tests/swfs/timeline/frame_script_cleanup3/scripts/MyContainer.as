package
{
   import flash.display.MovieClip;
   
   public class MyContainer extends MovieClip
   {
      public var myChild1:*;
      
      public var myChild2:*;
      
      public var repeats:uint = 0;
      
      public function MyContainer()
      {
         trace("Container constructor");
         super();
      }
      
      public function collectDescendants() : Array
      {
         return [this.myChild1].concat(this.myChild1.collectDescendants()).concat([this.myChild2]).concat(this.myChild2.collectDescendants());
      }
      
      public function addScripts() : *
      {
         trace("Container addFrameScript");
         addFrameScript(0,this.frame1,1,this.frame2);
      }
      
      internal function frame1() : *
      {
         trace("Container frame1");
      }
      
      internal function frame2() : *
      {
         trace("Container frame2");
         if(this.repeats == 0)
         {
            this.myChild1.addMoreScripts();
         }
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

