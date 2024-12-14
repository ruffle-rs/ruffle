package flash.security
{
    [API("674")]
    public final class X509Certificate
    {
        import flash.utils.ByteArray;

        private var _encoded: ByteArray;
        private var _issuer: X500DistinguishedName;
        private var _issuerUniqueID: String;
        private var _serialNumber: String;
        private var _signatureAlgorithmOID: String;
        private var _signatureAlgorithmParams: ByteArray;
        private var _subject: X500DistinguishedName;
        private var _subjectPublicKey: String;
        private var _subjectPublicKeyAlgorithmOID: String;
        private var _subjectUniqueID: String;
        private var _validNotAfter: Date;
        private var _validNotBefore: Date;
        private var _version: uint;

        public function X509Certificate() {}

        public function get encoded():ByteArray {
            return this._encoded;
        }
        
        public function get issuer():X500DistinguishedName {
            return this._issuer;
        }

        public function get issuerUniqueID():String {
            return this._issuerUniqueID;
        }

        public function get serialNumber():String {
            return this._serialNumber;
        }

        public function get signatureAlgorithmOID():String {
            return this._signatureAlgorithmOID;
        }

        public function get signatureAlgorithmParams():ByteArray {
            return this._signatureAlgorithmParams;
        }

        public function get subject():X500DistinguishedName {
            return this._subject;
        }
        
        public function get subjectPublicKey():String {
            return this._subjectPublicKey;
        }

        public function get subjectPublicKeyAlgorithmOID():String {
            return this._subjectPublicKeyAlgorithmOID;
        }

        public function get subjectUniqueID():String {
            return this._subjectUniqueID;
        }

        public function get validNotAfter():Date {
            return this._validNotAfter;
        }

        public function get validNotBefore():Date {
            return this._validNotBefore;
        }

        public function get version():uint {
            return this._version;
        }
    }
}
