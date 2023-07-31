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


function const1Arg( arg1 ) {

    const localVar = arg1;
    return localVar;
}


const myConstVar:Number = -10;

Assert.expectEq("Initialize global function local const with global const", -10, const1Arg( myConstVar ));

