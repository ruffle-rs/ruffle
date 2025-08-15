package flash.utils {
    import flash.events.EventDispatcher;
    import flash.events.TimerEvent;
    public class Timer extends EventDispatcher {
        private var _currentCount: int;
        private var _repeatCount: int;

        [Ruffle(NativeAccessible)]
        private var _delay: Number;

        [Ruffle(NativeAccessible)]
        private var _timerId: int = -1;

        // Returns 'true' if we should cancel the underlying Ruffle native timer
        [Ruffle(NativeAccessible)]
        private var _onUpdateClosure:Function;

        private function checkDelay(delay:Number): void {
            if (!isFinite(delay) || delay < 0) {
                throw new RangeError("Timer delay out of range", 2066);
            }
        }

        public function Timer(delay:Number, repeatCount:int=0) {
            this.checkDelay(delay);
            this._currentCount = 0;
            this._delay = delay;
            this._repeatCount = repeatCount;

            // We need this closure so we can access it by slot from native code
            // (rather than making it an instance method, which would force us
            // to access it as a bound method)
            var self:Timer = this;
            this._onUpdateClosure = function():Boolean {
                self._currentCount += 1;
                self.dispatchEvent(new TimerEvent(TimerEvent.TIMER, false, false));
                if (self.repeatCount != 0 && self._currentCount >= self._repeatCount) {
                    // This will make 'running' return false in a TIMER_COMPLETE event handler
                    self._timerId = -1;
                    self.dispatchEvent(new TimerEvent(TimerEvent.TIMER_COMPLETE, false, false));
                    return true;
                }
                return false;
            }
        }

        public function get currentCount(): int {
            return this._currentCount;
        }

        public function get delay(): Number {
            return this._delay;
        }

        public function set delay(value:Number): void {
            this.checkDelay(delay);
            this._delay = value;
            if (this.running) {
                this.updateDelay();
            }
        }

        private native function updateDelay():void;

        public function get repeatCount(): int {
            return this._repeatCount;
        }

        public function set repeatCount(value:int): void {
            this._repeatCount = value;
            if (this._repeatCount != 0 && this._repeatCount <= this._currentCount) {
                this.stop();
            }
        }

        public function get running(): Boolean {
            return this._timerId != -1;
        }

        public function reset():void {
            this._currentCount = 0;
            this.stop();
        }

        public native function stop():void;
        public native function start():void;
    }
}
