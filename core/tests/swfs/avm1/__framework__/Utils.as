class Utils {
    /**
    * Given a list of lists, where each list contains every possible value for that group,
    * this function will generate the cartesian product and call the given callback with
    * each valid set of .
    * 
    * For example, given the input:
    * [[1, 2], ["x", "y"], ["purple", "yellow"]]
    * You can expect the following behaviour:
    * callback([1, "x", "purple"]);
    * callback([2, "x", "purple"]);
    * callback([1, "y", "purple"]);
    * callback([2, "y", "purple"]);
    * callback([1, "x", "yellow"]);
    * callback([2, "x", "yellow"]);
    * callback([1, "y", "yellow"]);
    * callback([2, "y", "yellow"]);
    */
    static function cartesianProduct(input, callback) {
        if (input.length === 0) {
            callback([]);
            return;
        }

        var indices = [];

        for (var i = 0; i < input.length; i++) {
            indices.push(0);
        }

        while (true) {
            var result = [];
            for (var i = 0; i < input.length; i++) {
                result.push(input[i][indices[i]]);
            }
            callback(result);

            var index = 0;

            while(true) {
                indices[index]++;
                if (indices[index] < input[index].length) {
                    break;
                }

                indices[index] = 0;
                index++;

                if (index === input.length) {
                    return;
                }
            }
        }
    }

    /**
    * Escapes the given string so that it's valid ActionScript. 
    *
    * For example, escapeString("Hello\tWorld") will result in \"Hello\\tWorld\"
    */
    static function escapeString(string) {
        var result = "\"";

        for (var i = 0; i < result.length; i++) {
            var c = string.charAt(i);
            if (c === "\"") {
                result += "\\\"";
            } else if (c == "\\") {
                result += "\\\\";
            } else if (c == "\n") {
                result += "\\n";
            } else if (c == "\r") {
                result += "\\r";
            } else if (c == "\t") {
                result += "\\t";
            } else if (c == "\b") {
                result += "\\b";
            } else if (c == "\f") {
                result += "\\f";
            } else {
                result += c;
            }
        }

        return result + "\"";
    }

    /**
    * If the input is a string, this escapes it. Otherwise, returns String(input).
    */
    static function stringify(input) {
        if (typeof input === "string") {
            return Utils.escapeString(input);
        } else {
            return String(input);
        }
    }
}

/*
function testCallback(values) {
    var result = "[";
    for (var i = 0; i < values.length; i++) {
        result += values[i];
        if (i < values.length - 1) {
            result += ", ";
        }
    }
    result += "]";
    trace(result);
}

Utils.cartesianProduct([[1, 2], ["x", "y"], ["purple", "yellow"]], testCallback);
*/

/*
trace(Utils.escapeString("Escape \"this\" string\nplease! \b\f\n\r\t\"\\"));
trace(Utils.escapeString("HELLO WORLD"));
*/