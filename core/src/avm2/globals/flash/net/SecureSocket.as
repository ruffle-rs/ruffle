package flash.net {
    import flash.security.CertificateStatus;
    import flash.security.X509Certificate;
    import flash.utils.ByteArray;

    [API("668")]
    public class SecureSocket extends Socket {
        private var _serverCertificateStatus:String = CertificateStatus.UNKNOWN;

        public function SecureSocket() {
            super();
        }

        public static function get isSupported():Boolean {
            return false;
        }

        public function get serverCertificateStatus():String {
            return this._serverCertificateStatus;
        }

        [API("674")]
        public function get serverCertificate():X509Certificate {
            return null;
        }

        [API("674")]
        public function addBinaryChainBuildingCertificate(certificate:ByteArray, trusted:Boolean):void {
            if (certificate == null) {
                Error.throwError(TypeError, 2007, "certificate");
            }
        }
    }
}
