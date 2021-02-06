package {
	public class Test {}
}

trace("//\"abcdefg\".charAt();")
trace("abcdefg".charAt());
trace("//\"abcdefg\".charAt(1);")
trace("abcdefg".charAt(1));
trace("//\"abcdefg\".charAt(1.1);")
trace("abcdefg".charAt(1.1));
trace("//\"abcdefg\".charAt(1.5);")
trace("abcdefg".charAt(1.5));
trace("//\"abcdefg\".charAt(7);")
trace("abcdefg".charAt(7));
trace("//\"abcdefg\".charAt(-1);")
trace("abcdefg".charAt(-1));
trace("//\"abcdefg\".charAt(NaN);")
trace("abcdefg".charAt(NaN));
trace("//\"abcdefg\".charAt(1.79e+308);")
trace("abcdefg".charAt(1.79e+308));
trace("//\"abcdefg\".charAt(Infinity);")
trace("abcdefg".charAt(Infinity));
trace("//\"abcdefg\".charAt(-Infinity);")
trace("abcdefg".charAt(-Infinity));
trace("//\"あいうえお\".charAt(1);")
trace("あいうえお".charAt(1));
trace("//\"مَرحَبًا\".charAt(1);")
trace("مَرحَبًا".charAt(1));
trace("//\"👨‍👨‍👧‍👦\".charAt(0);")
trace("👨‍👨‍👧‍👦".charAt(0));
trace("//\"\".charAt(0);")
trace("".charAt(0));
