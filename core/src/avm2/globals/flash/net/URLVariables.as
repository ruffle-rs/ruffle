package flash.net {
    namespace AS3 = "http://adobe.com/AS3/2006/builtin";

    import flash.utils.escapeMultiByte;
    import flash.utils.unescapeMultiByte;
    public dynamic class URLVariables {
        public function URLVariables(str: String = null) {
            if (str) {
                this.decode(str);
            }
        }

        public function decode(str: String) {
            for each (var pair in str.AS3::split("&")) {
                var splitIndex = pair.AS3::indexOf("=");
                if (splitIndex === -1) {
                    throw new Error("Error #2101: The String passed to URLVariables.decode() must be a URL-encoded query string containing name/value pairs.", 2101);
                }
                pair = pair.AS3::replace("+", " ");
                var prop = unescapeMultiByte(pair.AS3::slice(0, splitIndex));
                var val = unescapeMultiByte(pair.AS3::slice(splitIndex + 1));
                if (this[prop] == null) {
                    this[prop] = val;
                } else if (this[prop] instanceof Array) {
                    this[prop].push(val);
                } else {
                    this[prop] = [this[prop], val];
                }
            }
        }

        public function toString(): String {
            var acc : String = ""
            var sep :String = ""
            for (p in this) {
                var pe : String = escapeMultiByte(p);
                var val = this[p];
                if (val is Array) {
                    for (i in val) {
                        acc += sep;
                        acc += pe;
                        acc += "=";
                        acc += escapeMultiByte(val[i]);
                        sep = "&";
                    }
                    continue;
                }
                acc += sep;
                acc += pe;
                acc += "=";
                acc += escapeMultiByte(val);
                sep="&";
            }
            return acc;
        }
    }
}
