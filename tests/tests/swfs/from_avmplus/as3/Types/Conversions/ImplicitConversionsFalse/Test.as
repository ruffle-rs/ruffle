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



// Value = false

var thisError = "no exception thrown";
try{
    var string:String = false;
} catch (e0) {
    thisError = e0.toString();
} finally {
    Assert.expectEq("string:String = false", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("string:String = false", "false", string);
}

thisError = "no exception thrown";
try{
    var number:Number = false;
} catch (e) {
    thisError = e.toString();
} finally {
    Assert.expectEq("number:Number = false", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("number:Number = false", 0, number );
}

thisError = "no exception thrown";
try{
    var myInt:int = false;
} catch(e1) {
    thisError = e1.toString();
} finally {
    Assert.expectEq("myInt:int = false", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("myInt:int = false", 0, myInt );
}

thisError = "no exception thrown";
try{
    var myUint:uint = false;
} catch(e2) {
    thisError = e2.toString();
} finally {
    Assert.expectEq("myUInt:uint = false", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("myUInt:uint = false", 0, myUint );
}

thisError = "no exception thrown";
try{
    var boolean:Boolean = false;
} catch(e3) {
    thisError = e3.toString();
} finally {
    Assert.expectEq("boolean:Boolean = false", "no exception thrown", Utils.typeError(thisError) );
}

var object:Object = false;
Assert.expectEq( "var object:Object = false", false, object);





