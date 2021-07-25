package {
	public class Test {
	}
}

/* NOTE: This is not the actual source to this test.
 * 
 * This is just a tribute. The actual source is in test-0 and this file (as
 * well as the accompanying FLA) is provided purely for reference only.
 * 
 * The AS3 compiler in modern versions of Adobe Animate does not allow
 * generating references to the package-internal specializations of Vector, so
 * this code will yield runtime errors if compiled normally.
 * 
 * Instead, compile the test, disassemble the resulting ABC (using rabcasm),
 * and open the PackageInternalNs("__AS3__.vec") namespace in each access to
 * the Vector$... classes. After reassembling and running the movie you should
 * be able to get debug output (at least in Scout).
 */

trace("///Vector$int === Vector.<int>");
trace(Vector$int === Vector.<int>);

trace("///Vector$uint === Vector.<uint>");
trace(Vector$uint === Vector.<uint>);

trace("///Vector$double === Vector.<Number>");
trace(Vector$double === Vector.<Number>);

trace("///Vector$object === Vector.<Object>");
trace(Vector$object === Vector.<Object>);

trace("///Vector$object === Vector.<*>");
trace(Vector$object === Vector.<*>);