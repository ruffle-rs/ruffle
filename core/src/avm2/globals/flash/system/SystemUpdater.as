package flash.system {
    import flash.events.EventDispatcher;

    [API("668")]
    public class SystemUpdater extends EventDispatcher {
        public function SystemUpdater() {
            super();
        }

        public function cancel():void {
        }

        public function update(type:String):void {
        }
    }
}
