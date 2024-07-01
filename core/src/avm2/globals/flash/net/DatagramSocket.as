package flash.net {
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    import flash.utils.ByteArray;

    [API("668")] // AIR 2.0
    [Ruffle(InstanceAllocator)]
    public class DatagramSocket {
        public function DatagramSocket() {}

        public static function get isSupported():Boolean {
            return false;
        }

        public function get bound():Boolean {
            stub_getter("flash.net.DatagramSocket", "bound");
            return false;
        }

        public function get connected():Boolean {
            stub_getter("flash.net.DatagramSocket", "connected");
            return false;
        }

        public function get localAddress():String {
            stub_getter("flash.net.DatagramSocket", "localAddress");
            return null;
        }

        public function get localPort():String {
            stub_getter("flash.net.DatagramSocket", "localPort");
            return 0;
        }

        public function get remoteAddress():String {
            stub_getter("flash.net.DatagramSocket", "remoteAddress");
            return null;
        }

        public function get remotePort():int {
            stub_getter("flash.net.DatagramSocket", "remotePort");
            return 0;
        }

        public function bind(localPort:int = 0, localAddress = "0.0.0.0"):void {
            stub_method("flash.net.DatagramSocket", "bind");
        }

        public function close():void {
            stub_method("flash.net.DatagramSocket", "close");
        }

        public function connect(remoteAddress:String, remotePort:int):void {
            stub_method("flash.net.DatagramSocket", "connect");
        }

        public function receive():void {
            stub_method("flash.net.DatagramSocket", "receive");
        }

        public function send(bytes:ByteArray, offset:uint = 0, length:uint = 0, address:String = null, port:int = 0):void {
            stub_method("flash.net.DatagramSocket", "send");
        }
    }
}