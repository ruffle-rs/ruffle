package {
    [Ruffle(InstanceAllocator)]
    [Ruffle(CallHandler)]
    public dynamic class Date {
        public static const length:int = 7;

        prototype.valueOf = function():* {
            var d:Date = this;
            return d.AS3::valueOf();
        }
        prototype.toString = function():String {
            var d:Date = this;
            return d.AS3::toString();
        }
        prototype.toDateString = function():String {
            var d:Date = this;
            return d.AS3::toDateString();
        }
        prototype.toTimeString = function():String {
            var d:Date = this;
            return d.AS3::toTimeString();
        }
        prototype.toLocaleString = function():String {
            var d:Date = this;
            return d.AS3::toLocaleString();
        }
        prototype.toLocaleDateString = function():String {
            var d:Date = this;
            return d.AS3::toLocaleDateString();
        }
        prototype.toLocaleTimeString = function():String {
            var d:Date = this;
            return d.AS3::toLocaleTimeString();
        }
        prototype.toUTCString = function():String {
            var d:Date = this;
            return d.AS3::toUTCString();
        }
        prototype.toJSON = function(k:String):* {
            var d:Date = this;
            return d.AS3::toString();
        }
        prototype.getUTCFullYear = function():Number {
            var d:Date = this;
            return d.AS3::getUTCFullYear();
        }
        prototype.getUTCMonth = function():Number {
            var d:Date = this;
            return d.AS3::getUTCMonth();
        }
        prototype.getUTCDate = function():Number {
            var d:Date = this;
            return d.AS3::getUTCDate();
        }
        prototype.getUTCDay = function():Number {
            var d:Date = this;
            return d.AS3::getUTCDay();
        }
        prototype.getUTCHours = function():Number {
            var d:Date = this;
            return d.AS3::getUTCHours();
        }
        prototype.getUTCMinutes = function():Number {
            var d:Date = this;
            return d.AS3::getUTCMinutes();
        }
        prototype.getUTCSeconds = function():Number {
            var d:Date = this;
            return d.AS3::getUTCSeconds();
        }
        prototype.getUTCMilliseconds = function():Number {
            var d:Date = this;
            return d.AS3::getUTCMilliseconds();
        }
        prototype.getFullYear = function():Number {
            var d:Date = this;
            return d.AS3::getFullYear();
        }
        prototype.getMonth = function():Number {
            var d:Date = this;
            return d.AS3::getMonth();
        }
        prototype.getDate = function():Number {
            var d:Date = this;
            return d.AS3::getDate();
        }
        prototype.getDay = function():Number {
            var d:Date = this;
            return d.AS3::getDay();
        }
        prototype.getHours = function():Number {
            var d:Date = this;
            return d.AS3::getHours();
        }
        prototype.getMinutes = function():Number {
            var d:Date = this;
            return d.AS3::getMinutes();
        }
        prototype.getSeconds = function():Number {
            var d:Date = this;
            return d.AS3::getSeconds();
        }
        prototype.getMilliseconds = function():Number {
            var d:Date = this;
            return d.AS3::getMilliseconds();
        }
        prototype.getTimezoneOffset = function():Number {
            var d:Date = this;
            return d.AS3::getTimezoneOffset();
        }
        prototype.getTime = function():Number {
            var d:Date = this;
            return d.AS3::getTime();
        }
        prototype.setTime = function(t:* = undefined):Number {
            var d:Date = this;
            return d.AS3::setTime(t);
        }
        prototype.setFullYear = function(year:* = undefined, month:* = undefined, day:* = undefined):Number {
            var d:Date = this;
            return d._setFullYear(arguments);
        }
        prototype.setMonth = function(month:* = undefined, day:* = undefined):Number {
            var d:Date = this;
            return d._setMonth(arguments);
        }
        prototype.setDate = function(day:* = undefined):Number {
            var d:Date = this;
            return d._setDate(day);
        }
        prototype.setHours = function(hour:* = undefined, min:* = undefined, sec:* = undefined, ms:* = undefined):Number {
            var d:Date = this;
            return d._setHours(arguments);
        }
        prototype.setMinutes = function(min:* = undefined, sec:* = undefined, ms:* = undefined):Number {
            var d:Date = this;
            return d._setMinutes(arguments);
        }
        prototype.setSeconds = function(sec:* = undefined, ms:* = undefined):Number {
            var d:Date = this;
            return d._setSeconds(arguments);
        }
        prototype.setMilliseconds = function(ms:* = undefined):Number {
            var d:Date = this;
            return d._setMilliseconds(arguments);
        }
        prototype.setUTCFullYear = function(year:* = undefined, month:* = undefined, day:* = undefined):Number {
            var d:Date = this;
            return d._setUTCFullYear(arguments);
        }
        prototype.setUTCMonth = function(month:* = undefined, day:* = undefined):Number {
            var d:Date = this;
            return d._setUTCMonth(arguments);
        }
        prototype.setUTCDate = function(day:* = undefined):Number {
            var d:Date = this;
            return d._setUTCDate(arguments);
        }
        prototype.setUTCHours = function(hour:* = undefined, min:* = undefined, sec:* = undefined, ms:* = undefined):Number {
            var d:Date = this;
            return d._setUTCHours(arguments);
        }
        prototype.setUTCMinutes = function(min:* = undefined, sec:* = undefined, ms:* = undefined):Number {
            var d:Date = this;
            return d._setUTCMinutes(arguments);
        }
        prototype.setUTCSeconds = function(sec:* = undefined, ms:* = undefined):Number {
            var d:Date = this;
            return d._setUTCSeconds(arguments);
        }
        prototype.setUTCMilliseconds = function(ms:* = undefined):Number {
            var d:Date = this;
            return d._setUTCMilliseconds(arguments);
        }

        prototype.setPropertyIsEnumerable("valueOf", false);
        prototype.setPropertyIsEnumerable("toString", false);
        prototype.setPropertyIsEnumerable("toDateString", false);
        prototype.setPropertyIsEnumerable("toTimeString", false);
        prototype.setPropertyIsEnumerable("toLocaleString", false);
        prototype.setPropertyIsEnumerable("toLocaleDateString", false);
        prototype.setPropertyIsEnumerable("toLocaleTimeString", false);
        prototype.setPropertyIsEnumerable("toUTCString", false);
        prototype.setPropertyIsEnumerable("toJSON", false);
        prototype.setPropertyIsEnumerable("getUTCFullYear", false);
        prototype.setPropertyIsEnumerable("getUTCMonth", false);
        prototype.setPropertyIsEnumerable("getUTCDate", false);
        prototype.setPropertyIsEnumerable("getUTCDay", false);
        prototype.setPropertyIsEnumerable("getUTCHours", false);
        prototype.setPropertyIsEnumerable("getUTCMinutes", false);
        prototype.setPropertyIsEnumerable("getUTCSeconds", false);
        prototype.setPropertyIsEnumerable("getUTCMilliseconds", false);
        prototype.setPropertyIsEnumerable("getFullYear", false);
        prototype.setPropertyIsEnumerable("getMonth", false);
        prototype.setPropertyIsEnumerable("getDate", false);
        prototype.setPropertyIsEnumerable("getDay", false);
        prototype.setPropertyIsEnumerable("getHours", false);
        prototype.setPropertyIsEnumerable("getMinutes", false);
        prototype.setPropertyIsEnumerable("getSeconds", false);
        prototype.setPropertyIsEnumerable("getMilliseconds", false);
        prototype.setPropertyIsEnumerable("getTimezoneOffset", false);
        prototype.setPropertyIsEnumerable("getTime", false);
        prototype.setPropertyIsEnumerable("setTime", false);
        prototype.setPropertyIsEnumerable("setFullYear", false);
        prototype.setPropertyIsEnumerable("setMonth", false);
        prototype.setPropertyIsEnumerable("setDate", false);
        prototype.setPropertyIsEnumerable("setHours", false);
        prototype.setPropertyIsEnumerable("setMinutes", false);
        prototype.setPropertyIsEnumerable("setSeconds", false);
        prototype.setPropertyIsEnumerable("setMilliseconds", false);
        prototype.setPropertyIsEnumerable("setUTCFullYear", false);
        prototype.setPropertyIsEnumerable("setUTCMonth", false);
        prototype.setPropertyIsEnumerable("setUTCDate", false);
        prototype.setPropertyIsEnumerable("setUTCHours", false);
        prototype.setPropertyIsEnumerable("setUTCMinutes", false);
        prototype.setPropertyIsEnumerable("setUTCSeconds", false);
        prototype.setPropertyIsEnumerable("setUTCMilliseconds", false);


        public function Date(year:* = undefined, month:* = undefined, day:* = undefined, hours:* = undefined, minutes:* = undefined, seconds:* = undefined, ms:* = undefined) {
            this.init(arguments);
        }
        private native function init(args:Array);

        public static native function parse(date:*):Number;

        public static native function UTC(year:*, month:*, date:* = 1, hour:* = 0, minute:* = 0, second:* = 0, millisecond:* = 0, ... rest):Number;

        AS3 function valueOf():Number {
            return this.AS3::getTime();
        }

        AS3 native function toString():String;

        AS3 native function toDateString():String;

        AS3 native function toTimeString():String;

        AS3 native function toLocaleString():String;

        AS3 function toLocaleDateString():String {
            return this.AS3::toDateString();
        }

        AS3 native function toLocaleTimeString():String;

        AS3 native function toUTCString():String;

        AS3 native function getUTCDay():Number;

        AS3 native function getDay():Number;

        AS3 native function getTimezoneOffset():Number;

        AS3 native function getTime():Number;
        AS3 native function setTime(time:* = undefined):Number;

        AS3 native function getFullYear():Number;
        private native function _setFullYear(args:Array):Number;
        AS3 function setFullYear(year:* = undefined, month:* = undefined, day:* = undefined):Number {
            return _setFullYear(arguments);
        }

        AS3 native function getMonth():Number;
        private native function _setMonth(args:Array):Number;
        AS3 function setMonth(month:* = undefined, day:* = undefined):Number {
            return _setMonth(arguments);
        }

        AS3 native function getDate():Number;
        private native function _setDate(args:Array):Number;
        AS3 function setDate(day:* = undefined):Number {
            return _setDate(arguments);
        }

        AS3 native function getHours():Number;
        private native function _setHours(args:Array):Number;
        AS3 function setHours(hour:* = undefined, min:* = undefined, sec:* = undefined, ms:* = undefined):Number {
            return _setHours(arguments);
        }

        AS3 native function getMinutes():Number;
        private native function _setMinutes(args:Array):Number;
        AS3 function setMinutes(min:* = undefined, sec:* = undefined, ms:* = undefined):Number {
            return _setMinutes(arguments);
        }

        AS3 native function getSeconds():Number;
        private native function _setSeconds(args:Array):Number;
        AS3 function setSeconds(sec:* = undefined, ms:* = undefined):Number {
            return _setSeconds(arguments);
        }

        AS3 native function getMilliseconds():Number;
        private native function _setMilliseconds(args:Array):Number;
        AS3 function setMilliseconds(ms:* = undefined):Number {
            return _setMilliseconds(arguments);
        }

        AS3 native function getUTCFullYear():Number;
        private native function _setUTCFullYear(args:Array):Number;
        AS3 function setUTCFullYear(year:* = undefined, month:* = undefined, day:* = undefined):Number {
            return _setUTCFullYear(arguments);
        }

        AS3 native function getUTCMonth():Number;
        private native function _setUTCMonth(args:Array):Number;
        AS3 function setUTCMonth(month:* = undefined, day:* = undefined):Number {
            return _setUTCMonth(arguments);
        }

        AS3 native function getUTCDate():Number;
        private native function _setUTCDate(args:Array):Number;
        AS3 function setUTCDate(day:* = undefined):Number {
            return _setUTCDate(arguments);
        }

        AS3 native function getUTCHours():Number;
        private native function _setUTCHours(args:Array):Number;
        AS3 function setUTCHours(hour:* = undefined, min:* = undefined, sec:* = undefined, ms:* = undefined):Number {
            return _setUTCHours(arguments);
        }

        AS3 native function getUTCMinutes():Number;
        private native function _setUTCMinutes(args:Array):Number;
        AS3 function setUTCMinutes(min:* = undefined, sec:* = undefined, ms:* = undefined):Number {
            return _setUTCMinutes(arguments);
        }

        AS3 native function getUTCSeconds():Number;
        private native function _setUTCSeconds(args:Array):Number;
        AS3 function setUTCSeconds(sec:* = undefined, ms:* = undefined):Number {
            return _setUTCSeconds(arguments);
        }

        AS3 native function getUTCMilliseconds():Number;
        private native function _setUTCMilliseconds(args:Array):Number;
        AS3 function setUTCMilliseconds(ms:* = undefined):Number {
            return _setUTCMilliseconds(arguments);
        }


        public function get fullYear():Number {
            return this.AS3::getFullYear();
        }
        public function set fullYear(value:Number):void {
            this.AS3::setFullYear(value);
        }

        public function get month():Number {
            return this.AS3::getMonth();
        }
        public function set month(value:Number):void {
            this.AS3::setMonth(value);
        }

        public function get date():Number {
            return this.AS3::getDate();
        }
        public function set date(value:Number):void {
            this.AS3::setDate(value);
        }

        public function get hours():Number {
            return this.AS3::getHours();
        }
        public function set hours(value:Number):void {
            this.AS3::setHours(value);
        }

        public function get minutes():Number {
            return this.AS3::getMinutes();
        }
        public function set minutes(value:Number):void {
            this.AS3::setMinutes(value);
        }

        public function get seconds():Number {
            return this.AS3::getSeconds();
        }
        public function set seconds(value:Number):void {
            this.AS3::setSeconds(value);
        }

        public function get milliseconds():Number {
            return this.AS3::getMilliseconds();
        }
        public function set milliseconds(value:Number):void {
            this.AS3::setMilliseconds(value);
        }

        public function get fullYearUTC():Number {
            return this.AS3::getUTCFullYear();
        }
        public function set fullYearUTC(value:Number):void {
            this.AS3::setUTCFullYear(value);
        }

        public function get monthUTC():Number {
            return this.AS3::getUTCMonth();
        }
        public function set monthUTC(value:Number):void {
            this.AS3::setUTCMonth(value);
        }

        public function get dateUTC():Number {
            return this.AS3::getUTCDate();
        }
        public function set dateUTC(value:Number):void {
            this.AS3::setUTCDate(value);
        }

        public function get hoursUTC():Number {
            return this.AS3::getUTCHours();
        }
        public function set hoursUTC(value:Number):void {
            this.AS3::setUTCHours(value);
        }

        public function get minutesUTC():Number {
            return this.AS3::getUTCMinutes();
        }
        public function set minutesUTC(value:Number):void {
            this.AS3::setUTCMinutes(value);
        }

        public function get secondsUTC():Number {
            return this.AS3::getUTCSeconds();
        }
        public function set secondsUTC(value:Number):void {
            this.AS3::setUTCSeconds(value);
        }

        public function get millisecondsUTC():Number {
            return this.AS3::getUTCMilliseconds();
        }
        public function set millisecondsUTC(value:Number):void {
            this.AS3::setUTCMilliseconds(value);
        }

        public function get time():Number {
            return this.AS3::getTime();
        }
        public function set time(value:Number):void {
            this.AS3::setTime(value);
        }

        public function get timezoneOffset():Number {
            return this.AS3::getTimezoneOffset();
        }

        public function get day():Number {
            return this.AS3::getDay();
        }

        public function get dayUTC():Number {
            return this.AS3::getUTCDay();
        }
    }
}
