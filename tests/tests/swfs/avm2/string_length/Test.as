package {
	public class Test {}
}

trace("//\"\".length;");
trace("".length);
trace("//\"\\n\\r\".length;");
trace("\n\r".length);
trace("//\"\\t\".length;");
trace("\t".length);
trace("//\"abc012aáâ\".length;");
trace("abc012aáâ".length);
trace("//\"你好こんにちは\".length;");
trace("你好こんにちは".length);
trace("//\"مَرحَبًا\".length;");
trace("مَرحَبًا".length);
trace("//\"😀\".length;");
trace("😀".length);
trace("//\"👨‍👨‍👧‍👦\".length;");
trace("👨‍👨‍👧‍👦".length);
