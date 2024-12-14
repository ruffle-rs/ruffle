package
{
   import flash.accessibility.*;
   import flash.display.*;
   import flash.errors.*;
   import flash.events.*;
   import flash.filters.*;
   import flash.geom.*;
   import flash.media.*;
   import flash.net.*;
   import flash.net.drm.*;
   import flash.system.*;
   import flash.text.*;
   import flash.text.ime.*;
   import flash.ui.*;
   import flash.utils.*;
   
   public dynamic class Main extends MovieClip
   {
	   
	 public static var INSTANCE: Main;
       
      
      public var timeline_symbol:MyButton;
      
      public var my_button:*;
      
      public function Main()
      {
		 INSTANCE = this;
		 trace("Calling Main super()");
         super();
		 trace("Called Main super()");
         addFrameScript(0,this.frame1,1,this.frame2);
      }
      
      public function inspect_display_object(dobj:DisplayObject) : *
      {
         var i:* = undefined;
         trace(dobj);
         if(dobj instanceof DisplayObjectContainer)
         {
            trace("// numChildren: ",dobj.numChildren);
            for(i = 0; i < dobj.numChildren; i += 1)
            {
               trace(dobj.getChildAt(i));
            }
         }
      }
      
      internal function frame1() : *
      {
         trace("//var my_button = new MyButton();");
         this.my_button = new MyButton();
         trace("//this.addChild(my_button);");
         this.addChild(this.my_button);
         trace("//my_button");
         trace(this.my_button);
         trace("//my_button.upState");
         this.inspect_display_object(this.my_button.upState);
         trace("//my_button.overState");
         this.inspect_display_object(this.my_button.overState);
         trace("//my_button.downState");
         this.inspect_display_object(this.my_button.downState);
         trace("//my_button.hitTestState");
         this.inspect_display_object(this.my_button.hitTestState);
         trace("//my_button.upState = new UpButtonShape();");
         this.my_button.upState = new UpButtonShape();
         trace("//my_button.overState = new OverButtonShape();");
         this.my_button.overState = new OverButtonShape();
         trace("//my_button.downState = new DownButtonShape();");
         this.my_button.downState = new DownButtonShape();
         trace("//my_button.hitTestState = new HitButtonShape();");
         this.my_button.hitTestState = new HitButtonShape();
         trace("//my_button.upState");
         this.inspect_display_object(this.my_button.upState);
         trace("//my_button.overState");
         this.inspect_display_object(this.my_button.overState);
         trace("//my_button.downState");
         this.inspect_display_object(this.my_button.downState);
         trace("//my_button.hitTestState");
         this.inspect_display_object(this.my_button.hitTestState);
      }
      
      internal function frame2() : *
      {
         trace("//this.timeline_symbol.upState");
         this.inspect_display_object(this.timeline_symbol.upState);
         trace("//this.timeline_symbol.overState");
         this.inspect_display_object(this.timeline_symbol.overState);
         trace("//this.timeline_symbol.downState");
         this.inspect_display_object(this.timeline_symbol.downState);
         trace("//this.timeline_symbol.hitTestState");
         this.inspect_display_object(this.timeline_symbol.hitTestState);
         this.stop();
      }
   }
}
