/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Class Definition";       // Provide ECMA section title or a description
var BUGNUMBER = "";


//-----------------------------------------------------------------------------

import Bug118272Package.*;

import com.adobe.test.Assert;
import com.adobe.test.Utils;

var eg = new BugTest();
Assert.expectEq("Trying to initialize public  class identifier", "ReferenceError: Error #1074", Utils.referenceError(eg.thisError));
Assert.expectEq("Trying to initialize default class identifier", "ReferenceError: Error #1074", Utils.referenceError(eg.thisError1));
Assert.expectEq("Trying to initialize internal class identifier", "ReferenceError: Error #1074", Utils.referenceError(eg.thisError2));
Assert.expectEq("Trying to initialize dynamic class identifier", "ReferenceError: Error #1074", Utils.referenceError(eg.thisError3));
Assert.expectEq("Trying to initialize final class identifier", "ReferenceError: Error #1074", Utils.referenceError(eg.thisError4));



//test case in bug118272
class A { }
var thisError:String = "no error";
try{
A = null;
}catch(e:ReferenceError){
    thisError = e.toString();
}finally{
Assert.expectEq("Trying to initialize class identifier,the class is outside the package", "ReferenceError: Error #1074", Utils.referenceError(thisError));
}

              // displays results.
