package {
   import flash.display.MovieClip;
   import flash.utils.*;

   public class Test extends MovieClip {
      public function Test() {
         var t1:Function = new Function();
         var t2:Vector.<Function> = Vector.<Function>([new Function(),new Function(),new Function()]);
         var t3:Vector.<Function> = Vector.<Function>([null,new Function()]);
         var t4:Vector.<Function> = Vector.<Function>([]);
         var t5:Array = [new Function(),new Function()];
         runTest("Just function",0,t1);
         runTest("Just function",3,t1);
         runTest("Function vector",3,t2);
         runTest("Function vector with null element",3,t3);
         runTest("Empty function vector",3,t4);
         runTest("Array with two function elements",3,t5);
      }
      
      public function printByteArray(name:String, array:ByteArray):void {
         trace("Printing ByteArray:");
         var str:* = "";
         var i:* = 0;
         while(i < array.length)
         {
            str += array[i];
            if(i != array.length - 1)
            {
               str += ", ";
            }
            i++;
         }
         trace(str);
      }
      
      public function runTest(name:String, amfversion:*, obj:*):void {
         trace("Running test \"" + name + "\" with AMF" + amfversion);
         var bytearray:* = new ByteArray();
         bytearray.objectEncoding = amfversion;
         bytearray.writeObject(obj);
         printByteArray(name,bytearray);
         bytearray.position = 0;
         var read:* = bytearray.readObject();
         trace("read back: " + getQualifiedClassName(read));
         if(read && read instanceof Vector.<*>)
         {
            trace("Was vector, length " + read.length + ". Elements:");
            for(var i in read)
            {
               trace(i + "th element: " + read[i]);
            }
         }
         if(read && read instanceof Array)
         {
            trace("Was array, length " + read.length + ". Elements:");
            for(i in read)
            {
               trace(i + "th element: " + read[i]);
            }
         }
         trace(read);
         trace("Done with test!");
      }
   }
}

