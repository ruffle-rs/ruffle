package flash.net {
[Ruffle(InstanceAllocator)]
    public class Responder {
        public function Responder(result:Function, status:Function = null) {
            init(result, status);
        }

        private native function init(result:Function, status:Function = null):void;
    }
}
