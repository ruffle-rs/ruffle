package {
	public class Test {
		public function Test() {
			function noArgs() {}
			function twoArgs(arg1:*, arg2:Number) {}
			function threeArgsWithRest(arg1:Boolean, arg2:Object, arg3:uint, ...rest) {}
		
			trace("noArgs.length = " + noArgs.length);
			trace("twoArgs.length = " + twoArgs.length);
			trace("threeArgsWithRest.length = " + threeArgsWithRest.length);
		
			var lambdaNoArgs = function() {}
			var lambdaTwoArgs = function(arg1:*, arg2:Number) {}
			var lambdaThreeArgsWithRest = function(arg1:Boolean, arg2:Object, arg3:uint, ...rest) {}
		
			trace("lambdaNoArgs.length = " + lambdaNoArgs.length);
			trace("lambdaTwoArgs.length = " + lambdaTwoArgs.length);
			trace("lambdaThreeArgsWithRest.length = " + lambdaThreeArgsWithRest.length);
		}
	}
}