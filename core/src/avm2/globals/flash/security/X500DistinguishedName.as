package flash.security
{
    [API("674")]
    public final class X500DistinguishedName
    {
        private var _commonName: String;
        private var _countryName: String;
        private var _localityName: String;
        private var _organizationalUnitName: String;
        private var _organizationName: String;
        private var _stateOrProvinceName: String;

        public function X500DistinguishedName() {}

        public function get commonName():String {
            return this._commonName;
        }
        
        public function get countryName():String {
            return this._countryName;
        }

        public function get localityName():String {
            return this._localityName;
        }

        public function get organizationalUnitName():String {
            return this._organizationalUnitName;
        }

        public function get organizationName():String {
            return this._organizationName;
        }

        public function get stateOrProvinceName():String {
            return this._stateOrProvinceName;
        }

        public function toString(): String {
            // TODO: figure out exact format
            return "C=" + this._countryName +
                   ",S=" + this._stateOrProvinceName +
                   ",L=" + this._localityName +
                   ",O=" + this._organizationName +
                   ",OU=" + this._organizationalUnitName +
                   ",CN=" + this._commonName;
        }
    }
}
