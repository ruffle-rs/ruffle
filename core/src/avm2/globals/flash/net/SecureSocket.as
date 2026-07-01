package flash.net {
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;

    import flash.utils.ByteArray;

    [API("662")]
    public class SecureSocket extends Socket {
        public function SecureSocket() {
            super();
            this.timeout = 20000;
        }

        public static native function get isSupported():Boolean;

        public native function get serverCertificateStatus():String;

        public function get serverCertificate():* {
            stub_getter("flash.net.SecureSocket", "serverCertificate");
            return null;
        }

        public function addBinaryChainBuildingCertificate(certificate:ByteArray, trusted:Boolean):void {
            stub_method("flash.net.SecureSocket", "addBinaryChainBuildingCertificate");
        }

        public override native function connect(host:String, port:int):void;
    }
}
