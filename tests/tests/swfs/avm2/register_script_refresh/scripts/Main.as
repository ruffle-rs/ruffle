package
{
   import flash.display.MovieClip;
   
   public class Main extends MovieClip
   {
      public var child:*;
      
      public var repeats:uint = 0;
      
      public var descendant_names:Array = ["Container","Child","GrandChild","LeafChild"];
      
      public var descendant_order:Array = [0,0,0,0];
      
      public function Main()
      {
         trace("Main constructor");
         super();
         this.addScripts();
      }
      
      public function collectDescendants() : Array
      {
         return [this.child].concat(this.child.collectDescendants());
      }
      
      public function addScripts() : *
      {
         trace("Main addFrameScript");
         addFrameScript(0,this.frame1,1,this.frame2);
      }
      
      internal function frame1() : *
      {
         trace("Main frame1");
      }
      
      internal function frame2() : *
      {
         trace("Main frame2");
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

