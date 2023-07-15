package flash.net
{
   import flash.events.EventDispatcher;
   import __ruffle__.stub_method;

   namespace ruffle = "__ruffle__";
   
   public class SharedObject extends EventDispatcher
   {
      public function SharedObject()
      {
         this.data = {};
      }

      // NOTE: We currently always use AMF3 serialization.
      // If you implement the `defaultObjectEncoding` or `objectEncoding`,
      // you will need to adjust the serialization and deserialization code
      // to work with AMF0.

      native public static function getLocal(name:String, localPath:String = null, secure:Boolean = false): SharedObject;

      native public function get size() : uint;

      native public function flush(minDiskSpace:int = 0) : String;
      native public function close() : void;
      native public function clear() : void;

      public function setProperty(propertyName:String, value:Object = null):void {
         stub_method("flash.net.SharedObject", "setProperty");
      }

      // note: this is supposed to be a read-only property
      public var data: Object;

      ruffle var _ruffleName: String;
   }
}
