/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Definitions\const";                  // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";           // Version of JavaScript or ECMA
// var TITLE   = "Initialize a local const inside a function after its initializer";       // Provide ECMA section title or a description
var BUGNUMBER = "";


function myConstArgs( arg1 ) {

    const n2;
    n2 = -( arg1 / 30 );
    return n2;
}


var thisError:String = "no error";
try
{
    myConstArgs( 10 );
}
catch(err)
{
    thisError = err.toString();
}
finally
{
    Assert.expectEq("Initialize a local const inside a function after its initializer", "Illegal write to local const n2", thisError);
}

