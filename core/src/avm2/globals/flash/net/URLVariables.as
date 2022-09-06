package flash.net {
    import flash.utils.escapeMultiByte;
    public dynamic class URLVariables {
	// TODO: construct from String
	// TODO: implement decode()
	public function URLVariables() {}

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
