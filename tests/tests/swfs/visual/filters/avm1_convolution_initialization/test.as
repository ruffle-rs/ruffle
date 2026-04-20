// Compile with: mtasc -main -version 8 -header 200:150:30 -swf test.swf test.as

// TODO: Incorporate this into a more comprehensive test of ConvolutionFilter

import flash.filters.ConvolutionFilter;
class Test {
    static function main() {
	// Initializing a ConvolutionFilter and specifying a color
	var convoTest = new ConvolutionFilter(3,3,[1,1,1,1,1,1,1,1,1],9,0,true,true,128);
	trace(convoTest.alpha);
	trace(convoTest.bias);
	trace(convoTest.clamp);
	trace(convoTest.color);
	trace(convoTest.divisor);
	trace(convoTest.matrix);
	trace(convoTest.matrixX);
	trace(convoTest.matrixY);
	trace(convoTest.preserveAlpha);
	// Initializing a ConvolutionFilter without specifying a color
	var convoTest2 = new ConvolutionFilter(3,3,[1,1,1,1,1,1,1,1,1],9,0,true,true);
	trace(convoTest2.alpha);
	trace(convoTest2.bias);
	trace(convoTest2.clamp);
	trace(convoTest2.color);
	trace(convoTest2.divisor);
	trace(convoTest2.matrix);
	trace(convoTest2.matrixX);
	trace(convoTest2.matrixY);
	trace(convoTest2.preserveAlpha);
    }
}