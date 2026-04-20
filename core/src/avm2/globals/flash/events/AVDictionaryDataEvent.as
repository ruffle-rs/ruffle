package flash.events {
    import flash.utils.Dictionary;

    public class AVDictionaryDataEvent extends Event {
        public static const AV_DICTIONARY_DATA:String = "avDictionaryData";

        private var _dictionary:Dictionary;
        private var _time:Number;

        public function AVDictionaryDataEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            init_dictionary:Dictionary = null,
            init_dataTime:Number = 0
        ) {
            super(type, bubbles, cancelable);
            this._dictionary = init_dictionary;
            this._time = init_dataTime;
        }

        public function get dictionary():Dictionary {
            return this._dictionary;
        }

        public function get time():Number {
            return this._time;
        }
    }
}
