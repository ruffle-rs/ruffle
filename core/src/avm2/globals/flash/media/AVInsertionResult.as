package flash.media {
    [API("688")]
    public class AVInsertionResult extends AVResult {
        private var _periodIndex:int;
        private var _insertedBeforeReadHead:Boolean;

        public function AVInsertionResult(
            result:int,
            periodIndex:int,
            insertedBeforeReadHead:Boolean
        ) {
            super(result);
            this._periodIndex = periodIndex;
            this._insertedBeforeReadHead = insertedBeforeReadHead;
        }

        public function get periodIndex():int {
            return this._periodIndex;
        }

        public function get insertedBeforeReadHead():Boolean {
            return this._insertedBeforeReadHead;
        }
    }
}
