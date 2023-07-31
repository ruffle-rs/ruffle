/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import DynamicClassSameMethPropNamePackage.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Add function with the same name as a property to Dynamic Class";  // Provide ECMA section title or a description
var BUGNUMBER = "";




var Obj = new DynamicClassSameMethPropName();
var thisError = "no exception given";
try{
    Obj.pubBoolean = function () { return 100;}
} catch (e){
    thisError = e.toString();
} finally {
    Assert.expectEq( "attempt to assign a function to a boolean property", "no exception given",Utils.typeError(thisError) );
    Assert.expectEq( "get function to a boolean property", true, Obj.pubBoolean );
}

thisError = "no exception given";
try{
    Obj.pubBoolean = 123;
} catch(e2) {
    thisError = e2.toString();
} finally {
    Assert.expectEq( "attempt to assign a number to a boolean property", "no exception given", Utils.typeError(thisError));
    Assert.expectEq( "get number assigned to a boolean property", true, Obj.pubBoolean);
}


              // displays results.
