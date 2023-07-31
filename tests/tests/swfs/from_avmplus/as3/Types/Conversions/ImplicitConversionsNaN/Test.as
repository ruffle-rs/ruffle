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



// Value = NaN

/*var thisError = "no exception thrown";
try{
    var string:String = NaN;
} catch (e0) {
    thisError = e0.toString();
} finally {
    Assert.expectEq( "var string:String = NaN", "no exception thrown", Utils.typeError(thisError));
    Assert.expectEq( "var string:String = NaN", "NaN", string);
}*/

var string:String = NaN;
Assert.expectEq( "var string:String = NaN", "NaN", string);

var number:Number = NaN;
Assert.expectEq("number:Number = NaN", NaN, number );

/*thisError = "no exception thrown";
try{
    var myInt:int = NaN;
} catch(e1) {
    thisError = e1.toString();
} finally {
    Assert.expectEq("myInt:int = NaN", "RangeError: Error #1061", Utils.rangeError(thisError) );
    Assert.expectEq("myInt:int = NaN", 0, myInt );
}*/

var myInt:int = NaN;
Assert.expectEq("myInt:int = NaN", 0, myInt );


/*thisError = "no exception thrown";
try{
    var myUint:uint = NaN;
} catch(e2) {
    thisError = e2.toString();
} finally {
    Assert.expectEq("myUInt:uint = NaN", "RangeError: Error #1061", Utils.rangeError(thisError) );
    Assert.expectEq("myUInt:uint = NaN", 0, myUint );
}*/

var myUint:uint = NaN;
Assert.expectEq("myUInt:uint = NaN", 0, myUint );


/*thisError = "no exception thrown";
try{
    var boolean:Boolean = NaN;
} catch(e3) {
    thisError = e3.toString();
} finally {
    Assert.expectEq("boolean:Boolean = NaN", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("boolean:Boolean = NaN", false, boolean );
}*/

var boolean:Boolean = NaN;
Assert.expectEq("boolean:Boolean = NaN", false, boolean );

var object:Object = NaN;
Assert.expectEq( "var object:Object = NaN", NaN, object);





