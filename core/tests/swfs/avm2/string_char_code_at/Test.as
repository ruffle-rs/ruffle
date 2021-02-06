package {
	public class Test {}
}

trace("//\"abcdefg\".charCodeAt();")
trace("abcdefg".charCodeAt());
trace("//\"abcdefg\".charCodeAt(1);")
trace("abcdefg".charCodeAt(1));
trace("//\"abcdefg\".charCodeAt(1.1);")
trace("abcdefg".charCodeAt(1.1));
trace("//\"abcdefg\".charCodeAt(1.5);")
trace("abcdefg".charCodeAt(1.5));
trace("//\"abcdefg\".charCodeAt(7);")
trace("abcdefg".charCodeAt(7));
trace("//\"abcdefg\".charCodeAt(-1);")
trace("abcdefg".charCodeAt(-1));
trace("//\"abcdefg\".charCodeAt(NaN);")
trace("abcdefg".charCodeAt(NaN));
trace("//\"abcdefg\".charCodeAt(1.79e+308);")
trace("abcdefg".charCodeAt(1.79e+308));
trace("//\"abcdefg\".charCodeAt(Infinity);")
trace("abcdefg".charCodeAt(Infinity));
trace("//\"abcdefg\".charCodeAt(-Infinity);")
trace("abcdefg".charCodeAt(-Infinity));
trace("//\"あいうえお\".charCodeAt(1);")
trace("あいうえお".charCodeAt(1));
trace("//\"مَرحَبًا\".charCodeAt(1);")
trace("مَرحَبًا".charCodeAt(1));
trace("//\"👨‍👨‍👧‍👦\".charCodeAt(0);")
trace("👨‍👨‍👧‍👦".charCodeAt(0));
trace("//\"\".charCodeAt(0);")
trace("".charCodeAt(0));
