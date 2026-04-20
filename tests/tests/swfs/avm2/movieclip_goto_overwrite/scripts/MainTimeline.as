package test_fla
{
   import flash.display.MovieClip;
   
   public dynamic class MainTimeline extends MovieClip
   {
      public var again:Boolean = false;
      
      public function MainTimeline()
      {
         super();
         addFrameScript(0,this.frame1,1,this.frame2,2,this.frame3,3,this.frame4,4,this.frame5,5,this.frame6,6,this.frame7,7,this.frame8,8,this.frame9,9,this.frame10,10,this.frame11,11,this.frame12,12,this.frame13,13,this.frame14,14,this.frame15);
      }
      
      internal function frame1() : *
      {
         trace("//Scene 1, Frame 1");
         if(this.again)
         {
            this.stop();
         }
         else
         {
            trace("//gotoAndStop(4, \"Scene 1\");");
            this.gotoAndPlay(2);
            this.gotoAndStop(4,"Scene 1");
         }
      }
      
      internal function frame2() : *
      {
         trace("//TEST FAIL: Scene 1, Frame 2");
         this.stop();
      }
      
      internal function frame3() : *
      {
         trace("//TEST FAIL: Scene 1, Frame 3");
         this.stop();
      }
      
      internal function frame4() : *
      {
         trace("//Scene 1, Frame 4");
         trace("//gotoAndStop(6);");
         this.gotoAndStop(3);
         this.gotoAndStop(6);
      }
      
      internal function frame5() : *
      {
         trace("//TEST FAIL: Scene 1, Frame 5");
      }
      
      internal function frame6() : *
      {
         trace("//Scene 2, Frame 1");
         trace("//gotoAndStop(\"frame3\");");
         this.gotoAndPlay(5,"Scene 1");
         this.gotoAndPlay("frame3");
      }
      
      internal function frame7() : *
      {
         trace("//TEST FAIL: Scene 2, Frame 2");
      }
      
      internal function frame8() : *
      {
         trace("//Scene 2, Frame 3");
         trace("//gotoAndStop(\"frame2\", \"Scene 3\");");
         this.gotoAndStop("frame2","Scene 3");
      }
      
      internal function frame9() : *
      {
         trace("//TEST FAIL: Scene 3, Frame 1");
      }
      
      internal function frame10() : *
      {
         trace("//Scene 3, Frame 2");
         trace("//gotoAndStop(2.1, \"Scene 4\");");
         this.gotoAndStop(2.1,"Scene 4");
      }
      
      internal function frame11() : *
      {
         trace("//TEST FAIL: Scene 4, Frame 1");
         this.stop();
      }
      
      internal function frame12() : *
      {
         trace("//TEST FAIL: Scene 4, Frame 2");
         this.stop();
      }
      
      internal function frame13() : *
      {
         trace("//Scene 4, Frame 3");
         trace("//gotoAndStop(\"2\", \"Scene 5\");");
         this.gotoAndPlay("2","Scene 5");
      }
      
      internal function frame14() : *
      {
         trace("//TEST FAIL: Scene 5, Frame 1");
         this.stop();
      }
      
      internal function frame15() : *
      {
         trace("//Scene 5, Frame 2");
         this.gotoAndStop(2,"Scene 1");
         this.again = true;
         this.gotoAndPlay(this.currentFrame);
      }
   }
}

