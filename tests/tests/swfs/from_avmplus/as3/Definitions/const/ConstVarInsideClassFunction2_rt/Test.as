/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Definitions\const";                      // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";                   // Version of JavaScript or ECMA
// var TITLE   = "Initialize const inside a class function after its initializer";     // Provide ECMA section title or a description
var BUGNUMBER = "";






class myTestConst {

    const arg1, arg2;

    function myConstArgs( arg1, arg2 ) {
 
        const arg3;
        arg3 = arg1 + arg2;
        arg3 = arg3 - 10;
        arg3 = arg3 * 10;
        arg3 = arg3 / 10;
        return arg3;
    }
}


var thisError:String = "no error";
var Obj = new myTestConst();

try
{
    Obj.myConstArgs( 20, 10 )
}
catch(err)
{
    thisError = err.toString();
}
finally
{
    Assert.expectEq("Initialize local const inside a class function after its initializer", "Illegal write to local const arg3", thisError);
}

            // This function is for executing the test case and then
            // displaying the result on to the console or the LOG file.
