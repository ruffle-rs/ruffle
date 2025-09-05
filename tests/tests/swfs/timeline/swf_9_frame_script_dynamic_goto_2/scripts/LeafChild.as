package
{
   import flash.display.MovieClip;
   import flash.events.Event;
   
   [Embed(source="/_assets/assets.swf", symbol="symbol11")]
   public class LeafChild extends MovieClip
   {
      
      public static var counter:uint = 1;
      
      public var id:uint = counter;
      
      public var repeats:uint = 0;
      
      public function LeafChild()
      {
         counter += 1;
         trace("LeafChild",this.id,"constructor");
         super();
         trace("LeafChild",this.id,"addFrameScript");
         addFrameScript(0,this.frame1,1,this.frame2);
      }
      
      internal function frame1() : *
      {
         trace("LeafChild",this.id,"frame1");
         if(this.repeats == 0 && this.id == 1)
         {
            addEventListener(Event.ENTER_FRAME,enter_frame);
         }
      }
      
      internal function frame2() : *
      {
         if(this.id != 2)
         {
            trace("LeafChild",this.id,"frame2");
         }
         if(this.repeats > 0)
         {
            stop();
         }
         this.repeats += 1;
      }
      
      internal function enter_frame(event:*) : *
      {
         trace("LeafChild",this.id,"enter frame");
         if(this.repeats > 0)
         {
            removeEventListener(Event.ENTER_FRAME,enter_frame);
         }
      }
   }
}

