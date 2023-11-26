package
{
   import flash.display.*;
   import flash.events.*;
   import flash.system.*;
   
   public class Main extends EventWatcher
   {
      
      public static var frame1ScriptRan:Array = [];
      
      public static var frame2ScriptRan:Array = [];
       
      
      public var my_button:SimpleButton;
	   
	  public static var INSTANCE: Main;
      
      public function Main()
      {
		 INSTANCE = this;
         var self:*;
         
         addFrameScript(0,this.frame1,1,this.frame2);
         self = this;
         this.addEventListener(Event.EXIT_FRAME,function(e:*):*
         {
            frame1ScriptRan.sort();
            frame2ScriptRan.sort();
            trace("frame1ScriptRan = " + frame1ScriptRan);
            trace("frame2ScriptRan = " + frame2ScriptRan);
            frame1ScriptRan = [];
            frame2ScriptRan = [];
         });
         trace("Calling Main super()");
         super();
      }
      
      public function stop_display_object_handlers(dobj:DisplayObject) : *
      {
         var i:* = undefined;
         if(dobj instanceof EventWatcher)
         {
            dobj.destroy();
         }
         if(dobj instanceof DisplayObjectContainer)
         {
            for(i = 0; i < dobj.numChildren; i += 1)
            {
               this.stop_display_object_handlers(dobj.getChildAt(i));
            }
         }
      }
      
      internal function frame1() : *
      {
         Main.frame1ScriptRan.push("MainTimeline");
      }
      
      internal function frame2() : *
      {
         Main.frame2ScriptRan.push("MainTimeline");
         this.stop();
         this.stop_display_object_handlers(this.my_button.upState);
         this.stop_display_object_handlers(this.my_button.downState);
         this.stop_display_object_handlers(this.my_button.overState);
         this.stop_display_object_handlers(this.my_button.hitTestState);
         System.gc();
      }
   }
}
