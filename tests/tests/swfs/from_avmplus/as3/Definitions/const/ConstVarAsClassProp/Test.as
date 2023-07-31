/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Definitions\const";                      // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";                   // Version of JavaScript or ECMA
// var TITLE   = "Operations on const variable inside a class";        // Provide ECMA section title or a description
var BUGNUMBER = "";






class myTestConst {

    public const num1 = 1;
    private const num2 = 2;


    function privatenum2() {
        
        return num2;
    }
}


// Start the tests.

var myConstObj = new myTestConst();

Assert.expectEq( "const public property", 1, myConstObj.num1 );
Assert.expectEq( "const private property", 2, myConstObj.privatenum2() );


            // This function is for executing the test case and then
            // displaying the result on to the console or the LOG file.
