package {
	import flash.display.MovieClip;

	public class Test extends MovieClip{
	}
}
import flash.utils.getQualifiedSuperclassName;
import com.very.long.namespace.example;
import flash.utils.ByteArray;

trace(getQualifiedSuperclassName(Test));

trace(getQualifiedSuperclassName(flash.utils.ByteArray));

trace(getQualifiedSuperclassName(example));

trace(getQualifiedSuperclassName(int));

trace(getQualifiedSuperclassName(String));

trace(getQualifiedSuperclassName(flash.display.Sprite));
