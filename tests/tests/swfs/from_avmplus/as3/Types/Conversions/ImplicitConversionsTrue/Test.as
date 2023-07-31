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



// Value = true

var thisError = "no exception thrown";
try{
    var string:String = true;
} catch (e0) {
    thisError = e0.toString();
} finally {
    Assert.expectEq("string:String = true", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("string:String = true", "true", string);
}

thisError = "no exception thrown";
try{
    var number:Number = true;
} catch (e) {
    thisError = e.toString();
} finally {
    Assert.expectEq("number:Number = true", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("number:Number = true", 1, number);
}

thisError = "no exception thrown";
try{
    var myInt:int = true;
} catch(e1) {
    thisError = e1.toString();
} finally {
    Assert.expectEq("myInt:int = true", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("myInt:int = true", 1, myInt);
}

thisError = "no exception thrown";
try{
    var myUint:uint = true;
} catch(e2) {
    thisError = e2.toString();
} finally {
    Assert.expectEq("myUInt:uint = true", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("myUInt:uint = true", 1, myUint);
}

thisError = "no exception thrown";
try{
    var boolean:Boolean = true;
} catch(e3) {
    thisError = e3.toString();
} finally {
    Assert.expectEq("boolean:Boolean = true", "no exception thrown", Utils.typeError(thisError) );
}

var object:Object = true;
Assert.expectEq( "var object:Object = true", true, object);





