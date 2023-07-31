/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Types: Conversions";
// var VERSION = "as3";
// var TITLE   = "implicit type conversions";



// Value = 1.23

/*var thisError = "no exception thrown";
try{
    var string:String = 1.23;
} catch (e0) {
    thisError = e0.toString();
} finally {
    Assert.expectEq( "var string:String = 1.23", "no exception thrown", Utils.typeError(thisError));
    Assert.expectEq( "var string:String = 1.23", "1.23", string);
}*/
var string:String = 1.23;
Assert.expectEq( "var string:String = 1.23", "1.23", string);

var number:Number = 1.23;
Assert.expectEq("number:Number = 1.23", 1.23, number );


/*thisError = "no exception thrown";
try{
    var myInt:int;
    myInt = 1.23;
} catch(e1) {
    thisError = e1.toString();
} finally {
    Assert.expectEq("myInt:int = 1.23", "RangeError: Error #1061", Utils.rangeError(thisError) );
}
print(myInt);*/

var myInt:int= 1.23;
Assert.expectEq("myint:int = 1.23", 1, myInt );

/*thisError = "no exception thrown";
try{
    var myUint:uint;
    myUint = 1.23;
} catch(e2) {
    thisError = e2.toString();
} finally {
    Assert.expectEq("myUInt:uint = 1.23", "RangeError: Error #1061", Utils.rangeError(thisError) );
}*/

var myUint:uint;
    myUint = 1.23;
Assert.expectEq("myUInt:uint = 1.23", 1, myUint);

/*thisError = "no exception thrown";
try{
    var boolean:Boolean = 1.23;
} catch(e3) {
    thisError = e3.toString();
} finally {
    Assert.expectEq("boolean:Boolean = 1.23", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("boolean:Boolean = 1.23", true, boolean);
}*/

var boolean:Boolean = 1.23;
Assert.expectEq("boolean:Boolean = 1.23", true, boolean);

var object:Object = 1.23;
Assert.expectEq( "var object:Object = 1.23", 1.23, object);





