package flash.media {
    public class AVTagData {
        private var _data: String;
        private var _localTime: Number;

        public function AVTagData(
            init_data:String,
            init_localTime:Number
        ) {
            _data = init_data;
            _localTime = init_localTime;
        }

        public function get data():String {
            return this._data;
        }

        public function get localTime():Number {
            return this._localTime;
        }
    }
}
