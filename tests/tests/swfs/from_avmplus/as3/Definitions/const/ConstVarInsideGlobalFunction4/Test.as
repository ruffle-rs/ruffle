/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Definitions\const";                  // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";           // Version of JavaScript or ECMA
// var TITLE   = "Initialize a local const in function with a global const";       // Provide ECMA section title or a description
var BUGNUMBER = "";


function constMultiArgs( arg1, arg2, arg3 ) {

    const localVar = arg1 + arg2 + arg3;
    return localVar;

}


const myVar:Number = -10;

Assert.expectEq("Initialize global multi-arg function local const with global const", 30, constMultiArgs( 10, myVar, 30 ));

