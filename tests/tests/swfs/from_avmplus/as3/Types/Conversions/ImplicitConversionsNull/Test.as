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



// Value = null

var string:String = null;
Assert.expectEq( "var string:String =null", null, string );

var thisError = "no exception thrown";
try{
    var mynumber:Number = null;
} catch (e) {
    thisError = e.toString();
} finally {
    Assert.expectEq("number:Number = null", "no exception thrown", Utils.typeError(thisError) );
        Assert.expectEq("number:Number = null", 0, mynumber);
}

thisError = "no exception thrown";
try{
    var myInt:int = null;
} catch(e1) {
    thisError = e1.toString();
} finally {
    Assert.expectEq("myInt:int = null", "no exception thrown", Utils.typeError(thisError) );
        Assert.expectEq("myInt:int = null", 0, myInt );
}

thisError = "no exception thrown";
try{
    var myUint:uint = null;
} catch(e2) {
    thisError = e2.toString();
} finally {
    Assert.expectEq("myUInt:uint = null", "no exception thrown", Utils.typeError(thisError) );
        Assert.expectEq("myUInt:uint = null", 0, myUint );
}

thisError = "no exception thrown";
try{
    var myboolean:Boolean = null;
} catch(e3) {
    thisError = e3.toString();
} finally {
    Assert.expectEq("boolean:Boolean = null", "no exception thrown", Utils.typeError(thisError) );
        Assert.expectEq("boolean:Boolean = null", false, myboolean);
}

var object:Object = null;
Assert.expectEq( "var object:Object = null", null, object);





