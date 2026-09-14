package flash.utils {
    import flash.errors.IllegalOperationError;

    public namespace flash_proxy = "http://www.adobe.com/2006/actionscript/flash/proxy";

    [Ruffle(InstanceAllocator)]
    public class Proxy {
        [Ruffle(NativeCallable)]
        flash_proxy function getProperty(name:*):* {
            Error.throwError(IllegalOperationError, 2088);
        }

        [Ruffle(NativeCallable)]
        flash_proxy function setProperty(name:*, value:*):void {
            Error.throwError(IllegalOperationError, 2089);
        }

        [Ruffle(NativeCallable)]
        flash_proxy function callProperty(name:*, ...rest):* {
            Error.throwError(IllegalOperationError, 2090);
        }

        [Ruffle(NativeCallable)]
        flash_proxy function hasProperty(name:*):Boolean {
            Error.throwError(IllegalOperationError, 2091);
        }

        [Ruffle(NativeCallable)]
        flash_proxy function deleteProperty(name:*):Boolean {
            Error.throwError(IllegalOperationError, 2092);
        }

        // TODO implement this
        flash_proxy function getDescendants(name:*):* {
            Error.throwError(IllegalOperationError, 2093);
        }

        [Ruffle(NativeCallable)]
        flash_proxy function nextNameIndex(index:int):int {
            Error.throwError(IllegalOperationError, 2105);
        }

        [Ruffle(NativeCallable)]
        flash_proxy function nextName(index:int):String {
            Error.throwError(IllegalOperationError, 2106);
        }

        [Ruffle(NativeCallable)]
        flash_proxy function nextValue(index:int):* {
            Error.throwError(IllegalOperationError, 2107);
        }

        native flash_proxy function isAttribute(name:*):Boolean;
    }
}
