/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Expressions";        // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";                // Version of ECMAScript or ActionScript
// var TITLE   = "delete operator";    // Provide ECMA section title or a description
var BUGNUMBER = "";


// create the new object "account"
var account:Object = new Object();

// create function
account.func = function () { return "account.func"; };

Assert.expectEq("object's function", "function Function() {}", account.func.toString());
Assert.expectEq("call object's function", "account.func", account.func());
Assert.expectEq("delete instantiated object's function", true, delete account.func);
Assert.expectEq("account.func", undefined, account.func);


              // displays results.
