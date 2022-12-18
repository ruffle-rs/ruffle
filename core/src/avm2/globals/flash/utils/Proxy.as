 
package flash.utils
{
   import flash.errors.IllegalOperationError;

   public namespace flash_proxy = "http://www.adobe.com/2006/actionscript/flash/proxy";
   
   [Ruffle(InstanceAllocator)]
   public class Proxy
   {
      flash_proxy function getProperty(name:*) : *
      {
         throw new IllegalOperationError("Error #2088: The Proxy class does not implement getProperty. It must be overridden by a subclass.", 2088);
      }
      
      flash_proxy function setProperty(name:*, value:*) : void
      {
         throw new IllegalOperationError("Error #2089: The Proxy class does not implement setProperty. It must be overridden by a subclass.", 2089);
      }
      
      flash_proxy function callProperty(name:*, ... rest) : *
      {
         throw new IllegalOperationError("Error #2090: The Proxy class does not implement callProperty. It must be overridden by a subclass.", 2090);
      }
      
      flash_proxy function hasProperty(name:*) : Boolean
      {
         throw new IllegalOperationError("Error #2091: The Proxy class does not implement hasProperty. It must be overridden by a subclass.", 2091);
      }
      
      flash_proxy function deleteProperty(name:*) : Boolean
      {
         throw new IllegalOperationError("Error #2092: The Proxy class does not implement deleteProperty. It must be overridden by a subclass.", 2092);
      }
      
      flash_proxy function getDescendants(name:*) : *
      {
         throw new IllegalOperationError("Error #2093: The Proxy class does not implement getDescendants. It must be overridden by a subclass.", 2093);
      }
      
      flash_proxy function nextNameIndex(index:int) : int
      {
         throw new IllegalOperationError("Error #2105: The Proxy class does not implement nextNameIndex. It must be overridden by a subclass.", 2105);
      }
      
      flash_proxy function nextName(index:int) : String
      {
         throw new IllegalOperationError("Error #2106: The Proxy class does not implement nextName. It must be overridden by a subclass.", 2106);
      }
      
      flash_proxy function nextValue(index:int) : *
      {
         throw new IllegalOperationError("Error #2107: The Proxy class does not implement nextValue. It must be overridden by a subclass.", 2107);
      }
      
      native flash_proxy function isAttribute(name:*) : Boolean;
   }
}
