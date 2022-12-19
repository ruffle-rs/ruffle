 
package flash.net
{
   import flash.events.EventDispatcher;

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

      native public function flush(minDiskSpace:int = 0) : String;
      native public function close() : void;
      native public function clear() : void;

      // note: this is supposed to be a read-only property
      public var data: Object;
      
      // note: this is supposed to be a read-only property
      public var size: uint;

      ruffle var _ruffleName: String;
   }
}
