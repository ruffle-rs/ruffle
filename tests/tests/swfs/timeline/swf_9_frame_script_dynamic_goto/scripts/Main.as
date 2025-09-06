package
{
   import flash.display.MovieClip;
   
   public class Main extends MovieClip
   {
      
      public var child1:*;
      
      public var child2:*;
      
      public var child3:*;
      
      public var repeats:uint = 0;
      
      public function Main()
      {
         trace("Main constructor");
         super();
         trace("Main addFrameScript");
         addFrameScript(0,this.frame1,1,this.frame2);
      }
      
      internal function frame1() : *
      {
         trace("Main frame1");
         if(this.repeats == 0)
         {
            this.child1 = new LeafChild();
            this.child1.x = 400;
            this.child1.y = 285;
            trace("adding child1 at",0);
            addChildAt(this.child1,0);
            this.child2 = new NodeChild();
            this.child2.x = 0;
            this.child2.y = 0;
            trace("adding child2 at",0);
            addChildAt(this.child2,0);
            this.child3 = new LeafChild();
            this.child3.x = 285;
            this.child3.y = 100;
            trace("adding child3 at",2);
            addChildAt(this.child3,2);
         }
      }
      
      internal function frame2() : *
      {
         trace("Main frame2");
         if(this.repeats > 0)
         {
            stop();
         }
         this.repeats += 1;
      }
   }
}

