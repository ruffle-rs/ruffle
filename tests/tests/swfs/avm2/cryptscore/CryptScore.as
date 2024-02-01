package
{
   import flash.display.MovieClip;
   import flash.events.Event;
   
   public class CryptScore
   {
      
      private static var randamaroo:MovieClip;
       
      
      private var bs1:Number;
      
      private var a:Number;
      
      private var b:Number;
      
      private var bs2:Number;
      
      private var c:Number;
      
      private var bs3:Number;
      
      private var bs4:Number;
      
      private var d:Number;
      
      private var count:int = 0;
      
      public function CryptScore(param1:Number = 0, param2:Boolean = false)
      {
         super();
         if(randamaroo == null)
         {
            randamaroo = new MovieClip();
         }
         randamaroo.addEventListener(Event.ENTER_FRAME,this.randomise,false,0,true);
         this.count = Math.random() * 30;
         this.value = param1;
      }
      
      private function randomise(param1:Event) : void
      {
         ++this.count;
         if(this.count >= 15)
         {
            this.value = this.value;
            this.count -= 15;
         }
      }
      
      public function get value() : Number
      {
         return this.a - this.b + (this.c - this.d);
      }
      
      public function set value(param1:Number) : void
      {
         this.bs1 = param1;
         this.a = Math.random() * 32;
         this.bs2 = -this.a;
         do
         {
            this.b = Math.random() * 14;
            this.bs3 = -this.b;
         }
         while(false);
         
         var _loc2_:Number = param1 - (this.a - this.b);
         this.c = Math.random() * 23;
         this.d = this.c - _loc2_;
         this.bs4 = -param1;
      }
   }
}
