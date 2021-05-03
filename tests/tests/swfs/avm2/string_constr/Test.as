package {
	public class Test {}
}

trace("//new String();");
trace(new String());

trace("//new String(undefined);");
trace(new String(undefined));
trace("//new String(null);");
trace(new String(null));

trace("//new String(false);");
trace(new String(false));
trace("//new String(true);");
trace(new String(true));

trace("//new String(0);");
trace(new String(0));
trace("//new String(123);");
trace(new String(123));
trace("//new String(-1.23);");
trace(new String(-1.23));

trace("//new String(\"\");");
trace(new String(""));
trace("//new String(\"abc012aáâ!?*你好こんにちはمَرحَبًا\");");
trace(new String("abc012aáâ!?*你好こんにちはمَرحَبًا"));

trace("//new String(new Object());");
trace(new String(new Object()));
trace("//function f():void {}");
trace("//new String(f);");
function f():void {}
trace(new String(f));
