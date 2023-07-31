/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

 
 

import testdynfinpublicClassInitializer.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Class Initializers";  // Provide ECMA section title or a description
var BUGNUMBER = "";


var testInit= new testdyfinpublicClassInitializersWrap();
//print(testInit.MyNumber1());
//print(testInit.MyNumber2());
//print();
Assert.expectEq("Result from for loop",1,testInit.MyNumber1());
Assert.expectEq("Result from if else stt",2,testInit.MyNumber2());
Assert.expectEq("Result from do loop",2,testInit.MyNumber3());
              // displays results.
