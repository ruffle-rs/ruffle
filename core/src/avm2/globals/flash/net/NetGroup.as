package flash.net {
    import flash.events.EventDispatcher;

    [API("667")]
    public class NetGroup extends EventDispatcher {
        private var _connection:NetConnection;
        private var _groupspec:String;
        private var _receiveMode:String = NetGroupReceiveMode.EXACT;
        private var _replicationStrategy:String = NetGroupReplicationStrategy.RAREST_FIRST;
        private var _info:NetGroupInfo = new NetGroupInfo();

        public function NetGroup(connection:NetConnection, groupspec:String) {
            super();
            if (connection == null) {
                Error.throwError(TypeError, 2007, "connection");
            }
            if (groupspec == null) {
                Error.throwError(TypeError, 2007, "groupspec");
            }
            this._connection = connection;
            this._groupspec = groupspec;
        }

        public function get estimatedMemberCount():Number {
            return 0;
        }

        public function get info():NetGroupInfo {
            return this._info;
        }

        public function get localCoverageFrom():String {
            return "";
        }

        public function get localCoverageTo():String {
            return "";
        }

        public function get neighborCount():Number {
            return 0;
        }

        public function get receiveMode():String {
            return this._receiveMode;
        }

        public function set receiveMode(mode:String):void {
            this._receiveMode = mode;
        }

        public function get replicationStrategy():String {
            return this._replicationStrategy;
        }

        public function set replicationStrategy(strategy:String):void {
            this._replicationStrategy = strategy;
        }

        public function addHaveObjects(startIndex:Number, endIndex:Number):void {
        }

        public function addMemberHint(peerID:String):Boolean {
            return false;
        }

        public function addWantObjects(startIndex:Number, endIndex:Number):void {
        }

        public function close():void {
        }

        public function convertPeerIDToGroupAddress(peerID:String):String {
            return peerID;
        }

        public function denyRequestedObject(requestID:int):void {
        }

        public function post(message:Object):String {
            return null;
        }

        public function removeHaveObjects(startIndex:Number, endIndex:Number):void {
        }

        public function removeWantObjects(startIndex:Number, endIndex:Number):void {
        }

        public function sendToAllNeighbors(message:Object):String {
            return NetGroupSendResult.NO_ROUTE;
        }

        public function sendToNearest(message:Object, groupAddress:String):String {
            return NetGroupSendResult.NO_ROUTE;
        }

        public function sendToNeighbor(message:Object, sendMode:String):String {
            return NetGroupSendResult.NO_ROUTE;
        }

        public function writeRequestedObject(requestID:int, object:Object):void {
        }
    }
}
