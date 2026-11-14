package flash.net {
    import flash.utils.ByteArray;

    [API("667")]
    public class GroupSpecifier {
        private var _name:String;
        private var _client:Object = null;
        private var _ipMulticastMemberUpdatesEnabled:Boolean = false;
        private var _multicastEnabled:Boolean = false;
        private var _objectReplicationEnabled:Boolean = false;
        private var _peerToPeerDisabled:Boolean = false;
        private var _postingEnabled:Boolean = false;
        private var _routingEnabled:Boolean = false;
        private var _serverChannelEnabled:Boolean = false;
        private var _postingPassword:String = null;
        private var _publishPassword:String = null;
        private var _multicastAddresses:Array = [];
        private var _bootstrapPeers:Array = [];
        private var _uniqueSuffix:String = "";

        public function GroupSpecifier(name:String) {
            if (name == null || name.length == 0) {
                Error.throwError(ArgumentError, 2004);
            }
            this._name = name;
        }

        public static function get maxSupportedGroupspecVersion():int {
            return 2;
        }

        public function get minGroupspecVersion():int {
            return 1;
        }

        public function set minGroupspecVersion(version:int):void {
        }

        public function get client():Object {
            return this._client;
        }

        public function set client(value:Object):void {
            this._client = value;
        }

        public function get ipMulticastMemberUpdatesEnabled():Boolean {
            return this._ipMulticastMemberUpdatesEnabled;
        }

        public function set ipMulticastMemberUpdatesEnabled(enabled:Boolean):void {
            this._ipMulticastMemberUpdatesEnabled = enabled;
        }

        public function get multicastEnabled():Boolean {
            return this._multicastEnabled;
        }

        public function set multicastEnabled(enabled:Boolean):void {
            this._multicastEnabled = enabled;
        }

        public function get objectReplicationEnabled():Boolean {
            return this._objectReplicationEnabled;
        }

        public function set objectReplicationEnabled(enabled:Boolean):void {
            this._objectReplicationEnabled = enabled;
        }

        public function get peerToPeerDisabled():Boolean {
            return this._peerToPeerDisabled;
        }

        public function set peerToPeerDisabled(disable:Boolean):void {
            this._peerToPeerDisabled = disable;
        }

        public function get postingEnabled():Boolean {
            return this._postingEnabled;
        }

        public function set postingEnabled(enabled:Boolean):void {
            this._postingEnabled = enabled;
        }

        public function get routingEnabled():Boolean {
            return this._routingEnabled;
        }

        public function set routingEnabled(enabled:Boolean):void {
            this._routingEnabled = enabled;
        }

        public function get serverChannelEnabled():Boolean {
            return this._serverChannelEnabled;
        }

        public function set serverChannelEnabled(enabled:Boolean):void {
            this._serverChannelEnabled = enabled;
        }

        public static function encodeBootstrapPeerIDSpec(peerID:String):String {
            if (peerID == null) {
                Error.throwError(TypeError, 2007, "peerID");
            }
            return "010b" + peerID;
        }

        public static function encodeIPMulticastAddressSpec(
            address:String,
            port:* = null,
            source:String = null
        ):String {
            if (address == null) {
                Error.throwError(TypeError, 2007, "address");
            }
            var spec:String = address;
            if (port != null) {
                spec += ":" + String(port);
            }
            if (source != null) {
                spec += "@" + source;
            }
            return spec;
        }

        public static function encodePostingAuthorization(password:String):String {
            if (password == null) {
                Error.throwError(TypeError, 2007, "password");
            }
            return "0302" + password;
        }

        public static function encodePublishAuthorization(password:String):String {
            if (password == null) {
                Error.throwError(TypeError, 2007, "password");
            }
            return "0303" + password;
        }

        public function addBootstrapPeer(peerID:String):void {
            if (peerID == null) {
                Error.throwError(TypeError, 2007, "peerID");
            }
            this._bootstrapPeers.push(peerID);
        }

        public function addIPMulticastAddress(address:String, port:* = null, source:String = null):void {
            this._multicastAddresses.push(encodeIPMulticastAddressSpec(address, port, source));
        }

        public function setPostingPassword(password:String = null, salt:String = null):void {
            this._postingPassword = password;
        }

        public function setPublishPassword(password:String = null, salt:String = null):void {
            this._publishPassword = password;
        }

        public function makeUnique():void {
            this._uniqueSuffix = String(Math.random()).substring(2);
        }

        public function authorizations():String {
            var auth:String = "";
            if (this._postingPassword != null) {
                auth += encodePostingAuthorization(this._postingPassword);
            }
            if (this._publishPassword != null) {
                auth += encodePublishAuthorization(this._publishPassword);
            }
            return auth;
        }

        public function groupspecWithoutAuthorizations():String {
            var flags:String = "";
            if (this._serverChannelEnabled) flags += "S";
            if (this._routingEnabled) flags += "R";
            if (this._multicastEnabled) flags += "M";
            if (this._objectReplicationEnabled) flags += "O";
            if (this._postingEnabled) flags += "P";
            if (this._ipMulticastMemberUpdatesEnabled) flags += "I";
            if (this._peerToPeerDisabled) flags += "D";
            return "G:0101" + flags + "0e" + this._name + this._uniqueSuffix;
        }

        public function groupspecWithAuthorizations():String {
            return this.groupspecWithoutAuthorizations() + this.authorizations();
        }

        public function toString():String {
            return this.groupspecWithAuthorizations();
        }
    }
}
