package flash.globalization {
    [API("667")]
    public final class DateTimeFormatter {
        private var _dateStyle:String;
        private var _dateTimePattern:String;
        private var _localeIDName:String;
        private var _timeStyle:String;
        private var _lastOperationStatus:String = LastOperationStatus.NO_ERROR;

        private static const MONTHS_FULL:Array = [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December"
        ];
        private static const MONTHS_SHORT:Array = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun",
            "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"
        ];
        private static const DAYS_FULL:Array = [
            "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"
        ];
        private static const DAYS_SHORT:Array = [
            "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"
        ];

        private static function throwNonNull(name:String):void {
            Error.throwError(TypeError, 2007, name);
        }

        public function DateTimeFormatter(requestedLocaleIDName:String, dateStyle:String = "long", timeStyle:String = "long") {
            if (requestedLocaleIDName == null) {
                throwNonNull("requestedLocaleIDName");
            }
            this._localeIDName = requestedLocaleIDName;
            this.setDateTimeStyles(dateStyle, timeStyle);
        }

        public function get actualLocaleIDName():String {
            return this._localeIDName == LocaleID.DEFAULT ? "en-US" : this._localeIDName;
        }

        public function get lastOperationStatus():String {
            return this._lastOperationStatus;
        }

        public function get requestedLocaleIDName():String {
            return this._localeIDName;
        }

        private static function pad2(n:int):String {
            return n < 10 ? "0" + n : String(n);
        }

        private function formatWithPattern(d:Date, utc:Boolean):String {
            var year:int = utc ? d.fullYearUTC : d.fullYear;
            var month:int = utc ? d.monthUTC : d.month;
            var day:int = utc ? d.dateUTC : d.date;
            var weekday:int = utc ? d.dayUTC : d.day;
            var hour24:int = utc ? d.hoursUTC : d.hours;
            var minute:int = utc ? d.minutesUTC : d.minutes;
            var second:int = utc ? d.secondsUTC : d.seconds;
            var hour12:int = hour24 % 12;
            if (hour12 == 0) {
                hour12 = 12;
            }
            var ampm:String = hour24 < 12 ? "AM" : "PM";

            var out:String = "";
            var pat:String = this._dateTimePattern;
            var i:int = 0;
            while (i < pat.length) {
                var ch:String = pat.charAt(i);
                if (ch == "'") {
                    i++;
                    while (i < pat.length && pat.charAt(i) != "'") {
                        out += pat.charAt(i);
                        i++;
                    }
                    i++;
                    continue;
                }
                var end:int = i + 1;
                while (end < pat.length && pat.charAt(end) == ch) {
                    end++;
                }
                var runLen:int = end - i;
                switch (ch) {
                    case "y":
                        out += runLen == 2 ? pad2(year % 100) : String(year);
                        break;
                    case "M":
                        if (runLen >= 4) {
                            out += MONTHS_FULL[month];
                        } else if (runLen == 3) {
                            out += MONTHS_SHORT[month];
                        } else if (runLen == 2) {
                            out += pad2(month + 1);
                        } else {
                            out += String(month + 1);
                        }
                        break;
                    case "E":
                        out += runLen >= 4 ? DAYS_FULL[weekday] : DAYS_SHORT[weekday];
                        break;
                    case "d":
                        out += runLen >= 2 ? pad2(day) : String(day);
                        break;
                    case "H":
                        out += runLen >= 2 ? pad2(hour24) : String(hour24);
                        break;
                    case "h":
                        out += runLen >= 2 ? pad2(hour12) : String(hour12);
                        break;
                    case "m":
                        out += runLen >= 2 ? pad2(minute) : String(minute);
                        break;
                    case "s":
                        out += runLen >= 2 ? pad2(second) : String(second);
                        break;
                    case "a":
                        out += ampm;
                        break;
                    default:
                        out += pat.substring(i, end);
                        break;
                }
                i = end;
            }
            return out;
        }

        public function format(dateTime:Date):String {
            if (dateTime == null) {
                throwNonNull("dateTime");
            }
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            return this.formatWithPattern(dateTime, false);
        }

        public function formatUTC(dateTime:Date):String {
            if (dateTime == null) {
                throwNonNull("dateTime");
            }
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            return this.formatWithPattern(dateTime, true);
        }

        public static function getAvailableLocaleIDNames():Vector.<String> {
            return new <String>["en-US"];
        }

        public function getDateStyle():String {
            return this._dateStyle;
        }

        public function getDateTimePattern():String {
            return this._dateTimePattern;
        }

        public function getFirstWeekday():int {
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            return 0;
        }

        public function getMonthNames(nameStyle:String = "full", context:String = "standalone"):Vector.<String> {
            if (nameStyle == null) {
                throwNonNull("nameStyle");
            }
            if (context == null) {
                throwNonNull("context");
            }
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            var src:Array = (nameStyle == DateTimeNameStyle.LONG_ABBREVIATION || nameStyle == DateTimeNameStyle.SHORT_ABBREVIATION)
                ? MONTHS_SHORT
                : MONTHS_FULL;
            var res:Vector.<String> = new Vector.<String>();
            for each (var m:String in src) {
                res.push(m);
            }
            return res;
        }

        public function getTimeStyle():String {
            return this._timeStyle;
        }

        public function getWeekdayNames(nameStyle:String = "full", context:String = "standalone"):Vector.<String> {
            if (nameStyle == null) {
                throwNonNull("nameStyle");
            }
            if (context == null) {
                throwNonNull("context");
            }
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
            var src:Array = (nameStyle == DateTimeNameStyle.LONG_ABBREVIATION || nameStyle == DateTimeNameStyle.SHORT_ABBREVIATION)
                ? DAYS_SHORT
                : DAYS_FULL;
            var res:Vector.<String> = new Vector.<String>();
            for each (var d:String in src) {
                res.push(d);
            }
            return res;
        }

        public function setDateTimePattern(pattern:String):void {
            if (pattern == null) {
                throwNonNull("pattern");
            }
            this._dateTimePattern = pattern;
            this._dateStyle = DateTimeStyle.CUSTOM;
            this._timeStyle = DateTimeStyle.CUSTOM;
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
        }

        public function setDateTimeStyles(dateStyle:String, timeStyle:String):void {
            if (dateStyle == null) {
                throwNonNull("dateStyle");
            }
            if (timeStyle == null) {
                throwNonNull("timeStyle");
            }
            this._dateStyle = dateStyle;
            this._timeStyle = timeStyle;

            var dPat:String = "";
            switch (dateStyle) {
                case DateTimeStyle.SHORT:
                    dPat = "M/d/yyyy";
                    break;
                case DateTimeStyle.MEDIUM:
                    dPat = "MMM d, yyyy";
                    break;
                case DateTimeStyle.LONG:
                    dPat = "MMMM d, yyyy";
                    break;
                case DateTimeStyle.FULL:
                    dPat = "EEEE, MMMM d, yyyy";
                    break;
            }

            var tPat:String = "";
            switch (timeStyle) {
                case DateTimeStyle.SHORT:
                    tPat = "h:mm a";
                    break;
                case DateTimeStyle.MEDIUM:
                case DateTimeStyle.LONG:
                case DateTimeStyle.FULL:
                    tPat = "h:mm:ss a";
                    break;
            }

            if (dPat.length > 0 && tPat.length > 0) {
                this._dateTimePattern = dPat + " " + tPat;
            } else if (dPat.length > 0) {
                this._dateTimePattern = dPat;
            } else if (tPat.length > 0) {
                this._dateTimePattern = tPat;
            }
            this._lastOperationStatus = LastOperationStatus.NO_ERROR;
        }
    }
}
