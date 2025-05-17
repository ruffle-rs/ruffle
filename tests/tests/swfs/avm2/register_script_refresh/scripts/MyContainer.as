package
{
   import flash.display.MovieClip;
   import flash.events.*;
   
   public class MyContainer extends MovieClip
   {
      public var myChild1:*;
      
      public var myChild2:*;
      
      public var went_once:Boolean = false;
      
      public var repeats:uint = 0;
      
      public function MyContainer()
      {
         trace("Container constructor");
         super();
         this.addScripts()
         addEventListener(Event.ENTER_FRAME,this.enter_frame);
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
      
      internal function enter_frame(param1:Event = null) : *
      {
         trace("entered frame", this.currentFrame);
         if (!went_once) {
           went_once = true;
           trace("calling goto on grandchild");
           myChild1.grandchild.gotoAndStop(1);
           trace("calling goto on child");
           myChild1.gotoAndPlay(1);
           trace("adding frame-script to grandchild");
           myChild1.addFrameScript(0,this.secret_frame);
         }
      }
      
      internal function secret_frame() : *
      {
         trace("secret reached!");
      }
      
      internal function frame1() : *
      {
         trace("Container frame1");
      }
      
      internal function frame2() : *
      {
         trace("Container frame2");
         if(this.repeats > 0)
         {
            removeEventListener(Event.ENTER_FRAME,this.enter_frame);
            stop();
         }
         else
         {
            this.repeats += 1;
         }
      }
   }
}

