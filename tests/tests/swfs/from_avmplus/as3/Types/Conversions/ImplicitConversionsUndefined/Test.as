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



// Value = undefined

var thisError = "no exception thrown";
try{
    var string:String = undefined;
} catch (e0) {
    thisError = e0.toString();
} finally {
    Assert.expectEq( "var string:String =null", "no exception thrown", Utils.typeError(thisError));
        Assert.expectEq( "var string:String =null", null,string);
}

thisError = "no exception thrown";
try{
    var mynumber:Number = undefined;
} catch (e) {
    thisError = e.toString();
} finally {
    Assert.expectEq("number:Number = undefined", "no exception thrown", Utils.typeError(thisError) );
        Assert.expectEq("number:Number = undefined", NaN, mynumber );
}

thisError = "no exception thrown";
try{
    var myInt:int = undefined;
} catch(e1) {
    thisError = e1.toString();
} finally {
    Assert.expectEq("myInt:int = undefined", "no exception thrown", Utils.typeError(thisError) );
        Assert.expectEq("myInt:int = undefined", 0, myInt );
}

thisError = "no exception thrown";
try{
    var myUint:uint = undefined;
} catch(e2) {
    thisError = e2.toString();
} finally {
        Assert.expectEq("myUInt:uint = undefined", "no exception thrown", Utils.typeError(thisError));

    Assert.expectEq("myUInt:uint = undefined", 0, myUint);
}

thisError = "no exception thrown";
try{
    var myboolean:Boolean = undefined;
} catch(e3) {
    thisError = e3.toString();
} finally {
    Assert.expectEq("myboolean:Boolean = undefined", "no exception thrown", Utils.typeError(thisError) );
        Assert.expectEq("myboolean:Boolean = undefined", false, myboolean);
}

var object:Object = undefined;
Assert.expectEq( "var object:Object = null", null, object);





