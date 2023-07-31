/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;





function Uint1Arg(n1:uint):uint {
 
 var n2:uint = n1;
 
 return n2;

}

function UintMultiArgs(n1:uint, n2:uint, n3:uint):uint {
 
 var n4:uint = n1 + n2 + n3;
 
 return n4;

}

// mt: adding different types of args test cases
function diffArgs( arg1:int, arg2:uint, arg3:Number ):uint{
    return arg1+arg2+arg3;
}

function returnNegUint():uint {
    return -10;
}

function addNegUintInFunc(){
    var a:uint;
    var b = -100;
    var c = 1;
    return (a = b+c);
}

Assert.expectEq( "Calling function with 1 uint argument", 1 , Uint1Arg(1) );
Assert.expectEq( "Calling function with 1 uint argument", 6 , UintMultiArgs(1,2,3) );

// RangeError precision exceptions

var pResult = null;
try{
    Uint1Arg(-1);
    pResult = "exception NOT caught";
} catch (e) {
    pResult = "exception caught";
}
Assert.expectEq( "Uint1Arg(-1)", "exception NOT caught" , pResult );
Assert.expectEq( "Uint1Arg(-1)", 4294967295 , Uint1Arg(-1) );
pResult = null;
try{
    UintMultiArgs(-1,-1,-1);
    pResult = "exception NOT caught";
} catch (e) {
    pResult = "exception caught";
}
Assert.expectEq( "UintMultiArgs(-1,-1,-1)", "exception NOT caught" , pResult );
Assert.expectEq( "UintMultiArgs(-1,-1,-1)", 4294967293, UintMultiArgs(-1,-1,-1) )
pResult = null;
try{
    diffArgs(-1,-1,-1);
    pResult = "exception NOT caught";
} catch (e) {
    pResult = "exception caught";
}
Assert.expectEq( "diffArgs(-1,-1,-1)", "exception NOT caught" , pResult );
Assert.expectEq( "diffArgs(-1,-1,-1)", 4294967293 , diffArgs(-1,-1,-1) );
pResult = null;
try{
    returnNegUint();
    pResult = "exception NOT caught";
} catch (e) {
    pResult = "exception caught";
}
Assert.expectEq( "returnNegUint()", "exception NOT caught" , pResult );
Assert.expectEq( "returnNegUint()", 4294967286 , returnNegUint() );

var n:Number = -20;

pResult = null;
try{
    Uint1Arg(n);
    pResult = "exception NOT caught";
} catch (e) {
    pResult = "exception caught";
}
Assert.expectEq( "var n:Number = -20; Uint1Arg(n)", "exception NOT caught" , pResult );



pResult = null;
try{
    UintMultiArgs(n,n,n);
    pResult = "exception NOT caught";
} catch (e) {
    pResult = "exception caught";
}
Assert.expectEq( "var n:Number = -20; UintMultiArgs(n,n,n)", "exception NOT caught" , pResult );

pResult = null;
try{
    diffArgs(n,n,n);
    pResult = "exception NOT caught";
} catch (e) {
    pResult = "exception caught";
}
Assert.expectEq( "var n:Number = -20; diffArgs(n,n,n)", "exception NOT caught" , pResult );

var i:int = -20;

pResult = null;
try{
    Uint1Arg(i);
    pResult = "exception NOT caught";
} catch (e) {
    pResult = "exception caught";
}
Assert.expectEq( "var i:int = -20; Uint1Arg(i)", "exception NOT caught" , pResult );

pResult = null;
try{
    UintMultiArgs(i,i,i);
    pResult = "exception NOT caught";
} catch (e) {
    pResult = "exception caught";
}
Assert.expectEq( "var i:int = -20; UintMultiArgs(i,i,i)", "exception NOT caught" , pResult );

pResult = null;
try{
    diffArgs(i,i,i);
    pResult = "exception NOT caught";
} catch (e) {
    pResult = "exception caught";
}
Assert.expectEq( "var i:int = -20; diffArgs(i,i,i)", "exception NOT caught" , pResult );

pResult = null;
try{
    addNegUintInFunc();
    pResult = "exception NOT caught";
} catch (e) {
    pResult = "exception caught";
}
Assert.expectEq( "add negitive number to uint in function", "exception NOT caught" , pResult );

              // displays results.
