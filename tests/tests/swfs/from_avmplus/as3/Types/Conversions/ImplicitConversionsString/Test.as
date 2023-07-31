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



// Value = "string"

var string:String = "string";
Assert.expectEq( "var string:String ='string'", "string", string );

var thisError = "no exception thrown";
try{
    var number:Number = "string";
} catch (e) {
//print( "hello?" );
    thisError = e.toString();
} finally {
    Assert.expectEq("number:Number = 'string'", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("number:Number = 'string'", NaN, number );
}

thisError = "no exception thrown";
try{
    var myInt:int = "string";
} catch(e1) {
    thisError = e1.toString();
} finally {
    Assert.expectEq("myInt:int = 'string'", "no exception thrown", Utils.typeError(thisError) );
        Assert.expectEq("myInt:int = 'string'", 0, myInt );
}


thisError = "no exception thrown";
try{
    var myUint:uint = "string";
} catch(e2) {
    thisError = e2.toString();
} finally {
    Assert.expectEq("myUInt:uint = 'string'", "no exception thrown", Utils.typeError(thisError) );
        Assert.expectEq("myUInt:uint = 'string'", 0, myUint );
}


thisError = "no exception thrown";
try{
    var boolean:Boolean = "string";
} catch(e3) {
    thisError = e3.toString();
} finally {
    Assert.expectEq("boolean:Boolean = 'string'", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("boolean:Boolean = 'string'", true, boolean );
}

var object:Object = "string";
Assert.expectEq( "var object:Object ='string'", "string", object);

// empty string conversions ---------------------------------------------------------------------
var emptyString:String = "";
Assert.expectEq( 'var string:String =""', "", emptyString );

thisError = "no exception thrown";
try{
    var number:Number = "";
} catch (e) {
    thisError = e.toString();
} finally {
    Assert.expectEq("number:Number = ''", "no exception thrown", Utils.typeError(thisError) );
    Assert.expectEq("number:Number = ''", 0, number );
}

thisError = "no exception thrown";
try{
    var myInt:int = "";
} catch(e1) {
    thisError = e1.toString();
} finally {
    Assert.expectEq("myInt:int = ''", "no exception thrown", Utils.typeError(thisError) );
        Assert.expectEq("myInt:int = ''", 0, myInt );
}


thisError = "no exception thrown";
try{
    var myUint:uint = "";
} catch(e2) {
    thisError = e2.toString();
} finally {
    Assert.expectEq("myUInt:uint = ''", "no exception thrown", Utils.typeError(thisError) );
        Assert.expectEq("myUInt:uint = ''", 0, myUint );
}


thisError = "no exception thrown";
try{
    var boolean:Boolean = "";
} catch(e3) {
    thisError = e3.toString();
} finally {
    Assert.expectEq("boolean:Boolean = ''", "no exception thrown", Utils.typeError(thisError) );

    //Note that the boolean result for empty string is opposite a non-empty string
    Assert.expectEq("boolean:Boolean = ''", false, boolean );
}

var object:Object = "";
Assert.expectEq( "var object:Object =''", "", object);





