/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


// var SECTION = "Directives\const";                       // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";                   // Version of JavaScript or ECMA
// var TITLE   = "'const' inside a class access from outside the class";   // Provide ECMA section title or a description
var BUGNUMBER = "";



/*===========================================================================*/


class ConstClass {
    
    const myConst = 10;
}

var Obj = new ConstClass();
myObjConst = Obj.myConst;


Assert.expectEq( "Testing the 'const' keywords access from an object of a class: const myConst = 10;", 10, myObjConst );


            // This function is for executing the test case and then
            // displaying the result on to the console or the LOG file.
