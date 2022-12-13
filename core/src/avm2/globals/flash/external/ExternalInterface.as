package flash.external
{
   public final class ExternalInterface
   {
      public static native function get available(): Boolean;
      
      public static native function addCallback(functionName: String, closure: Function) : void;
      
      public static native function call(functionName: String, ... arguments) : *;
   }
}
